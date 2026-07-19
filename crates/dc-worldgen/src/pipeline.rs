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
//! The S7 stages (tectonics, climate, hydrology, history) are the first four
//! registered passes; their declared reads/writes force exactly the order
//! `Pregen::run` used to hand-maintain, so the refactor is output-preserving
//! by construction (and the S7 byte-identical tests prove it).
//!
//! **Write semantics.** For each resource: the pass that writes without
//! reading is its *creator* (at most one); passes that read *and* write are
//! *modifiers* (they run after the creator, and their mutual order must be
//! forced by some other resource edge — otherwise the build fails rather
//! than silently picking one); pure readers run after every writer.
//! Determinism note: the final order is a pure function of the declarations
//! (Kahn's algorithm with lexicographic-id tie-break), never of registration
//! order.

use std::collections::BTreeMap;

use crate::geology::StrataCtx;
use crate::pregen::{CellGrid, history};

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
    /// Settlement history: ledger, overlay, sites.
    History,
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
    pub history: Option<history::History>,
}

/// What a pass does when it runs (plain function pointers: deterministic,
/// no captured state, trivially registrable as data).
#[derive(Clone, Copy, Debug)]
pub enum PassBody {
    Pregen(fn(&mut PregenCtx)),
    Strata(fn(&mut StrataCtx)),
}

/// One registered pass: identity, phase, declared reads/writes, body.
#[derive(Clone, Copy, Debug)]
pub struct Pass {
    pub id: &'static str,
    pub phase: Phase,
    pub reads: &'static [Resource],
    pub writes: &'static [Resource],
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
}

/// A validated, ordered pass graph.
#[derive(Debug)]
pub struct Pipeline {
    passes: Vec<Pass>,
    /// Indices into `passes`, in execution order.
    order: Vec<usize>,
}

impl Pipeline {
    /// Validate and topo-sort. The order is a pure function of the
    /// declarations: Kahn's algorithm, ties broken by pass id.
    pub fn new(passes: Vec<Pass>) -> Result<Self, PipelineError> {
        for (i, p) in passes.iter().enumerate() {
            if passes[..i].iter().any(|q| q.id == p.id) {
                return Err(PipelineError::DuplicateId(p.id.to_string()));
            }
        }

        // Classify per resource: (creators, modifiers, pure readers).
        type Roles = (Vec<usize>, Vec<usize>, Vec<usize>);
        let mut by_resource: BTreeMap<Resource, Roles> = BTreeMap::new();
        for (i, p) in passes.iter().enumerate() {
            for &r in p.writes {
                let entry = by_resource.entry(r).or_default();
                if p.reads.contains(&r) {
                    entry.1.push(i);
                } else {
                    entry.0.push(i);
                }
            }
            for &r in p.reads {
                if !p.writes.contains(&r) {
                    by_resource.entry(r).or_default().2.push(i);
                }
            }
        }

        // Edges (adjacency): creator -> modifiers + readers; modifier -> readers.
        let n = passes.len();
        let mut adj = vec![Vec::<usize>::new(); n];
        let push_edge = |adj: &mut Vec<Vec<usize>>, from: usize, to: usize| {
            if from != to && !adj[from].contains(&to) {
                adj[from].push(to);
            }
        };
        for (&resource, (creators, modifiers, readers)) in &by_resource {
            if creators.len() > 1 {
                return Err(PipelineError::MultipleCreators {
                    resource,
                    a: passes[creators[0]].id.to_string(),
                    b: passes[creators[1]].id.to_string(),
                });
            }
            if creators.is_empty() {
                let culprit = modifiers.first().or(readers.first());
                if let Some(&i) = culprit {
                    return Err(PipelineError::UnwrittenResource {
                        resource,
                        pass: passes[i].id.to_string(),
                    });
                }
            }
            for &c in creators {
                if passes[c].phase == Phase::Collapse {
                    // A collapse-created resource must not feed pregen passes.
                    for &i in modifiers.iter().chain(readers.iter()) {
                        if passes[i].phase == Phase::Pregen {
                            return Err(PipelineError::PhaseViolation {
                                resource,
                                pass: passes[i].id.to_string(),
                            });
                        }
                    }
                }
                for &i in modifiers.iter().chain(readers.iter()) {
                    push_edge(&mut adj, c, i);
                }
            }
            for &m in modifiers {
                for &r in readers {
                    push_edge(&mut adj, m, r);
                }
            }
        }

        // Kahn with deterministic (id-lexicographic) tie-break.
        let mut indegree = vec![0usize; n];
        for out in &adj {
            for &t in out {
                indegree[t] += 1;
            }
        }
        let mut ready: Vec<usize> = (0..n).filter(|&i| indegree[i] == 0).collect();
        let mut order = Vec::with_capacity(n);
        while !ready.is_empty() {
            ready.sort_by_key(|&i| passes[i].id);
            let next = ready.remove(0);
            order.push(next);
            for &t in &adj[next] {
                indegree[t] -= 1;
                if indegree[t] == 0 {
                    ready.push(t);
                }
            }
        }
        if order.len() != n {
            let remaining = (0..n)
                .filter(|i| !order.contains(i))
                .map(|i| passes[i].id.to_string())
                .collect();
            return Err(PipelineError::Cycle(remaining));
        }

        // Ambiguity check: every writer pair of a resource must be connected
        // by a path, or their relative order is an accident of the sort.
        let reachable = transitive_closure(&adj);
        for (&resource, (creators, modifiers, _)) in &by_resource {
            let writers: Vec<usize> = creators.iter().chain(modifiers.iter()).copied().collect();
            for (ai, &a) in writers.iter().enumerate() {
                for &b in &writers[ai + 1..] {
                    if !reachable[a][b] && !reachable[b][a] {
                        return Err(PipelineError::AmbiguousWriters {
                            resource,
                            a: passes[a].id.to_string(),
                            b: passes[b].id.to_string(),
                        });
                    }
                }
            }
        }

        Ok(Self { passes, order })
    }

