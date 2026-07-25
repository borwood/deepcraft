//! The pass-graph kernel: topo-sort + validation over declared reads/writes,
//! **shared by every pass-runner in the crate** so the mechanism lives once
//! (spines A-4: build on the runner that exists, never a second one beside it).
//!
//! Both runners speak this vocabulary:
//! - the pregen [`crate::pipeline`] — a DAG that builds world state *from
//!   nothing* (a resource must have a creator; `require_creator = true`);
//! - the deep-time [`crate::deeptime::runner`] — a **loop** over state that
//!   persists across epochs (every resource is already present from the pre-loop
//!   seed or the previous turn, so there is no creator requirement;
//!   `require_creator = false`).
//!
//! What is identical between them — and therefore extracted here — is the graph
//! math: classify each resource's writers into a single *creator* (write without
//! read) and *modifiers* (read-and-write) plus pure *readers*; build
//! creator→{modifiers,readers} and modifier→readers edges **plus the lagged
//! reader→writer anti-dependency edges** (see below); topo-sort by Kahn's
//! algorithm with a **lexicographic-id tie-break** (never registration order);
//! and reject, with a named culprit, duplicate ids, multiple creators of one
//! resource, cycles, and writer pairs the declarations leave unordered.
//!
//! ## The two edge kinds
//! - **`reads` — a true dependency (RAW).** "I need the value this schedule
//!   produces", so every writer of the resource is ordered **before** the reader.
//! - **`reads_prev` — an anti-dependency (WAR).** "I need the value from *before*
//!   this schedule ran" — the previous epoch's, in a loop. Because the plane is
//!   **overwritten in place**, that is only true if the reader runs **before** the
//!   writer clobbers it, so the edge points the other way: **reader → writer**.
//!
//! The direction is the whole point, and it is why a lagged read cannot simply be
//! folded into `reads`: folding it would order the reader *after* the writer —
//! the exact opposite — and, in a loop, close the feedback into a within-epoch
//! cycle the kernel would (correctly) reject. Before this edge existed
//! (journal/0104) `reads_prev` was consumed by nothing, so which epoch a lagged
//! reader actually saw was decided by the id-lexicographic tie-break: renaming a
//! pass silently changed the physics.
//!
//! The **crossing constraint** (north-star): a declaration handed here is *plain
//! data + opaque ids* — an `id: &str` and `&[R]` slices of a `Copy + Ord`
//! resource token. No closures, no behavior, cross this seam; the runner that
//! owns the bodies keeps them on its own side.

use std::collections::BTreeMap;

/// A resource axis token: orderable + copyable so it can key the classification
/// map and be named in a rejection. Blanket-implemented — a caller just needs a
/// `#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]` enum.
pub trait Axis: Copy + Ord + std::fmt::Debug {}
impl<T: Copy + Ord + std::fmt::Debug> Axis for T {}

/// One node's declaration *as the graph sees it*: an id, its within-schedule
/// reads/writes, and its **lagged** reads.
pub struct Decl<'a, R> {
    pub id: &'a str,
    /// True (RAW) dependencies: every writer of these is ordered **before** this
    /// node.
    pub reads: &'a [R],
    pub writes: &'a [R],
    /// **Lagged / cross-epoch reads** — "the value from before this schedule ran".
    /// An **anti-dependency** (WAR): this node is ordered **before** every writer
    /// of these resources, so it provably observes the pre-existing value rather
    /// than whatever this schedule's writer leaves behind. Empty for a pure DAG
    /// caller (nothing precedes a one-shot build), which is why it is a plain
    /// empty slice rather than an option.
    pub reads_prev: &'a [R],
}

/// Per-resource role classification (node indices into the declaration slice).
#[derive(Default, Debug)]
pub struct Roles {
    /// Writes the resource without reading it (at most one per resource).
    pub creators: Vec<usize>,
    /// Reads *and* writes the resource.
    pub modifiers: Vec<usize>,
    /// Reads without writing.
    pub readers: Vec<usize>,
}

