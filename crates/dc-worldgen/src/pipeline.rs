//! The pass graph: processes-as-passes (geology backbone,
//! docs/design/geology.md). Every deep-time process declares its phase
//! (pregen epoch vs lazy-collapse contributor), what it READS, and what it
//! WRITES over a closed vocabulary of core axes ([`Resource`] — core-axes-only
//! for v1; plugin-published axes are a later contract). The pipeline
//! topo-sorts passes from those declarations and rejects, at build time and
//! with a named culprit: cycles, two *creators* of one resource, and writer
//! pairs whose order the declarations leave ambiguous. The
//! geology→soil→ecology coupling-order problem becomes a graph problem
//! instead of a hand-maintained list.
//!
//! The S7 stages (tectonics, climate, hydrology) are the first three
//! registered passes; their declared reads/writes force exactly the order
//! `Pregen::run` used to hand-maintain, so the refactor is output-preserving
//! by construction (and the S7 byte-identical tests prove it). *There was a
//! fourth, `dc:pass/history`, removed 2026-07-28 (journal/0121); it was the
//! only writer of a `Resource::History` axis, which went with it.*
//!
//! **Write semantics.** For each resource: the pass that writes without
//! reading is its *creator* (at most one); passes that read *and* write are
//! *modifiers* (they run after the creator, and their mutual order must be
//! forced by some other resource edge — otherwise the build fails rather
//! than silently picking one); pure readers run after every writer.
//! Determinism note: the final order is a pure function of the declarations
//! (Kahn's algorithm with lexicographic-id tie-break), never of registration
//! order.

use dc_core::materials::geology::GeologySet;

use crate::deeptime::{DeepField, DeepOverrides};
use crate::geology::StrataCtx;
use crate::passgraph::{self, Decl, GraphError};
use crate::pregen::CellGrid;

/// When a pass runs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    /// Once, at world creation, over the coarse cell grid.
    Pregen,
    /// Per chunk-column, inside the lazy collapse (bounded lookahead:
    /// strata passes see one column's context, never the world).
    Collapse,
}

/// The core context axes / fields passes declare against (v1: closed set).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Resource {
    /// Voronoi plates + velocities.
    Plates,
    /// Coarse cell elevation.
    Elevation,
    /// Tectonic provenance per cell.
    Provenance,
    /// Latitude / temperature / precipitation fields.
    Climate,
    /// Filled elevations, flow graph, discharge, rivers, lakes.
    Hydrology,
    /// The deep-time eroded surface (the A-tier final elevation field). Created
    /// by the deep-time pass; the collapse elevation lattice reads it.
    DeepElevation,
    /// The deep-time per-cell strata record (tagged deposition log). Created by
    /// the deep-time pass; the collapse depositional passes read it as the
    /// at-deposition formation-context source (geology.md § formation context).
    DeepStrata,
    /// The per-column ordered deposition log.
    Strata,
    /// Alluvial-fan state (flow energy, the graded coarse body) the placer
    /// pass reworks.
    Alluvium,
}

/// Mutable state the pregen-phase passes build up, in declaration-forced
/// order. Slots start empty; each creator pass fills its own.
pub struct PregenCtx {
    pub seed: u64,
    /// Grid edge in cells.
    pub w: i32,
    pub grid: Option<CellGrid>,
    /// The deep-time field, filled by the deep-time pass (creator of
    /// [`Resource::DeepElevation`] + [`Resource::DeepStrata`]).
    pub deep: Option<DeepField>,
    /// Gen-time overrides the deep-time pass applies on top of the production
    /// config (`full_agents` / `tectonic_history` / amplitude flipped on at
    /// world creation). `Default` (all-inherit) reproduces production exactly.
    pub deep_overrides: DeepOverrides,
}

/// What a pass does when it runs (plain function pointers: deterministic,
/// no captured state, trivially registrable as data).
#[derive(Clone, Copy, Debug)]
pub enum PassBody {
    Pregen(fn(&mut PregenCtx)),
    Strata(fn(&mut StrataCtx)),
}

