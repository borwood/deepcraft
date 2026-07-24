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
//! **loop-carried edges** (the biology↔erosion lag) the within-epoch graph must
//! not see as cycles.
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
use super::grid::{DeepConfig, DeepGrid, sea_level_at};
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
    /// **Lagged** (previous-epoch) reads — documented on the pass, and
    /// deliberately **NOT** handed to the topo-sort. A lagged read is a
    /// back-edge in the epoch loop; declaring it here (not in `reads`) is how the
    /// biology↔erosion feedback is expressed as a loop-carried edge instead of a
    /// within-epoch cycle the runner would (correctly) reject.
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

/// The biotic layer: reads this epoch's fresh post-erosion terrain + drainage,
/// deposits its organic record, and writes the modifiers the NEXT epoch's
/// erosion consumes.
fn biotic_pass(ctx: &mut DeepStepCtx<'_>) {
    if let Some(b) = ctx.biota.as_mut() {
        ctx.biotic_total += b.step(&mut ctx.grid, &ctx.erosion, ctx.epoch);
    }
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
pub fn deep_passes(cfg: &DeepConfig) -> Vec<DeepPass> {
    let mut passes = Vec::new();

    // Climate — the one coarse-rate pass already in the loop.
    passes.push(DeepPass {
        id: "dc:deep/climate",
        reads: &[],
        writes: &[Climate],
        // Reads the start-of-epoch topography (last epoch's final terrain) — a
        // lagged read, not handed to the topo-sort. `forcing` reads `Climate`,
        // which sequences the whole terrain-mutation chain *after* this pass, so
        // climate always samples the pre-forcing surface (byte-identical to the
        // old loop's top-of-body `climate::march`).
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
    /// seeded before the loop). Lagged reads (`reads_prev`) are deliberately not
    /// handed to the kernel.
    pub fn new(passes: Vec<DeepPass>) -> Result<Self, GraphError<DeepAxis>> {
        let decls: Vec<Decl<DeepAxis>> = passes
            .iter()
            .map(|p| Decl {
                id: p.id,
                reads: p.reads,
                writes: p.writes,
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

    const PRODUCTION_ORDER: [&str; 14] = [
        "dc:deep/climate",
        "dc:deep/expose",
        "dc:deep/tectonics",
        "dc:deep/forcing",
        "dc:deep/drainage",
        "dc:deep/frost",
        "dc:deep/transport",
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