/// A graph rejection, each variant naming its culprit(s).
#[derive(Debug, PartialEq)]
pub enum GraphError<R> {
    DuplicateId(String),
    MultipleCreators {
        resource: R,
        a: String,
        b: String,
    },
    AmbiguousWriters {
        resource: R,
        a: String,
        b: String,
    },
    UnwrittenResource {
        resource: R,
        pass: String,
    },
    /// One node declares the same resource as both a live `reads` and a lagged
    /// `reads_prev` — "order me after the writer" and "order me before the
    /// writer" at once. Against another writer that is a 2-cycle; against
    /// *itself* it would collapse to a dropped self-edge and pass silently, so it
    /// is rejected up front by name.
    ContradictoryLag {
        resource: R,
        pass: String,
    },
    Cycle(Vec<String>),
}

/// A validated schedule: the execution order (indices) and the per-resource role
/// classification (so a caller can layer extra policy — the pregen phase rule —
/// on top of the generic graph verdict).
#[derive(Debug)]
pub struct Scheduled<R> {
    pub order: Vec<usize>,
    pub roles: BTreeMap<R, Roles>,
}

/// Validate and topo-sort. The order is a pure function of the declarations
/// (Kahn, id-lexicographic tie-break), never of the order `decls` arrived in.
pub fn schedule<R: Axis>(
    decls: &[Decl<R>],
    require_creator: bool,
) -> Result<Scheduled<R>, GraphError<R>> {
    // Duplicate ids.
    for (i, p) in decls.iter().enumerate() {
        if decls[..i].iter().any(|q| q.id == p.id) {
            return Err(GraphError::DuplicateId(p.id.to_string()));
        }
    }

    // A resource cannot be both a live and a lagged read of the same node — the
    // two edge kinds point in opposite directions.
    for p in decls {
        if let Some(&r) = p.reads_prev.iter().find(|r| p.reads.contains(r)) {
            return Err(GraphError::ContradictoryLag {
                resource: r,
                pass: p.id.to_string(),
            });
        }
    }

    // Classify each resource's touchers into creators / modifiers / readers.
    let mut by_resource: BTreeMap<R, Roles> = BTreeMap::new();
    for (i, p) in decls.iter().enumerate() {
        for &r in p.writes {
            let entry = by_resource.entry(r).or_default();
            if p.reads.contains(&r) {
                entry.modifiers.push(i);
            } else {
                entry.creators.push(i);
            }
        }
        for &r in p.reads {
            if !p.writes.contains(&r) {
                by_resource.entry(r).or_default().readers.push(i);
            }
        }
    }

    // Edges: creator -> {modifiers, readers}; modifier -> readers; and the
    // anti-dependency lagged-reader -> {creator, modifiers}.
    let n = decls.len();
    let mut adj = vec![Vec::<usize>::new(); n];
    let push_edge = |adj: &mut Vec<Vec<usize>>, from: usize, to: usize| {
        if from != to && !adj[from].contains(&to) {
            adj[from].push(to);
        }
    };
    for (&resource, roles) in &by_resource {
        if roles.creators.len() > 1 {
            return Err(GraphError::MultipleCreators {
                resource,
                a: decls[roles.creators[0]].id.to_string(),
                b: decls[roles.creators[1]].id.to_string(),
            });
        }
        if roles.creators.is_empty()
            && require_creator
            && let Some(&i) = roles.modifiers.first().or_else(|| roles.readers.first())
        {
            return Err(GraphError::UnwrittenResource {
                resource,
                pass: decls[i].id.to_string(),
            });
        }
        for &c in &roles.creators {
            for &i in roles.modifiers.iter().chain(roles.readers.iter()) {
                push_edge(&mut adj, c, i);
            }
        }
        for &m in &roles.modifiers {
            for &r in &roles.readers {
                push_edge(&mut adj, m, r);
            }
        }
    }

    // The **anti-dependency (WAR) edge**: a node declaring `reads_prev: [X]` wants
    // the value X held *before* this schedule ran, and the plane is overwritten in
    // place — so it must be ordered BEFORE every writer of X. Reader → writer,
    // the reverse direction of a `reads` edge, which is precisely why a lagged
    // read cannot be expressed by folding it into `reads`.
    //
    // Its own writes are irrelevant here (a node that reads-prev and writes the
    // same resource is its own predecessor, which `push_edge` drops as a self-edge
    // — a modifier trivially sees the pre-write value). Nothing is looked up that
    // the roster does not contain: a lagged read of a resource nobody writes adds
    // no edges, exactly as a live read of a seeded resource adds none in loop mode.
    for (reader, p) in decls.iter().enumerate() {
        for &r in p.reads_prev {
            let Some(roles) = by_resource.get(&r) else {
                continue;
            };
            for &w in roles.creators.iter().chain(roles.modifiers.iter()) {
                push_edge(&mut adj, reader, w);
            }
        }
    }

    // Kahn topo-sort, deterministic id-lexicographic tie-break.
    let mut indegree = vec![0usize; n];
    for out in &adj {
        for &t in out {
            indegree[t] += 1;
        }
    }
    let mut ready: Vec<usize> = (0..n).filter(|&i| indegree[i] == 0).collect();
    let mut order = Vec::with_capacity(n);
    while !ready.is_empty() {
        ready.sort_by_key(|&i| decls[i].id);
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
            .map(|i| decls[i].id.to_string())
            .collect();
        return Err(GraphError::Cycle(remaining));
    }

    // Ambiguity: every writer pair of a resource must be connected by a path, or
    // their relative order is an accident of the sort.
    let reachable = transitive_closure(&adj);
    for (&resource, roles) in &by_resource {
        let writers: Vec<usize> = roles
            .creators
            .iter()
            .chain(roles.modifiers.iter())
            .copied()
            .collect();
        for (ai, &a) in writers.iter().enumerate() {
            for &b in &writers[ai + 1..] {
                if !reachable[a][b] && !reachable[b][a] {
                    return Err(GraphError::AmbiguousWriters {
                        resource,
                        a: decls[a].id.to_string(),
                        b: decls[b].id.to_string(),
                    });
                }
            }
        }
    }

    Ok(Scheduled {
        order,
        roles: by_resource,
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
    enum R {
        A,
        B,
        C,
    }

    fn decl<'a>(id: &'a str, reads: &'a [R], writes: &'a [R]) -> Decl<'a, R> {
        Decl {
            id,
            reads,
            writes,
            reads_prev: &[],
        }
    }

    fn lagged<'a>(id: &'a str, reads_prev: &'a [R], writes: &'a [R]) -> Decl<'a, R> {
        Decl {
            id,
            reads: &[],
            writes,
            reads_prev,
        }
    }

    fn ids<'a>(s: &Scheduled<R>, decls: &'a [Decl<'a, R>]) -> Vec<&'a str> {
        s.order.iter().map(|&i| decls[i].id).collect()
    }

    #[test]
    fn creator_before_reader() {
        let s = schedule(&[decl("z", &[R::A], &[]), decl("a", &[], &[R::A])], true).unwrap();
        // "a" creates A, "z" reads it — creator first, regardless of decl order.
        assert_eq!(
            s.order.iter().map(|&i| ["z", "a"][i]).collect::<Vec<_>>(),
            vec!["a", "z"]
        );
    }

    #[test]
    fn cycle_is_rejected() {
        let err = schedule(
            &[decl("a", &[R::A], &[R::B]), decl("b", &[R::B], &[R::A])],
            false,
        )
        .unwrap_err();
        assert!(matches!(err, GraphError::Cycle(_)), "{err:?}");
    }

    #[test]
    fn multiple_creators_rejected() {
        let err = schedule(&[decl("a", &[], &[R::A]), decl("b", &[], &[R::A])], false).unwrap_err();
        assert!(
            matches!(err, GraphError::MultipleCreators { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn require_creator_is_the_loop_vs_dag_switch() {
        // A resource that is only read: a DAG rejects it (nothing creates it), a
        // loop accepts it (it is seeded before the loop).
        let decls = [decl("a", &[R::C], &[])];
        assert!(matches!(
            schedule(&decls, true).unwrap_err(),
            GraphError::UnwrittenResource { .. }
        ));
        assert!(schedule(&decls, false).is_ok());
    }

    /// **The rename-proof test** (journal/0104). A lagged read is an
    /// *anti-dependency*: the reader must be ordered before the writer, or the
    /// in-place overwrite means it reads this schedule's value instead of the
    /// previous one. The roster here is built so the id-lexicographic tie-break
    /// would decide it the WRONG way — both nodes start ready, and "a-writer"
    /// sorts first — so if `reads_prev` were still consumed by nothing, the
    /// reader would run second and silently see the fresh value.
    #[test]
    fn a_lagged_read_outranks_the_id_tie_break() {
        // Negative control: the identical roster with the lag *not* declared. This
        // is the pre-slice behaviour, and it is wrong — the writer goes first
        // purely because "a-writer" < "z-reader".
        let blind = [
            decl("z-reader", &[], &[R::B]),
            decl("a-writer", &[], &[R::A]),
        ];
        assert_eq!(
            ids(&schedule(&blind, false).unwrap(), &blind),
            vec!["a-writer", "z-reader"],
            "without the edge the alphabet decides which epoch's A the reader sees"
        );

        // Declared: the anti-dependency edge outranks the tie-break.
        let lagged_roster = [
            lagged("z-reader", &[R::A], &[R::B]),
            decl("a-writer", &[], &[R::A]),
        ];
        assert_eq!(
            ids(&schedule(&lagged_roster, false).unwrap(), &lagged_roster),
            vec!["z-reader", "a-writer"],
        );
    }

    /// Folding a lagged read into `reads` is the wrong fix, and the kernel proves
    /// it: the same pair, with the lag declared live, is a 2-cycle. That is the
    /// feedback loop the anti-dependency edge exists to express instead.
    #[test]
    fn folding_a_lagged_read_into_reads_reverses_it_into_a_cycle() {
        let ok = [
            lagged("reader", &[R::A], &[R::B]),
            decl("writer", &[R::B], &[R::A]),
        ];
        assert!(schedule(&ok, false).is_ok());
        let folded = [
            decl("reader", &[R::A], &[R::B]),
            decl("writer", &[R::B], &[R::A]),
        ];
        assert!(matches!(
            schedule(&folded, false).unwrap_err(),
            GraphError::Cycle(_)
        ));
    }

    /// A node may lag-read what it also writes — it is trivially its own
    /// predecessor, and the self-edge is dropped rather than read as a cycle. It
    /// still outranks the *other* writers.
    #[test]
    fn a_lagged_read_of_ones_own_write_is_not_a_cycle() {
        let decls = [
            Decl {
                id: "z-modifier",
                reads: &[],
                writes: &[R::A],
                reads_prev: &[R::A],
            },
            decl("a-other", &[R::A], &[R::A]),
        ];
        let s = schedule(&decls, false).unwrap();
        assert_eq!(ids(&s, &decls), vec!["z-modifier", "a-other"]);
    }

    /// Same resource, both live and lagged, on one node: two edges in opposite
    /// directions. Against another writer that is a cycle; against itself it would
    /// vanish as a self-edge, so it is named up front instead.
    #[test]
    fn reads_and_reads_prev_of_the_same_resource_is_rejected() {
        let err = schedule(
            &[Decl {
                id: "confused",
                reads: &[R::A],
                writes: &[R::A],
                reads_prev: &[R::A],
            }],
            false,
        )
        .unwrap_err();
        assert!(
            matches!(err, GraphError::ContradictoryLag { resource: R::A, .. }),
            "{err:?}"
        );
    }

    /// A lagged read of a resource this roster never writes adds no edges — the
    /// value came from the pre-loop seed, and there is nothing to be ordered
    /// against. (A pass's declaration is a property of the pass, not of the roster
    /// it happens to be scheduled in.)
    #[test]
    fn a_lagged_read_of_an_unwritten_resource_is_inert() {
        let decls = [lagged("a", &[R::C], &[R::A]), decl("z", &[R::A], &[])];
        let s = schedule(&decls, false).unwrap();
        assert_eq!(ids(&s, &decls), vec!["a", "z"]);
    }

    #[test]
    fn ambiguous_writers_rejected() {
        let err = schedule(
            &[
                decl("create", &[], &[R::A]),
                decl("mod-a", &[R::A], &[R::A]),
                decl("mod-b", &[R::A], &[R::A]),
            ],
            true,
        )
        .unwrap_err();
        assert!(
            matches!(err, GraphError::AmbiguousWriters { .. }),
            "{err:?}"
        );
    }
}