/// One registered pass: identity, phase, declared reads/writes, the content
/// classes it selects members from, and body.
#[derive(Clone, Copy, Debug)]
pub struct Pass {
    pub id: &'static str,
    pub phase: Phase,
    pub reads: &'static [Resource],
    pub writes: &'static [Resource],
    /// Content classes this pass selects members from (geology backbone). A
    /// world refuses to build if any of these has zero members — same
    /// named-culprit philosophy as the graph rejections (class-satisfiability
    /// enforcement, geology.md § unfilled slots).
    pub selects: &'static [&'static str],
    pub body: PassBody,
}

/// Build-time pass-graph rejections. Every variant names its culprits.
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum PipelineError {
    #[error("duplicate pass id `{0}`")]
    DuplicateId(String),
    #[error("passes `{a}` and `{b}` both create `{resource:?}` (write without reading)")]
    MultipleCreators {
        resource: Resource,
        a: String,
        b: String,
    },
    #[error(
        "writers `{a}` and `{b}` of `{resource:?}` have no declared ordering \
         (no read/write path connects them)"
    )]
    AmbiguousWriters {
        resource: Resource,
        a: String,
        b: String,
    },
    #[error("`{pass}` reads `{resource:?}`, which no pass writes")]
    UnwrittenResource { resource: Resource, pass: String },
    #[error(
        "pregen-phase pass `{pass}` reads `{resource:?}`, which only exists \
         at collapse time"
    )]
    PhaseViolation { resource: Resource, pass: String },
    #[error("pass graph has a cycle involving: {0:?}")]
    Cycle(Vec<String>),
    #[error("pass `{pass}` selects from class `{class}`, which has no members")]
    UnsatisfiableClass { pass: String, class: String },
}

/// A validated, ordered pass graph.
#[derive(Debug)]
pub struct Pipeline {
    passes: Vec<Pass>,
    /// Indices into `passes`, in execution order.
    order: Vec<usize>,
}

impl Pipeline {
    /// Validate and topo-sort. The graph math (classification, edges, Kahn's
    /// algorithm with an id-lexicographic tie-break, cycle + ambiguity
    /// rejection) is the shared [`passgraph`] kernel; the pregen tier layers one
    /// extra policy on the verdict — the **phase rule** (a collapse-created
    /// resource must not feed a pregen pass), which is meaningless at the
    /// deep-time loop tier that shares the kernel. Pregen builds state from
    /// nothing, so `require_creator = true`.
    pub fn new(passes: Vec<Pass>) -> Result<Self, PipelineError> {
        let decls: Vec<Decl<Resource>> = passes
            .iter()
            .map(|p| Decl {
                id: p.id,
                reads: p.reads,
                writes: p.writes,
                // Pregen is a one-shot DAG that builds state from nothing: there
                // is no "previous value" for anything, so the anti-dependency edge
                // kind is structurally empty here. The kernel is shared; only the
                // loop tier has lagged reads to declare.
                reads_prev: &[],
            })
            .collect();
        let scheduled = passgraph::schedule(&decls, true).map_err(map_graph_error)?;

        // Phase rule (pregen-specific): a collapse-created resource must not feed
        // a pregen-phase pass.
        for (&resource, roles) in &scheduled.roles {
            let collapse_created = roles
                .creators
                .iter()
                .any(|&c| passes[c].phase == Phase::Collapse);
            if collapse_created {
                for &i in roles.modifiers.iter().chain(roles.readers.iter()) {
                    if passes[i].phase == Phase::Pregen {
                        return Err(PipelineError::PhaseViolation {
                            resource,
                            pass: passes[i].id.to_string(),
                        });
                    }
                }
            }
        }

        Ok(Self {
            passes,
            order: scheduled.order,
        })
    }

    /// The vanilla pass graph: the three S7 stages plus the deep-time pass and
    /// the v1 geology passes, ordered purely by their declarations.
    pub fn vanilla() -> Result<Self, PipelineError> {
        Self::new(vanilla_passes())
    }

