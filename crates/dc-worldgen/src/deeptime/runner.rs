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
//! ## The three orthogonal axes (material-behavior.md §5 "order × rate")
//! - **ORDER** — the topo-sort of `{reads, writes}`. Reproduces exactly the
//!   hand-written phase order of the old loop, and would *reject* an illegal one.
//! - **RATE** — each pass's [`super::cadence::Cadence`] (epochs per firing × turns
//!   per firing) and the `dt` = phase length it is handed. **BUILT 2026-07-29**
//!   (journal/0123, `docs/dependency-graph.md` E3): the cadence numbers are
//!   **authored data** ([`CadenceTable`], applied in [`deep_passes_with`]) over
//!   each pass's declared default, a firing pass takes its declared **sub-turns**
//!   with the cell state carried between them, and `dt` is **live** in the passes
//!   converted to read it. An empty table reproduces the shipped schedule bit for
//!   bit, and `dt = 1.0` there, so the axis landed against journal/0122's
//!   known-good fixed point without moving a golden.
//! - **SCHEDULE** — [`DeepPass::schedule`] ([`super::schedule::Schedule`]:
//!   `Seed` · `Step(Cadence)` · `SeedAndStep(Cadence)`). **BUILT 2026-07-29**
//!   (journal/0124). RATE says *how often*; this says *whether it steps at all,
//!   and whether anything establishes its state before the loop opens*. It
//!   replaced the runner's inherited *"a coarse-rate pass does not fire at epoch
//!   0"* rule, which is **deleted**: [`DeepSchedule::run`] seeds once, then fires
//!   every pass whose period divides the epoch, **including epoch 0**.
//!
//! ## Byte-identity
//! Each pass body is *literally the same call* the old loop made, in the same
//! order the topo-sort reproduces, with the same arguments — so the production
//! world hashes to the same goldens. This is a re-housing, not a rewrite.
//!
//! **One authorized exception, journal/0124:** deleting the skip rule gave the
//! head field a real solve at epoch 0 in place of a bare-surface placeholder, and
//! the vertical flux entries for chapter 0 moved with it. Terrain and strata —
//! `GOLDEN_SURFACE` / `GOLDEN_RECORD` — did not move, and could not: the head
//! field is a pure sidecar.
//!
//! ## The crossing constraint (north-star)
//! A [`DeepPass`] is plain data + opaque ids (`&'static str`, `&[DeepAxis]`
//! slices) + a bare `fn` pointer body — **no closures cross the pass seam**. The
//! rich state lives on the runner's side in [`DeepStepCtx`]; the declaration a
//! future SDK / WASM backend would marshal is the plain-data part.
//!
//! ## Where the declaration HISTORY lives
//! **`docs/design/pass-declaration-history.md`** — extracted 2026-07-29 under the
//! file-size doctrine (*split by LIVENESS, never topic*). It holds the archaeology:
//! how `reads_prev` came to be enforced (journal/0104), how the terrain-revision
//! chain grew its `Incised` link and why the `dc:deep/head` sidecar was floating
//! (journal/0107), the superseded defences that were answering the wrong question,
//! and the slice-neutrality arguments a test now proves. **What stays in this file
//! is the CONTRACT** — why each read is or is not declared, stated at the site it
//! governs.

use super::biotic::BioticSim;
use super::cadence::CadenceTable;
use super::erosion::Erosion;
use super::flux::FluxAccum;
use super::grid::{DeepConfig, DeepGrid, sea_level_at};
use super::inventory::FactLedger;
use super::schedule::Schedule;
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
/// **revision** token the next stage consumes — `Forced → Incised → Weathered →
/// Diffused → Compensated → Windblown → Settled`. That is the honest declaration
/// of a fixed-order relaxation pipeline, and it is what forces the topo-sort to
/// reproduce the old loop's phase order.
///
/// ## Reading a revision, and the writer that supersedes it
/// A revision token orders a reader **after** the stage that produced it. It does
/// **not**, on its own, order that reader **before** the next stage to overwrite
/// the same plane — those are two different tokens, and the graph sees two
/// different resources. Inside the erosion pipeline that never matters, because
/// each stage is braced on its far side by a *forward* edge into the stages after
/// it. A pure **sidecar** field pass has no such brace and floats.
///
/// So the declaration is the **pair**, never the single read: **declare the
/// revision you consume as a `reads`, and declare the next revision of the same
/// plane as a [`DeepPass::reads_prev`]** — an anti-dependency onto its writer. Lag
/// against the revision whose writer comes **first** after your read point, never
/// the last, and every later writer is covered transitively. It is the same shape
/// `expose`/`frost` use against [`Recorded`](DeepAxis::Recorded) (three writers,
/// all this epoch, all of which they must precede), generalised to a plane with
/// several revisions per epoch.
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
    /// **Terrain after stream transport + bedrock incision** — the first terrain
    /// revision *inside* the erosion pipeline. `transport` writes (it lowers `R`
    /// by incision and moves `H` by entrainment/deposition, in place); `weather`
    /// reads it, and `head` **lag-reads** it. It is the link that names the moment
    /// the ground surface changes between [`Forced`](DeepAxis::Forced) and
    /// [`Weathered`](DeepAxis::Weathered), so a pass reading `R+H` there can say
    /// *when* it read and has a writer to be ordered ahead of.
    Incised,
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
/// once (from the built grid + biota init + the tectonic schedule), driven
/// through [`DeepSchedule::run`]'s setup pass and then `cfg.iterations` epochs,
/// then destructured
/// into the [`super::DeepRun`]. It is the runner-side rich state the crossing
/// constraint keeps *off* the declaration seam.
pub struct DeepStepCtx<'a> {
    pub cfg: &'a DeepConfig,
    /// **The registered geology content this world is being laid with**
    /// (P11 slice 1). The deep tier used to be content-blind — it recorded a
    /// class and let the collapse tier invent the member — and deposition-time
    /// fitness is exactly what ends that. A pass reaches it through the ctx it is
    /// handed, never through a global, like every other capability here.
    pub geology: &'a dc_core::materials::geology::GeologySet,
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
    /// **The phase length handed to the firing pass** — epochs of world time this
    /// turn covers, `= cadence.dt() = period / sub_turns`. The RATE axis's
    /// time-base, and a **capability**: a pass reaches its clock through the ctx
    /// it is handed, never through a global.
    ///
    /// It is exactly `1.0` for every pass in the shipped roster's erosion
    /// pipeline, which is why passes could be converted to scale by it one at a
    /// time without moving a hash (`x * 1.0 == x`).
    ///
    /// **A pass that ignores `dt` is making a claim**, and the honest ones do: the
    /// climate march, the geotherm and the head field are **relaxations toward an
    /// equilibrium set by the current state**, not rates integrated over an
    /// interval, so their coarse period says *when to resample* and there is
    /// nothing for a duration to scale. Rate-shaped transformations — uplift,
    /// crustal thickening, hillslope creep, inventory weathering — do scale.
    ///
    /// **A seeding pass is handed `0.0`** ([`DeepSchedule::plan`]`(None)`) — a seed
    /// establishes t=0 state and integrates zero time, so every rate-shaped term
    /// in its body multiplies out. That is the SCHEDULE axis's half of the
    /// contract, and it is enforced rather than requested.
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