    /// The vanilla pass graph: the four S7 stages plus the v1 geology
    /// passes, ordered purely by their declarations.
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
}

fn transitive_closure(adj: &[Vec<usize>]) -> Vec<Vec<bool>> {
    let n = adj.len();
    let mut reach = vec![vec![false; n]; n];
    for start in 0..n {
        let mut stack: Vec<usize> = adj[start].clone();
        while let Some(v) = stack.pop() {
            if !reach[start][v] {
                reach[start][v] = true;
                stack.extend(adj[v].iter().copied());
            }
        }
    }
    reach
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

fn history_pass(ctx: &mut PregenCtx) {
    let grid = ctx.grid.as_ref().expect("tectonics ran (declared read)");
    ctx.history = Some(history::run(ctx.seed, grid));
}

/// The vanilla pass roster with honest read/write declarations. The four S7
/// stages' declarations force the exact legacy order; the geology passes
/// slot in behind them: igneous creates the strata record, clastic deposits
/// on top of it (modifier) and creates the alluvium state, the placer
/// reworks the alluvial body (modifier ordered after clastic via Alluvium).
pub fn vanilla_passes() -> Vec<Pass> {
    use Resource::*;
    vec![
        Pass {
            id: "dc:pass/tectonics",
            phase: Phase::Pregen,
            reads: &[],
            writes: &[Plates, Elevation, Provenance],
            body: PassBody::Pregen(tectonics_pass),
        },
        Pass {
            id: "dc:pass/climate",
            phase: Phase::Pregen,
            reads: &[Elevation],
            writes: &[Climate],
            body: PassBody::Pregen(climate_pass),
        },
        Pass {
            id: "dc:pass/hydrology",
            phase: Phase::Pregen,
            reads: &[Elevation, Climate],
            writes: &[Hydrology],
            body: PassBody::Pregen(hydrology_pass),
        },
        Pass {
            id: "dc:pass/history",
            phase: Phase::Pregen,
            reads: &[Elevation, Climate, Hydrology],
            writes: &[History],
            body: PassBody::Pregen(history_pass),
        },
        Pass {
            id: "dc:pass/igneous-emplacement",
            phase: Phase::Collapse,
            reads: &[Provenance, Elevation, Climate],
            writes: &[Strata],
            body: PassBody::Strata(crate::geology::igneous_pass),
        },
        Pass {
            id: "dc:pass/clastic-deposition",
            phase: Phase::Collapse,
            reads: &[Climate, Hydrology, Elevation, Strata],
            writes: &[Strata, Alluvium],
            body: PassBody::Strata(crate::geology::clastic_pass),
        },
        Pass {
            id: "dc:pass/placer",
            phase: Phase::Collapse,
            reads: &[Alluvium, Hydrology, Strata],
            writes: &[Strata],
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
            body: PassBody::Pregen(noop),
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
                "dc:pass/hydrology",
                "dc:pass/history",
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
        use Resource::History;
        let err = Pipeline::new(vec![pass("t:a", &[History], &[])]).unwrap_err();
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
