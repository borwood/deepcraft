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
//! creator→{modifiers,readers} and modifier→readers edges; topo-sort by Kahn's
//! algorithm with a **lexicographic-id tie-break** (never registration order);
//! and reject, with a named culprit, duplicate ids, multiple creators of one
//! resource, cycles, and writer pairs the declarations leave unordered.
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

/// One node's declaration *as the graph sees it*: an id and its within-schedule
/// reads/writes. **Lagged / cross-epoch reads are the caller's concern and are
/// deliberately NOT handed here** — they are back-edges the within-schedule
/// graph must never see, or every loop-carried feedback would read as a cycle.
pub struct Decl<'a, R> {
    pub id: &'a str,
    pub reads: &'a [R],
    pub writes: &'a [R],
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
    MultipleCreators { resource: R, a: String, b: String },
    AmbiguousWriters { resource: R, a: String, b: String },
    UnwrittenResource { resource: R, pass: String },
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

    // Edges: creator -> {modifiers, readers}; modifier -> readers.
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
        Decl { id, reads, writes }
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