impl<'a> DeepStepCtx<'a> {
    /// The deposition-identity context for the epoch currently firing — the
    /// content set, the [`DeepMember`](crate::draws::DeepMember) stream seeded
    /// from this world's seed, and the epoch that addresses the draw.
    #[inline]
    pub fn member_ctx(&self) -> super::recorder::MemberCtx<'a> {
        Self::member_ctx_for(self.geology, self.cfg.seed, u64::from(self.epoch))
    }

    /// [`Self::member_ctx`] for a caller that is **outside the loop** — the
    /// post-loop coal promotion, which happens after the ctx has been destructured.
    #[inline]
    pub fn member_ctx_for(
        geology: &'a dc_core::materials::geology::GeologySet,
        seed: u64,
        epoch: u64,
    ) -> super::recorder::MemberCtx<'a> {
        super::recorder::MemberCtx::new(geology, seed, epoch)
    }
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
    /// **The kernel enforces this** — it is not an annotation (journal/0104).
    pub reads_prev: &'static [DeepAxis],
    /// **The SCHEDULE and RATE axes together** ([`super::schedule`],
    /// [`super::cadence`]): whether this pass runs before the loop, inside it, or
    /// both — and, when it steps, how often (`period`), how many turns per firing
    /// (`sub_turns`), and the `dt` = phase length the runner hands it. The value
    /// here is the pass's **declared default** — the pack author's opinion about
    /// its own process; a world may author a different **cadence** through a
    /// [`CadenceTable`], applied in [`deep_passes_with`], which never changes the
    /// [`Schedule`] variant.
    ///
    /// **Every pass in the shipped roster is [`Schedule::Step`]** — the three that
    /// were seeded before the loop were audited on the day the axis landed and all
    /// three turned out to be first-steps in disguise (journal/0124).
    pub schedule: Schedule,
    pub body: for<'a> fn(&mut DeepStepCtx<'a>),
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

/// External forcing, legacy path: add the uplift plane to bedrock, scaled by the
/// pass's phase length. The ledger value is the total uplift added.
fn forcing_legacy_pass(ctx: &mut DeepStepCtx<'_>) {
    let dt = ctx.dt;
    ctx.uplift_total += ctx.erosion.apply_uplift(&mut ctx.grid, dt);
}