    /// Pass ids in execution order (for tests and diagnostics).
    pub fn ordered_ids(&self) -> Vec<&'static str> {
        self.order.iter().map(|&i| self.passes[i].id).collect()
    }

    /// Run every pregen-phase pass, in order.
    pub fn run_pregen(&self, ctx: &mut PregenCtx) {
        for &i in &self.order {
            if let (Phase::Pregen, PassBody::Pregen(f)) =
                (self.passes[i].phase, self.passes[i].body)
            {
                f(ctx);
            }
        }
    }

    /// Run every collapse-phase (strata) pass on one column's context,
    /// in order.
    pub fn run_strata(&self, ctx: &mut StrataCtx) {
        for &i in &self.order {
            if let (Phase::Collapse, PassBody::Strata(f)) =
                (self.passes[i].phase, self.passes[i].body)
            {
                f(ctx);
            }
        }
    }

    /// World-build-time class-satisfiability check (geology.md § unfilled
    /// slots, layer 2): a world must not build if any class a registered pass
    /// selects from has zero members — a silent skip would make world content a
    /// function of installed-pack coincidence. Rejects with the named culprit
    /// (pass + class), the same philosophy as [`PipelineError::Cycle`] and the
    /// ambiguous-writer rejections. Registration order cannot change the
    /// verdict: it is a pure function of the declared `selects` and the set's
    /// member counts.
    pub fn check_class_satisfiability(&self, geology: &GeologySet) -> Result<(), PipelineError> {
        for p in &self.passes {
            for &class in p.selects {
                let satisfied = geology
                    .class(class)
                    .is_some_and(|c| !c.members().is_empty());
                if !satisfied {
                    return Err(PipelineError::UnsatisfiableClass {
                        pass: p.id.to_string(),
                        class: class.to_string(),
                    });
                }
            }
        }
        Ok(())
    }
}

/// Map the generic kernel's rejection onto the pregen tier's named error. The
/// phase rule has no kernel analogue, so `GraphError` carries no such variant.
fn map_graph_error(e: GraphError<Resource>) -> PipelineError {
    match e {
        GraphError::DuplicateId(s) => PipelineError::DuplicateId(s),
        GraphError::MultipleCreators { resource, a, b } => {
            PipelineError::MultipleCreators { resource, a, b }
        }
        GraphError::AmbiguousWriters { resource, a, b } => {
            PipelineError::AmbiguousWriters { resource, a, b }
        }
        GraphError::UnwrittenResource { resource, pass } => {
            PipelineError::UnwrittenResource { resource, pass }
        }
        GraphError::Cycle(v) => PipelineError::Cycle(v),
        // Structurally impossible at this tier: `Pipeline::new` hands the kernel
        // `reads_prev: &[]` for every pass, and the pregen tier has no vocabulary
        // for a lagged read (a one-shot DAG has no "previous value"). Loud rather
        // than a silent catch-all, so a future pregen lag has to come here first.
        GraphError::ContradictoryLag { resource, pass } => unreachable!(
            "pregen declares no lagged reads, yet the kernel reported one on \
             {pass} for {resource:?}"
        ),
    }
}

// ---------------------------------------------------- the vanilla passes --

fn tectonics_pass(ctx: &mut PregenCtx) {
    ctx.grid = Some(crate::pregen::tectonics::build(ctx.seed, ctx.w));
}

fn climate_pass(ctx: &mut PregenCtx) {
    crate::pregen::climate::apply(ctx.grid.as_mut().expect("tectonics ran (declared read)"));
}

fn hydrology_pass(ctx: &mut PregenCtx) {
    crate::pregen::hydrology::apply(ctx.grid.as_mut().expect("tectonics ran (declared read)"));
}

/// The always-on deep-time A tier (3e-1): run the two-plane erosion sim over
/// the coarse grid and keep its eroded surface + strata record. Reads
/// Elevation/Provenance/Climate (bilinear-resampled to the deep grid), creates
/// DeepElevation + DeepStrata. This is the "generating world history…" ritual.
fn deep_time_pass(ctx: &mut PregenCtx) {
    let grid = ctx.grid.as_ref().expect("tectonics ran (declared read)");
    ctx.deep = Some(crate::deeptime::build_field_with(
        grid,
        ctx.seed,
        &ctx.deep_overrides,
    ));
}