/// External forcing, tectonic path: add the blended analytic thickening rate to
/// the crustal columns (elevation is derived later by isostasy). No ledger here
/// — the tectonic path's ledger input is the isostatic injection.
fn forcing_tectonic_pass(ctx: &mut DeepStepCtx<'_>) {
    let dt = ctx.dt;
    ctx.erosion.apply_thickening(&mut ctx.grid, dt);
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

/// Flux-limited hillslope diffusion of the regolith — **the pass the RATE axis
/// was sequenced for** (`stubs.md` § 30). It takes its phase length from the ctx
/// and sub-divides it internally to its own stability bound: authored `dt` from
/// outside, derived sub-steps inside.
fn diffuse_pass(ctx: &mut DeepStepCtx<'_>) {
    let dt = ctx.dt;
    ctx.erosion.diffuse(&mut ctx.grid, ctx.cfg, dt);
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
    let mem = ctx.member_ctx();
    ctx.erosion.record(&mut ctx.grid, mem);
}

/// The eolian agent: wind deflation + downwind loess/dune deposition.
fn eolian_pass(ctx: &mut DeepStepCtx<'_>) {
    let mem = ctx.member_ctx();
    ctx.erosion.wind(&mut ctx.grid, ctx.cfg, mem);
}

/// The littoral wave agent: wave-cut erosion at the current sea stand.
fn wave_pass(ctx: &mut DeepStepCtx<'_>) {
    let mem = ctx.member_ctx();
    ctx.erosion.wave(&mut ctx.grid, ctx.cfg, mem);
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
    let mem = ctx.member_ctx();
    if let Some(b) = ctx.biota.as_mut() {
        ctx.biotic_total += b.step(&mut ctx.grid, &ctx.erosion, ctx.epoch, mem);
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
/// reads the solve's own per-face outgoing discharge and load (plus `area`/the
/// routed surface for the sink case) exactly as the sim computed them, and writes
/// only its own record ([`DeepAxis::FlowFlux`], which nothing in the epoch reads)
/// — so, like the geotherm and the inventory-weathering pass, it cannot perturb
/// the erosion result. **Under MFD (flow.md § 2.6) those per-face planes carry
/// several non-zero faces per cell per epoch**, which is where simultaneous
/// divergence enters the archive; the pass itself did not have to change shape,
/// because a record of faces was already the right shape for a partition.
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
        ctx.erosion.out_face_area(),
        ctx.erosion.out_face_load(),
        ctx.erosion.area(),
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
/// Like the geotherm it **plants a field and runs no edges** — it never *writes*
/// `R`/`H`/the record — so the erosion result is byte-unchanged. But it **reads**
/// all three, and the ground surface is a boundary condition, not a detail: it
/// reads the drainage solve's filled surface (where free water stands) and drainage
/// area (which channels are perennial), the **start-of-epoch** record for the
/// column's materials exactly as `dc:deep/expose` does, and — via `grid.surf_at` —
/// the **[`Forced`](DeepAxis::Forced) terrain revision those solve outputs were
/// themselves built from. All four have to come from one moment: `filled`/`routed`
/// place the free water, `ground` caps the seepage face, and a mismatch would put
/// them on two different landscapes. That is why the declaration names both the
/// revision it consumes and the writer that supersedes it (journal/0107). A
/// coarse-rate pass
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
/// And the lag itself is **enforced, not merely annotated**: each `reads_prev`
/// becomes a reader→writer anti-dependency, so `weather`/`diffuse`/`eolian` are
/// ordered before `biotic`, and the three passes that lag-read
/// [`DeepAxis::Recorded`] (`expose`, `frost`, `head`) are ordered before
/// `deposition`/`eolian`/`wave`. Rename-proof, which is what
/// `a_lagged_reader_stays_ahead_of_its_writer_under_a_hostile_rename` proves.
///
/// **Two more lags are on the TERRAIN plane**, which — unlike
/// `Recorded` — has several revisions per epoch: `climate` lag-reads
/// [`DeepAxis::Forced`] (the start-of-epoch topography, before `forcing` touches
/// it) and `head` lag-reads [`DeepAxis::Incised`] (before `transport` incises it).
/// Both name the **first** revision after their read point, never the last, which
/// is what makes one token cover the rest of the chain transitively.
pub fn deep_passes(cfg: &DeepConfig) -> Vec<DeepPass> {
    deep_passes_with(cfg, &CadenceTable::empty())
}

/// [`deep_passes`], with the world's **authored cadence table** applied over each
/// pass's declared default — the RATE axis's data half
/// (`ARCHITECTURE.md` § *The engine is plugin-agnostic*; `material-behavior.md`
/// § 5).
///
/// The pack declares, the world authors, the engine executes. An **empty** table
/// is the shipped schedule exactly, which is why [`deep_passes`] can delegate here
/// without moving a golden.
///
/// A table entry naming a pass the roster gated off is ignored rather than
/// rejected — the roster's own `if` guards already mean "this world has no such
/// pass"; [`CadenceTable::unmatched`] is there for a caller that wants to be
/// strict about a typo.
///
/// **The table re-rates; it never re-schedules.** [`Schedule::with_cadence`]
/// preserves the variant, and a [`Schedule::Seed`] has no cadence to author — a
/// world may say how often a pass steps, but handing a step to a pass that
/// declares none is a *declared-epochs* question, not a rate one.
pub fn deep_passes_with(cfg: &DeepConfig, cadence: &CadenceTable) -> Vec<DeepPass> {
    let mut passes = declared_passes(cfg);
    if !cadence.is_empty() {
        for p in &mut passes {
            if let Some(declared) = p.schedule.cadence() {
                p.schedule = p.schedule.with_cadence(cadence.resolve(p.id, declared));
            }
        }
    }
    passes
}

/// The roster with every pass at its **declared** cadence, before any world
/// authoring. Split out so `deep_passes_with` has exactly one place to apply the
/// table and cannot miss a pass.
fn declared_passes(cfg: &DeepConfig) -> Vec<DeepPass> {
    let mut passes = Vec::new();

    // Climate — the one coarse-rate pass already in the loop.
    //
    // **It used to be seeded pre-loop (mod.rs) and skipped at epoch 0; it is now a
    // plain `Step` and there is no pre-loop block** (journal/0124). The audit was
    // decided by the bodies being *the same call*: `climate::march` is a pure
    // function of `(surface, sea level)` writing `grid.precip`, and this pass is
    // first in the order, so the epoch-0 firing recomputes exactly what the seed
    // wrote from exactly the same inputs. The seed was its first step, run twenty
    // epochs early to fill the hole the skip made.
    passes.push(DeepPass {
        id: "dc:deep/climate",
        reads: &[],
        writes: &[Climate],
        // **The terrain lag.** `climate::march` reads `grid.surf_at` — the
        // start-of-epoch topography, i.e. last epoch's final surface. The honest
        // declaration is an anti-dependency against the **first** revision of the
        // terrain plane this epoch produces, `Forced`: climate must run before
        // `forcing` touches it, and every later terrain writer (`transport` →
        // `weather` → `diffuse` → `isostasy` → the agents) is downstream of
        // `forcing`, so one token covers the whole chain transitively. Lagging
        // against the *last* revision instead (`Settled`/`Compensated`/`Diffused`,
        // one cfg-selected slice each) would be three declarations that pin
        // strictly less — they would let climate slide past `forcing` and
        // `transport`.
        reads_prev: &[Forced],
        schedule: Schedule::every(cfg.remarch_interval),
        body: climate_pass,
    });

    // Tectonic forcing (blend) → erosion's forcing plane.
    if cfg.tectonic_history {
        passes.push(DeepPass {
            id: "dc:deep/tectonics",
            reads: &[],
            writes: &[Forcing],
            reads_prev: &[],
            schedule: Schedule::EVERY_EPOCH,
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
            schedule: Schedule::EVERY_EPOCH,
            body: forcing_tectonic_pass,
        }
    } else {
        DeepPass {
            id: "dc:deep/forcing",
            reads: &[Climate],
            writes: &[Forced],
            reads_prev: &[],
            schedule: Schedule::EVERY_EPOCH,
            body: forcing_legacy_pass,
        }
    });

    // Expose the outcropping lithology (reads the start-of-epoch record).
    passes.push(DeepPass {
        id: "dc:deep/expose",
        reads: &[],
        writes: &[Exposed],
        reads_prev: &[Recorded],
        schedule: Schedule::EVERY_EPOCH,
        body: expose_pass,
    });

    // Periglacial frost agent.
    if cfg.full_agents {
        passes.push(DeepPass {
            id: "dc:deep/frost",
            reads: &[Forced],
            writes: &[Frosted],
            reads_prev: &[Recorded],
            schedule: Schedule::EVERY_EPOCH,
            body: frost_pass,
        });
    }

    // Drainage solve (surface / flood / route / accumulate).
    passes.push(DeepPass {
        id: "dc:deep/drainage",
        reads: &[Forced],
        writes: &[Routed],
        reads_prev: &[],
        schedule: Schedule::EVERY_EPOCH,
        body: drainage_pass,
    });

    // Transport + incision (one interleaved flux chain). It **mutates the terrain**
    // — bedrock incision lowers `R`, entrainment/deposition moves `H` — so it
    // publishes the `Incised` revision alongside its Energy/DeltaH by-products.
    // That token is what a sidecar reading `R+H` between `forcing` and `weather`
    // is ordered against.
    passes.push(DeepPass {
        id: "dc:deep/transport",
        reads: &[Routed, Exposed],
        writes: &[Energy, DeltaH, Incised],
        reads_prev: &[],
        schedule: Schedule::EVERY_EPOCH,
        body: transport_pass,
    });

    // The head field (FLOW continuation (a)): the `head` condition-field, relaxed
    // over the live topography and the start-of-epoch record. Gated behind
    // `head_field` (absent = off ⇒ the planes stay empty ⇒ the flow record's
    // vertical faces stay at slice 1's honest zero). Writes only `Head`.
    //
    // **It reads the TERRAIN, and says so** — the ground surface is one of the
    // solve's boundary conditions, not a detail (see [`head_pass`]). The revision is
    // **`Forced`**, in every cfg path, determined from the code rather than chosen:
    // the only writers between `forcing` and `transport` are `drainage`/`frost`/
    // `geotherm`, none of which touches `R`/`H`. It is also the revision the pass
    // *should* read, which is the stronger reason to pin it there — `filled`,
    // `routed` and `area` are all snapshots the drainage solve took from the
    // `Forced` terrain, so a `ground` from any later revision would put the seepage
    // cap and the free-water anchors on two different landscapes.
    //
    // Hence the **pair**: `reads: Forced` pins it after the forcing that produced
    // that terrain, and `reads_prev: Incised` pins it before `transport`, the first
    // pass to overwrite it. Neither alone is enough — `Forced` is a different
    // resource from `Incised`, so reading it says nothing about who comes after.
    // `reads_prev: Recorded` (the strata record `expose` also lag-reads) stays: it
    // is true independently, and declare what you read.
    //
    // **It used to be seeded pre-loop (mod.rs) and skipped at epoch 0; it is now a
    // plain `Step`** (journal/0124), and this is the one incumbent whose
    // reclassification moved bytes. That seed relaxed the potential over the BARE
    // surface — no `filled`, no `routed`, no `area`, because no drainage had run —
    // and epoch 0's flow record read it. Now epoch 0 fires the real solve, in its
    // pinned window, from all four boundary conditions on one landscape. The seed
    // was not an initial condition but a **degraded copy of the step**, kept alive
    // only by the skip it was written to repair.
    if cfg.head_field {
        passes.push(DeepPass {
            id: "dc:deep/head",
            reads: &[Routed, Forced],
            writes: &[Head],
            reads_prev: &[Recorded, Incised],
            schedule: Schedule::every(super::head::HEAD_PERIOD),
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
            schedule: Schedule::EVERY_EPOCH,
            body: flow_record_pass,
        });
    }

    // Bedrock weathering (reads the incised terrain — cover thickness `H` and
    // bedrock `R` as transport left them — and last epoch's biotic weathering
    // multiplier). `Incised` states the terrain revision it consumes.
    passes.push(DeepPass {
        id: "dc:deep/weather",
        reads: &[DeltaH, Exposed, Frosted, Incised],
        writes: &[DeltaH, Weathered],
        reads_prev: &[BioMod],
        schedule: Schedule::EVERY_EPOCH,
        body: weather_pass,
    });

    // Hillslope diffusion (reads last epoch's biotic root-cohesion resistance).
    passes.push(DeepPass {
        id: "dc:deep/diffuse",
        reads: &[Weathered, Exposed, DeltaH],
        writes: &[DeltaH, Diffused],
        reads_prev: &[BioMod],
        schedule: Schedule::EVERY_EPOCH,
        body: diffuse_pass,
    });

    // Exhumation + isostasy (tectonic path).
    if cfg.tectonic_history {
        passes.push(DeepPass {
            id: "dc:deep/isostasy",
            reads: &[Diffused, CrustThick],
            writes: &[Compensated],
            reads_prev: &[],
            schedule: Schedule::EVERY_EPOCH,
            body: isostasy_pass,
        });
    }

    // The geotherm field pass (the `temperature` condition-field). Coarse-rate,
    // and only on the tectonic path — it solves over the crustal columns, which
    // exist only there. Reads Climate (surface temperature, the field's boundary
    // condition) and CrustThick (the crustal thickness the gradient reads); writes
    // the Geotherm axis, read by no in-epoch pass.
    //
    // **It used to be seeded pre-loop (mod.rs) and skipped at epoch 0; it is now a
    // plain `Step` and there is no pre-loop block** (journal/0124). The seed was an
    // overwrite of a plane nothing reads until this pass next fires — a write into
    // the void the moment epoch 0 stopped being skipped. `geotherm::march` is a
    // pure recompute from the current crustal state, so the value that survives the
    // run is the last firing's either way, and the shipped world did not move.
    if cfg.tectonic_history {
        passes.push(DeepPass {
            id: "dc:deep/geotherm",
            reads: &[Climate, CrustThick],
            writes: &[Geotherm],
            reads_prev: &[],
            schedule: Schedule::every(super::geotherm::GEOTHERM_PERIOD),
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
            schedule: Schedule::EVERY_EPOCH,
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
            schedule: Schedule::EVERY_EPOCH,
            body: eolian_pass,
        });
        passes.push(DeepPass {
            id: "dc:deep/wave",
            reads: &[Exposed, Recorded, Windblown],
            writes: &[Recorded, Settled],
            reads_prev: &[],
            schedule: Schedule::EVERY_EPOCH,
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
            schedule: Schedule::EVERY_EPOCH,
            body: biotic_pass,
        });
    }

    // Inventory weathering (journal/0094): the first cellular material pass, gated
    // behind `weather_inventory` (absent = off, the old `if` as pass presence — so
    // the production world is byte-identical off, the S-5 identity default). Reads
    // the contemporaneous terrain (for this-epoch regolith `H`), Frosted, and **this
    // epoch's BioMod as a real within-epoch read** (reasoning at WINV_READS_*).
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
            schedule: Schedule::EVERY_EPOCH,
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
    /// reader → writer anti-dependencies. Handing the lag over is what makes it a
    /// mechanism instead of a comment.
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

    /// **Seed, then step.** Run every seeding pass once in topo order, then drive
    /// the epoch loop: for each epoch, set the per-epoch state and run every pass
    /// in topo order that fires this epoch, handing it its `dt`. This is the whole
    /// scheduler — order from the topo-sort, rate from the cadence, and
    /// pre-loop-vs-in-loop from the schedule.
    ///
    /// **The RATE axis is here, and it is two lines.** A firing pass takes
    /// [`Cadence::sub_turns`](super::cadence::Cadence::sub_turns) turns, each
    /// handed [`Cadence::dt`](super::cadence::Cadence::dt) as its phase length,
    /// and **each turn sees the cell state the previous one left** — that is the
    /// whole of the user's fractional-phase sketch (*"a phase is handed the cell
    /// state at its start… duration is a scalar on its transformations"*). One
    /// sub-turn is the shipped default, so the loop below is bit-identical to the
    /// pre-RATE one on every world already created.
    ///
    /// **The SCHEDULE axis is the loop around it** (journal/0124). There is **no
    /// epoch-0 exception**: [`Schedule::firing`] is `epoch % period == 0` and
    /// nothing else, so a period-20 pass fires at 0, 20, 40 … and integrates
    /// exactly as many epochs of world time as the run lasts. The shipped roster
    /// declares no seeder, so the setup loop below runs zero passes on it — but it
    /// runs zero passes *by roster*, not by omission.
    pub fn run(&self, ctx: &mut DeepStepCtx<'_>) {
        // --- the setup epoch: every seeding pass, once, at dt = 0 ---
        ctx.epoch = 0;
        ctx.sea_level = sea_level_at(ctx.cfg, 0);
        ctx.erosion.set_sea_level(ctx.sea_level);
        for t in self.plan(None) {
            ctx.dt = t.dt;
            (self.passes[t.index].body)(ctx);
        }

        for it in 0..ctx.cfg.iterations {
            ctx.epoch = it;
            ctx.sea_level = sea_level_at(ctx.cfg, it);
            // The erosion phase methods read `self.sea_level` (the paleo-sea-level
            // stand); the old `Erosion::step` set it as its first line. Set it once
            // per epoch here so every erosion sub-pass sees the same stand — this
            // is load-bearing for byte-identity (the sinusoidal stand drives the
            // shoreline, marine deposition, and the wave agent).
            ctx.erosion.set_sea_level(ctx.sea_level);
            for t in self.plan(Some(it)) {
                ctx.dt = t.dt;
                for _ in 0..t.turns {
                    (self.passes[t.index].body)(ctx);
                }
            }
        }
    }

    /// **The scheduler's decision, separated from its execution.** `epoch = None`
    /// is the **setup epoch** — the [`Schedule::Seed`] / [`Schedule::SeedAndStep`]
    /// passes, once each, at `dt = 0`; `Some(e)` is the passes that step on epoch
    /// `e`, with the `dt` and turn count each is handed. Both in topo order.
    ///
    /// **[`Self::run`] drives itself through this**, which is the point: a test
    /// that reads the plan is reading the decision the world is generated from,
    /// not a restatement of it beside it (`spines.md` A-1). Building a world to
    /// observe *when a pass runs* would also be measuring the wrong thing — the
    /// answer is a property of the roster, not of the terrain.
    ///
    /// **`dt = 0.0` for a seed is the mechanism, not a convention.** A seed
    /// establishes t=0 state and integrates none, so every rate-shaped
    /// transformation in its body multiplies out. A pass whose "seed" is really
    /// its first step cannot survive that — the discrimination the 2026-07-29
    /// audit of the three pre-loop incumbents had to make by hand, and all three
    /// failed it (journal/0124).
    #[must_use]
    pub fn plan(&self, epoch: Option<u32>) -> Vec<Turn> {
        self.order
            .iter()
            .filter_map(|&index| {
                let p = &self.passes[index];
                match epoch {
                    None => p.schedule.seeds().then_some(Turn {
                        id: p.id,
                        index,
                        dt: 0.0,
                        turns: 1,
                    }),
                    Some(e) => p.schedule.firing(e).map(|c| Turn {
                        id: p.id,
                        index,
                        dt: c.dt(),
                        turns: c.sub_turns(),
                    }),
                }
            })
            .collect()
    }
}

/// One entry of a [`DeepSchedule::plan`]: a pass that runs, the phase length it is
/// handed, and how many turns it takes at that length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Turn {
    pub id: &'static str,
    /// Index into the schedule's pass table — how [`DeepSchedule::run`] reaches
    /// the body without a second lookup.
    pub index: usize,
    /// [`Cadence::dt`](super::cadence::Cadence::dt), or `0.0` for a seed.
    pub dt: f64,
    /// [`Cadence::sub_turns`](super::cadence::Cadence::sub_turns), or `1` for a
    /// seed — sub-turning is a subdivision of a phase, and a seed has none.
    pub turns: u32,
}

#[cfg(test)]
mod tests {
    use super::super::cadence::Cadence;
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
        // The head field sits here **because the graph puts it here**, not because
        // the tie-break did. It reads `Forced`, so it cannot precede `forcing`; it
        // lag-reads `Incised`, so it cannot follow `transport`. That window is
        // exactly where its four boundary-condition inputs
        // (`filled`/`routed`/`area`/`ground`) all describe the same landscape — and
        // its position decides the head FIELD'S OWN VALUES, and therefore the
        // vertical flux the record keeps.
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
                schedule: Schedule::EVERY_EPOCH,
                body: transport_pass,
            },
            DeepPass {
                id: "dc:deep/biotic",
                reads: &[Routed],
                writes: &[BioMod],
                reads_prev: &[],
                schedule: Schedule::EVERY_EPOCH,
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
                schedule: Schedule::EVERY_EPOCH,
                body: transport_pass,
            },
            DeepPass {
                id: "dc:deep/biotic",
                reads: &[Routed],
                writes: &[BioMod],
                reads_prev: &[],
                schedule: Schedule::EVERY_EPOCH,
                body: biotic_pass,
            },
        ];
        assert_eq!(
            DeepSchedule::new(passes).unwrap().ordered_ids(),
            vec!["dc:deep/erosion", "dc:deep/biotic"],
        );
    }

    /// **The lag must survive a hostile rename.** Rename each lagged reader to an
    /// id that sorts **after** every writer of the axis it lags on, and assert the
    /// schedule still puts it first: the anti-dependency edge holds it in place and
    /// the id-lexicographic tie-break never gets a say. (Why this test exists —
    /// `docs/design/pass-declaration-history.md` § 2.)
    #[test]
    fn a_lagged_reader_stays_ahead_of_its_writer_under_a_hostile_rename() {
        // Every (lagged reader, lagged axis) pair in the production roster, with a
        // rename chosen to lose the alphabet fight against that axis's writers.
        for (victim, hostile) in [
            ("dc:deep/head", "dc:deep/zzz_head"),
            // journal/0107 added two more lagged readers of a terrain revision:
            // `climate` (before `forcing`) and `head` (before `transport`).
            ("dc:deep/climate", "dc:deep/zzz_climate"),
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
        // Coarse rate — heat flow evolves slowly, so it re-marches on a cadence.
        // **And it fires at epoch 0**, like everything else (journal/0124): the
        // pre-loop seed it used to skip for is gone.
        let geo_c = geo.schedule.cadence().expect("a stepping pass");
        assert!(geo_c.period() > 1);
        assert!(geo.schedule.fires(0));
        assert!(geo.schedule.fires(geo_c.period()));
        assert!(
            !geo.schedule.seeds(),
            "the pre-loop geotherm seed is retired"
        );
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
        assert_eq!(head.reads, &[DeepAxis::Routed, DeepAxis::Forced]);
        assert_eq!(head.reads_prev, &[DeepAxis::Recorded, DeepAxis::Incised]);
        assert_eq!(head.writes, &[DeepAxis::Head]);
        // Coarse rate — groundwater equilibrates in millennia against a 2.5 Myr
        // epoch, so this samples the topography rather than relaxing the water.
        let head_c = head.schedule.cadence().expect("a stepping pass");
        assert!(head_c.period() > 1);
        assert!(
            head.schedule.fires(0),
            "epoch 0 fires for everyone (journal/0124)"
        );
        assert!(head.schedule.fires(head_c.period()));
        assert!(
            !head.schedule.seeds(),
            "the bare-surface pre-loop head seed is retired"
        );

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

    /// **The head field is pinned by the PAIR, not by one read.** The revision its
    /// four boundary-condition inputs all come from is `Forced`, and the pass is
    /// held in that window from **both** sides — `reads: Forced` after the writer
    /// that produced it, `reads_prev: Incised` before the writer that supersedes
    /// it. One without the other pins nothing: they are different resources to the
    /// graph. (The under-declaration this closed —
    /// `docs/design/pass-declaration-history.md` § 4.)
    #[test]
    fn the_head_field_is_pinned_into_the_terrain_revision_it_reads() {
        for cfg in [
            all_on(),
            DeepConfig {
                full_agents: false,
                ..all_on()
            },
            DeepConfig {
                tectonic_history: false,
                ..all_on()
            },
            DeepConfig {
                flow_record: false,
                ..all_on()
            },
        ] {
            let order = DeepSchedule::new(deep_passes(&cfg))
                .expect("valid roster")
                .ordered_ids();
            let at = |id: &str| order.iter().position(|x| *x == id).expect("scheduled");
            assert!(
                at("dc:deep/forcing") < at("dc:deep/head"),
                "head reads the FORCED terrain: {order:?}"
            );
            assert!(
                at("dc:deep/head") < at("dc:deep/transport"),
                "head must read the terrain before transport incises it: {order:?}"
            );
        }

        // Rename-proof on the `reads` side too: an id that would win the tie-break
        // against `dc:deep/forcing` still cannot be scheduled ahead of it.
        let mut passes = deep_passes(&all_on());
        let hostile = "dc:deep/aaa_head";
        assert!(hostile < "dc:deep/forcing", "the rename must be hostile");
        for p in &mut passes {
            if p.id == "dc:deep/head" {
                p.id = hostile;
            }
        }
        let order = DeepSchedule::new(passes)
            .expect("schedulable")
            .ordered_ids();
        let at = |id: &str| order.iter().position(|x| *x == id).expect("scheduled");
        assert!(at("dc:deep/forcing") < at(hostile), "{order:?}");
        assert!(at(hostile) < at("dc:deep/transport"), "{order:?}");
    }

    /// **Neutrality, proven rather than asserted.** The four terrain-revision
    /// declarations — `climate.reads_prev = [Forced]`, `transport.writes +=
    /// Incised`, `weather.reads += Incised`, and head's `Forced`/`Incised` pair —
    /// introduce only edges already implied by the existing transitive closure, so
    /// the schedule cannot move. Rebuild each roster with the **pre-slice**
    /// declarations and assert the order is identical: the direct check, rather
    /// than an appeal to a golden hash computed elsewhere.
    #[test]
    fn the_terrain_revision_declarations_are_schedule_neutral() {
        const PRE_TRANSPORT_WRITES: &[DeepAxis] = &[Energy, DeltaH];
        const PRE_WEATHER_READS: &[DeepAxis] = &[DeltaH, Exposed, Frosted];
        const PRE_HEAD_READS: &[DeepAxis] = &[Routed];
        const PRE_HEAD_LAG: &[DeepAxis] = &[Recorded];
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
            let after = DeepSchedule::new(deep_passes(&cfg))
                .expect("valid roster")
                .ordered_ids();
            let mut passes = deep_passes(&cfg);
            for p in &mut passes {
                match p.id {
                    "dc:deep/climate" => p.reads_prev = &[],
                    "dc:deep/transport" => p.writes = PRE_TRANSPORT_WRITES,
                    "dc:deep/weather" => p.reads = PRE_WEATHER_READS,
                    "dc:deep/head" => {
                        p.reads = PRE_HEAD_READS;
                        p.reads_prev = PRE_HEAD_LAG;
                    }
                    _ => {}
                }
            }
            let before = DeepSchedule::new(passes)
                .expect("the pre-slice roster was also valid")
                .ordered_ids();
            assert_eq!(
                before, after,
                "declaring the terrain revisions must not move the schedule"
            );
        }
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
        assert_eq!(w.schedule, Schedule::EVERY_EPOCH);
        assert!(w.schedule.fires(0) && w.schedule.fires(1));

        // **The declaration must state what the pass DOES.** `weather_epoch` reads
        // `grid.bio_weather`, which `biotic` overwrites in place each epoch — so it
        // observes THIS epoch's plane. BioMod is a real within-epoch read; nothing is
        // loop-carried here. (Reasoning in full at the `WINV_READS_*` const block.)
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
        // depends on is enforced by the graph and survives a rename of either pass.
        assert!(pos("dc:deep/weather_inventory") > pos("dc:deep/biotic"));

        // Off-flag order is exactly the production order (the pass added nothing).
        assert_eq!(
            DeepSchedule::new(deep_passes(&all_on()))
                .unwrap()
                .ordered_ids(),
            PRODUCTION_ORDER.to_vec()
        );
    }

    // ------------------------------------------------------------ RATE ------
    // The cadence axis (journal/0123). `cadence.rs` proves the arithmetic; these
    // prove the axis is wired to the roster and that the shipped schedule is the
    // empty table's answer.

    /// **The declared roster IS today's schedule, and an empty table is inert.**
    /// The first half is what makes the hash-identity claim checkable at all; the
    /// second is what makes `deep_passes` a safe delegation to `deep_passes_with`.
    #[test]
    fn an_empty_cadence_table_reproduces_the_declared_schedule() {
        let cfg = DeepConfig {
            weather_inventory: true,
            ..all_on()
        };
        let declared = deep_passes(&cfg);
        let empty = deep_passes_with(&cfg, &CadenceTable::empty());
        assert_eq!(declared.len(), empty.len());
        for (a, b) in declared.iter().zip(empty.iter()) {
            assert_eq!(a.id, b.id);
            assert_eq!(a.schedule, b.schedule, "{}", a.id);
        }
        // And the declared cadences are the pre-RATE constants: three coarse-rate
        // passes at their own periods, everything else every epoch, one turn.
        let at = |id: &str| declared.iter().find(|p| p.id == id).unwrap().schedule;
        assert_eq!(at("dc:deep/climate"), Schedule::every(cfg.remarch_interval));
        assert_eq!(
            at("dc:deep/geotherm"),
            Schedule::every(super::super::geotherm::GEOTHERM_PERIOD)
        );
        assert_eq!(
            at("dc:deep/head"),
            Schedule::every(super::super::head::HEAD_PERIOD)
        );
        for p in &declared {
            let c = p.schedule.cadence().expect("every shipped pass steps");
            assert_eq!(
                c.sub_turns(),
                1,
                "{} declares sub-turns; the shipped roster takes exactly one turn \
                 per firing and the goldens depend on it",
                p.id
            );
            if !["dc:deep/climate", "dc:deep/geotherm", "dc:deep/head"].contains(&p.id) {
                assert_eq!(p.schedule, Schedule::EVERY_EPOCH, "{}", p.id);
            }
        }
    }

    /// **The shipped roster declares no seeder, and that is a finding, not a
    /// default** (journal/0124). Three passes were seeded before the loop until
    /// 2026-07-29; the audit that came with the `Schedule` ruling found all three
    /// to be first-steps in disguise and deleted their pre-loop blocks. Asserted so
    /// that re-introducing a seed is a deliberate act with a test to update.
    #[test]
    fn no_shipped_pass_seeds_and_every_one_of_them_fires_at_epoch_zero() {
        for cfg in [
            all_on(),
            DeepConfig {
                weather_inventory: true,
                ..all_on()
            },
            DeepConfig::default(),
        ] {
            for p in deep_passes(&cfg) {
                assert!(
                    !p.schedule.seeds(),
                    "{} declares a pre-loop seed; the shipped roster has none",
                    p.id
                );
                assert!(
                    p.schedule.fires(0),
                    "{} does not fire at epoch 0 — the skip rule is deleted",
                    p.id
                );
            }
        }
    }

    /// **A world authors the rate; the engine derives nothing.** The user's sketch,
    /// as a data structure: weathering ×5 while tectonics ×1, and a climate
    /// re-march on a period the world picked rather than the one the pack declared.
    #[test]
    fn a_world_may_author_a_passs_cadence_over_the_declared_default() {
        let cfg = all_on();
        let table = CadenceTable::empty()
            .with("dc:deep/weather", Cadence::sub_turned(5))
            .with("dc:deep/climate", Cadence::every(7))
            .with("dc:deep/tectonics", Cadence::EVERY_EPOCH)
            // A pass this roster gated off: ignored, not rejected.
            .with("dc:deep/nonexistent", Cadence::every(3));
        let passes = deep_passes_with(&cfg, &table);
        let at = |id: &str| {
            passes
                .iter()
                .find(|p| p.id == id)
                .unwrap()
                .schedule
                .cadence()
                .expect("a stepping pass")
        };

        assert_eq!(at("dc:deep/weather").sub_turns(), 5);
        assert_eq!(at("dc:deep/weather").period(), 1);
        assert_eq!(at("dc:deep/weather").dt(), 0.2, "five turns, a fifth each");
        assert_eq!(at("dc:deep/climate").period(), 7);
        assert_eq!(at("dc:deep/tectonics"), Cadence::EVERY_EPOCH);
        assert_eq!(at("dc:deep/tectonics").dt(), 1.0);
        // Un-authored passes keep their declaration.
        assert_eq!(
            at("dc:deep/head"),
            Cadence::every(super::super::head::HEAD_PERIOD)
        );
        // The typo the roster cannot satisfy is findable, for a caller that wants
        // to be strict.
        let ids = passes.iter().map(|p| p.id).collect::<Vec<_>>();
        assert_eq!(table.unmatched(&ids), vec!["dc:deep/nonexistent"]);

        // Cadence is orthogonal to ORDER: re-rating a pass must not re-order the
        // roster. (It is the second axis of "order × rate × window" precisely
        // because the two compose without touching.)
        assert_eq!(
            DeepSchedule::new(deep_passes_with(&cfg, &table))
                .expect("still schedulable")
                .ordered_ids(),
            PRODUCTION_ORDER.to_vec()
        );
    }

    /// **`fires` reads the authored period, not the declared one** — otherwise the
    /// table would be a decoration on a schedule that ignores it.
    #[test]
    fn the_authored_period_decides_when_a_pass_fires() {
        let table = CadenceTable::empty().with("dc:deep/transport", Cadence::every(3));
        let passes = deep_passes_with(&all_on(), &table);
        let t = passes
            .iter()
            .find(|p| p.id == "dc:deep/transport")
            .expect("scheduled");
        // Declared EVERY_EPOCH — it fired at 0, 1, 2 before the table. Re-rated to
        // period 3 it fires at 0, 3, 6: the authored period spaces the firings and
        // **does not remove the one at epoch 0**. Under the deleted skip rule this
        // assertion read `!t.schedule.fires(0)` — a world that authored a period silently
        // lost its pass's first phase, which is the semantics nobody decided
        // (journal/0123's flag, journal/0124's fix).
        assert!(t.schedule.fires(0), "epoch 0 fires for everyone");
        assert!(!t.schedule.fires(1));
        assert!(!t.schedule.fires(2));
        assert!(t.schedule.fires(3));
        assert!(t.schedule.fires(6));
    }

    #[test]
    fn climate_is_a_coarse_rate_pass_that_fires_at_epoch_zero() {
        let passes = deep_passes(&all_on());
        let climate = passes.iter().find(|p| p.id == "dc:deep/climate").unwrap();
        let c = climate.schedule.cadence().expect("a stepping pass");
        assert!(c.period() > 1, "climate re-marches on a coarse cadence");
        // It marches on the START-of-epoch topography, and says so (journal/0107):
        // an anti-dependency against the first terrain revision of the epoch, which
        // transitively covers every later terrain writer.
        assert_eq!(climate.reads_prev, &[DeepAxis::Forced]);
        // It fires at 0 and on its multiples. The pre-loop march it used to be
        // seeded by was the same call on the same inputs, and is gone
        // (journal/0124).
        assert!(climate.schedule.fires(0));
        assert!(climate.schedule.fires(c.period()));
        assert!(!climate.schedule.fires(1));
        assert!(!climate.schedule.seeds());
        // the erosion sub-passes are rate-1: fire every epoch including 0.
        let transport = passes.iter().find(|p| p.id == "dc:deep/transport").unwrap();
        assert!(transport.schedule.fires(0) && transport.schedule.fires(1));
    }

    // -------------------------------------------------------- SCHEDULE ------
    // journal/0124. `schedule.rs` proves the arithmetic and
    // `tests/schedule_axis.rs` the whole-world clock; this proves the runner
    // actually honours the axis — that a `Seed` runs once, before the loop, at
    // `dt = 0`, and that a `SeedAndStep` does both.

    /// A roster the shipped one cannot provide: one pure seed, one seed-and-step,
    /// one ordinary per-epoch pass. Bodies are no-ops — every claim below is about
    /// the *schedule*, and [`DeepSchedule::plan`] is the decision the runner
    /// itself executes, so reading it is reading the real thing.
    fn synthetic_roster() -> DeepSchedule {
        fn nop(_ctx: &mut DeepStepCtx<'_>) {}
        DeepSchedule::new(vec![
            DeepPass {
                id: "dc:test/seed",
                reads: &[],
                writes: &[Climate],
                reads_prev: &[],
                schedule: Schedule::Seed,
                body: nop,
            },
            DeepPass {
                id: "dc:test/both",
                reads: &[Climate],
                writes: &[Forced],
                reads_prev: &[],
                schedule: Schedule::SeedAndStep(Cadence::every(4)),
                body: nop,
            },
            DeepPass {
                id: "dc:test/step",
                reads: &[Forced],
                writes: &[Routed],
                reads_prev: &[],
                schedule: Schedule::EVERY_EPOCH,
                body: nop,
            },
        ])
        .expect("valid synthetic roster")
    }

    /// **`Seed` is executed, not decorative.** The shipped roster declares no
    /// seeder, so without this the variant would be a type with no behaviour.
    #[test]
    fn the_runner_seeds_before_it_steps_and_a_seed_integrates_no_time() {
        let sched = synthetic_roster();

        let setup = sched.plan(None);
        assert_eq!(
            setup.iter().map(|t| t.id).collect::<Vec<_>>(),
            vec!["dc:test/seed", "dc:test/both"],
            "the setup epoch runs the seeders, in the loop's own topo order"
        );
        for t in &setup {
            assert_eq!(t.dt, 0.0, "{} seeded with a non-zero dt", t.id);
            assert_eq!(t.turns, 1, "a seed runs once; sub-turns are a step's axis");
        }
    }

    /// **The in-loop half: epoch 0 fires everyone, and a period spaces the rest.**
    #[test]
    fn the_loop_fires_every_pass_at_epoch_zero_and_a_pure_seed_never() {
        let sched = synthetic_roster();

        assert_eq!(
            sched.plan(Some(0)).iter().map(|t| t.id).collect::<Vec<_>>(),
            vec!["dc:test/both", "dc:test/step"],
            "epoch 0 fires the period-4 pass too; the pure seed never steps"
        );
        for e in [1u32, 2, 3] {
            assert_eq!(
                sched.plan(Some(e)).iter().map(|t| t.id).collect::<Vec<_>>(),
                vec!["dc:test/step"],
                "only the period-1 pass fires at epoch {e}"
            );
        }
        assert_eq!(
            sched.plan(Some(4)).iter().map(|t| t.id).collect::<Vec<_>>(),
            vec!["dc:test/both", "dc:test/step"]
        );
        // Over 8 epochs: the seeder once, the seed-and-step 1 + 2, the stepper 8.
        let count = |id: &str| {
            usize::from(sched.plan(None).iter().any(|t| t.id == id))
                + (0..8)
                    .filter(|&e| sched.plan(Some(e)).iter().any(|t| t.id == id))
                    .count()
        };
        assert_eq!(count("dc:test/seed"), 1);
        assert_eq!(count("dc:test/both"), 3);
        assert_eq!(count("dc:test/step"), 8);
    }
}