/// The vanilla pass roster with honest read/write declarations. The three S7
/// stages' declarations force the exact legacy order; the geology passes
/// slot in behind them: igneous creates the strata record, clastic deposits
/// on top of it (modifier) and creates the alluvium state, the placer
/// reworks the alluvial body (modifier ordered after clastic via Alluvium).
pub fn vanilla_passes() -> Vec<Pass> {
    use dc_core::materials::geology::{
        CLASS_ACCESSORY_MAFIC, CLASS_CLASTIC_COARSE, CLASS_CLASTIC_FINE, CLASS_IGNEOUS_EXTRUSIVE,
        CLASS_IGNEOUS_INTRUSIVE, CLASS_ORE_PLACER, CLASS_ORGANIC_CHARCOAL, CLASS_ORGANIC_COAL,
        CLASS_ORGANIC_PEAT, CLASS_ORGANIC_SOIL,
    };

    use Resource::*;
    vec![
        Pass {
            id: "dc:pass/tectonics",
            phase: Phase::Pregen,
            reads: &[],
            writes: &[Plates, Elevation, Provenance],
            selects: &[],
            body: PassBody::Pregen(tectonics_pass),
        },
        Pass {
            id: "dc:pass/climate",
            phase: Phase::Pregen,
            reads: &[Elevation],
            writes: &[Climate],
            selects: &[],
            body: PassBody::Pregen(climate_pass),
        },
        Pass {
            id: "dc:pass/hydrology",
            phase: Phase::Pregen,
            reads: &[Elevation, Climate],
            writes: &[Hydrology],
            selects: &[],
            body: PassBody::Pregen(hydrology_pass),
        },
        Pass {
            id: "dc:pass/deep-time",
            phase: Phase::Pregen,
            reads: &[Elevation, Provenance, Climate],
            writes: &[DeepElevation, DeepStrata],
            selects: &[],
            body: PassBody::Pregen(deep_time_pass),
        },
        Pass {
            id: "dc:pass/igneous-emplacement",
            phase: Phase::Collapse,
            reads: &[Provenance, Elevation, Climate],
            writes: &[Strata],
            // Selects the two igneous classes and the accessory it emplaces as
            // a pore partial — all must ship a member or the world refuses.
            selects: &[
                CLASS_IGNEOUS_INTRUSIVE,
                CLASS_IGNEOUS_EXTRUSIVE,
                CLASS_ACCESSORY_MAFIC,
            ],
            body: PassBody::Strata(crate::geology::igneous_pass),
        },
        Pass {
            id: "dc:pass/clastic-deposition",
            phase: Phase::Collapse,
            reads: &[Climate, Hydrology, Elevation, Strata, DeepStrata],
            writes: &[Strata, Alluvium],
            // The deep-time history this pass lays down carries the recorder's
            // biotic facies, which `geology::deep_class` routes to the organic
            // classes — so the pass genuinely selects from them and the
            // class-satisfiability check must hold them too (a world with
            // biology on and no coal member must refuse to build, by name).
            selects: &[
                CLASS_CLASTIC_COARSE,
                CLASS_CLASTIC_FINE,
                CLASS_ORGANIC_SOIL,
                CLASS_ORGANIC_PEAT,
                CLASS_ORGANIC_COAL,
                CLASS_ORGANIC_CHARCOAL,
            ],
            body: PassBody::Strata(crate::geology::clastic_pass),
        },
        Pass {
            id: "dc:pass/placer",
            phase: Phase::Collapse,
            reads: &[Alluvium, Hydrology, Strata],
            writes: &[Strata],
            selects: &[CLASS_ORE_PLACER],
            body: PassBody::Strata(crate::geology::placer_pass),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noop(_: &mut PregenCtx) {}

    fn pass(id: &'static str, reads: &'static [Resource], writes: &'static [Resource]) -> Pass {
        Pass {
            id,
            phase: Phase::Pregen,
            reads,
            writes,
            selects: &[],
            body: PassBody::Pregen(noop),
        }
    }

    #[test]
    fn vanilla_geology_set_satisfies_every_selecting_pass() {
        let p = Pipeline::vanilla().expect("vanilla graph is valid");
        p.check_class_satisfiability(&dc_core::materials::geology::vanilla())
            .expect("vanilla content satisfies the vanilla passes");
    }

    #[test]
    fn an_empty_selected_class_is_rejected_with_the_culprits() {
        // A geology set that declares the classes but registers no members: the
        // igneous pass selects from an empty class and the build must refuse,
        // naming the pass and the class.
        let mut b = dc_core::materials::geology::GeologySet::builder();
        for c in dc_core::materials::geology::v1_classes() {
            b.declare_class(c).unwrap();
        }
        let empty = b.build();
        let p = Pipeline::vanilla().expect("vanilla graph is valid");
        match p.check_class_satisfiability(&empty).unwrap_err() {
            PipelineError::UnsatisfiableClass { pass, class } => {
                assert!(pass.starts_with("dc:pass/"), "named a pass: {pass}");
                assert!(class.starts_with("dc:"), "named a class: {class}");
            }
            other => panic!("expected UnsatisfiableClass, got {other}"),
        }
    }

    #[test]
    fn vanilla_order_is_the_legacy_order_then_geology() {
        let p = Pipeline::vanilla().expect("vanilla graph is valid");
        assert_eq!(
            p.ordered_ids(),
            vec![
                "dc:pass/tectonics",
                "dc:pass/climate",
                // deep-time depends only on Elevation/Provenance/Climate, so it
                // is ready right after climate and its id sorts ahead of the
                // hydrology/igneous ties.
                "dc:pass/deep-time",
                "dc:pass/hydrology",
                "dc:pass/igneous-emplacement",
                "dc:pass/clastic-deposition",
                "dc:pass/placer",
            ]
        );
    }

    #[test]
    fn order_ignores_registration_order() {
        let mut passes = vanilla_passes();
        passes.reverse();
        let p = Pipeline::new(passes).expect("still valid");
        assert_eq!(p.ordered_ids(), Pipeline::vanilla().unwrap().ordered_ids());
    }

    #[test]
    fn cycles_are_rejected_with_names() {
        use Resource::{Climate, Hydrology};
        let err = Pipeline::new(vec![
            pass("t:a", &[Climate], &[Hydrology]),
            pass("t:b", &[Hydrology], &[Climate]),
        ])
        .unwrap_err();
        // Both passes modify nothing (each creates what the other reads):
        // the graph is a 2-cycle.
        match err {
            PipelineError::Cycle(names) => {
                assert!(names.contains(&"t:a".to_string()) && names.contains(&"t:b".to_string()))
            }
            other => panic!("expected cycle, got {other}"),
        }
    }

    #[test]
    fn double_creation_is_rejected() {
        use Resource::Climate;
        let err = Pipeline::new(vec![
            pass("t:a", &[], &[Climate]),
            pass("t:b", &[], &[Climate]),
        ])
        .unwrap_err();
        assert!(
            matches!(err, PipelineError::MultipleCreators { .. }),
            "{err}"
        );
    }

    #[test]
    fn unordered_modifiers_are_rejected() {
        use Resource::{Climate, Strata};
        // Two modifiers of Strata with no path between them.
        let err = Pipeline::new(vec![
            pass("t:create", &[], &[Strata]),
            pass("t:mod-a", &[Strata], &[Strata]),
            pass("t:mod-b", &[Strata], &[Strata]),
            pass("t:climate", &[], &[Climate]),
        ])
        .unwrap_err();
        assert!(
            matches!(err, PipelineError::AmbiguousWriters { .. }),
            "{err}"
        );
    }

    #[test]
    fn reads_of_unwritten_resources_are_rejected() {
        use Resource::Hydrology;
        let err = Pipeline::new(vec![pass("t:a", &[Hydrology], &[])]).unwrap_err();
        assert!(
            matches!(err, PipelineError::UnwrittenResource { .. }),
            "{err}"
        );
    }

    #[test]
    fn duplicate_ids_are_rejected() {
        let err = Pipeline::new(vec![pass("t:a", &[], &[]), pass("t:a", &[], &[])]).unwrap_err();
        assert_eq!(err, PipelineError::DuplicateId("t:a".into()));
    }
}
