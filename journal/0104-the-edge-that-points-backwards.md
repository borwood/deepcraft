# 0104 — The edge that points backwards

*2026-07-25. `crates/dc-worldgen/src/passgraph.rs`,
`crates/dc-worldgen/src/deeptime/runner.rs`. Background agent, isolated worktree.
User-ratified 2026-07-25 as option (a) of the ROADMAP Observed entry "A TIE-BREAK IS
DECIDING PHYSICS AGAIN".*

> blogworthy: **lens 3 (reflexions in a deepsim codebase)**, and a little of lens 1.
> A field that is declared, typed, documented and consumed by nothing is not a
> declaration — it is a comment with a type signature, and the compiler will help you
> keep it beautifully consistent while it means nothing at all. The fix is not a better
> comment. It is the second edge kind the graph was always missing.

## The declaration that wasn't

`DeepPass` has three read/write fields. Two of them are handed to the scheduler:

```rust
pub reads:  &'static [DeepAxis],
pub writes: &'static [DeepAxis],
pub reads_prev: &'static [DeepAxis],   // ← handed to nothing
```

`reads_prev` means *"I read LAST epoch's value of this axis"*. It is how the deep-time
runner expresses a loop-carried dependency: the biotic pass writes the vegetation
modifier planes at the end of an epoch, and the erosion passes that consume them
consume them *next* epoch. Declaring that as a normal `reads` would close a
within-epoch cycle — erosion → biotic (via the strata record) and biotic → erosion (via
the modifier) — which the runner correctly rejects. So the field exists, and the doc
comment on it explained, at some length, exactly why it was **deliberately not** handed
to `passgraph`.

And that was the whole mechanism. `reads_prev` appeared in `runner.rs` and in no other
file under `crates/`. `DeepSchedule::new` built its `Decl` from `reads` and `writes` and
dropped the third field on the floor.

## Why "not handed to the sort" quietly became "decided by the alphabet"

The deep-time planes are **overwritten in place** each epoch. `grid.strata`,
`grid.bio_weather`, `grid.bio_resist` — there is one copy, and this epoch's writer
clobbers it. Which means whether a lagged reader actually sees the previous epoch's
value depends on exactly one thing: **does it run before or after this epoch's writer?**

Nothing in the graph said. So it fell through to `passgraph`'s tie-break, which is
`ready.sort_by_key(|&i| decls[i].id)` — Kahn's algorithm parking every currently-ready
node until it wins a **lexicographic sort on the pass id**.

The concrete instance the audit found: `dc:field/head` declares
`reads_prev: [Recorded]`, and that claim is *true today*. `dc:deep/head` becomes ready
the moment `dc:deep/drainage` writes `Routed` — but "ready" is not "run". Kahn parks
every ready node in a pool and pops the alphabetically smallest, so what actually places
head at position 7, eight steps ahead of `dc:deep/deposition`, is that its **id wins the
sort** against everything else waiting.

Rename it `dc:deep/hydraulic_head` — or, as the test does, `dc:deep/zzz_head` — and it
loses that sort to every other pass in the roster, sits in the ready pool while
`deposition`, `eolian` and `wave` are popped ahead of it, and silently starts reading
*this* epoch's record. The groundwater field would begin equilibrating against strata
that had not been deposited when the drainage solve it also reads was computed. No test
would fail. No declaration would be wrong. The physics would just be different.

This was the **second** instance found in two consecutive `spine-audit` sweeps. The
first — `weather_inventory` declaring `reads_prev: [BioMod]` when it demonstrably read
*this* epoch's plane — was fixable by making the declaration honest: it was a real
within-epoch read, so declaring it as `reads` both told the truth and pinned the order.
The second one is not fixable that way, and the difference is the interesting part.

## Why declaring it as a normal read is the wrong fix

It reverses the ordering.

A `reads` edge is a **true dependency** (read-after-write, RAW): "I need the value this
schedule produces", so the writer is ordered *before* the reader. A `reads_prev` edge
wants the opposite: "I need the value from *before* this schedule ran", which — with an
in-place plane — is only true if the reader runs *before* the writer.

That is a **write-after-read hazard** (WAR), and the textbook fix is an
**anti-dependency edge**: reader → writer. Same graph, opposite direction.

Folding a lagged read into `reads` therefore does two wrong things at once. It orders
the reader after the writer, which is precisely the value it did not want. And in a loop
it closes the feedback into a within-epoch cycle, which the kernel rejects — so the
"fix" turns a working schedule into a hard error. There is a test for exactly this now
(`folding_a_lagged_read_into_reads_reverses_it_into_a_cycle`): the same two-pass roster
is schedulable when the lag is declared as a lag and a `GraphError::Cycle` when it is
folded in.

So the honest fix could not be a declaration change. It had to be a **second edge
kind**.

## The mechanism

`passgraph::Decl` grows a third field, and `schedule` grows about twelve lines:

```rust
// The anti-dependency (WAR) edge: a node declaring `reads_prev: [X]` wants the
// value X held *before* this schedule ran, and the plane is overwritten in
// place — so it must be ordered BEFORE every writer of X.
for (reader, p) in decls.iter().enumerate() {
    for &r in p.reads_prev {
        let Some(roles) = by_resource.get(&r) else { continue };
        for &w in roles.creators.iter().chain(roles.modifiers.iter()) {
            push_edge(&mut adj, reader, w);
        }
    }
}
```

Everything downstream is unchanged: same Kahn sort, same cycle rejection, same
ambiguous-writers check (which only gets *easier* to satisfy, since anti-dependency
edges add reachability). Three details were worth thinking about:

- **A node may lag-read what it also writes.** `push_edge` already drops self-edges, and
  the semantics are right for free: a modifier trivially observes the pre-write value.
- **A lagged read of an axis nobody writes is inert.** The value came from the pre-loop
  seed; there is nothing to order against. This matters because a pass's declaration is
  a property of the *pass*, not of the roster it happens to be scheduled in — with
  `head_field` off, `dc:deep/head` is absent and `Recorded`'s lagged readers simply have
  one fewer writer to outrank.
- **`reads` ∩ `reads_prev` on one node is now a named rejection**
  (`GraphError::ContradictoryLag`). Two edges in opposite directions: against another
  writer that is a 2-cycle and would be caught anyway, but against *itself* both edges
  collapse to a dropped self-edge and the contradiction would pass silently. Given that
  the entire slice exists because a declaration passed silently, that seemed like the
  wrong thing to leave to chance.

`pipeline.rs` shares the kernel and hands `reads_prev: &[]`. That is not a placeholder —
pregen is a one-shot DAG that builds state from nothing, so there is no "previous value"
for anything, and the anti-dependency edge kind is *structurally* empty at that tier.
Its `map_graph_error` meets `ContradictoryLag` with an `unreachable!` that says so,
rather than a silent catch-all, so a future pregen lag has to come and argue here first.

## The order did not move — and that was the point

The production roster's 17 passes schedule in exactly the same order after the change as
before:

```
climate · expose · tectonics · forcing · drainage · frost · geotherm · head ·
transport · flow_record · weather · diffuse · isostasy · deposition · eolian ·
wave · biotic
```

`order_reproduces_the_erosion_step_phase_order` and
`the_production_world_still_hashes_to_the_pre_slice_goldens` are both green, unmoved, by
name. Six lagged readers produce twelve new anti-dependency edges —
`expose`/`frost`/`head` each → the three writers of `Recorded`
(`deposition`, `eolian`, `wave`), and `weather`/`diffuse`/`eolian` each → `biotic` — and
every one of them points from a pass that was *already* ahead of its target. Which is
exactly what you want to find:
**the existing physics was correct, it was just unenforced.** The slice converts an
accident into a guarantee and changes nothing else.

(The argument that it *cannot* change anything else is small enough to state: no node
whose indegree went from zero to non-zero existed — every node that gained an in-edge
already had one — and for each such node the newly-added predecessor pops strictly
earlier than a predecessor it already had, so nothing's readiness step moves, so the
ready pool at every step is identical, so the pop order is identical. The test is still
the authority; but it is nice when the test and the reasoning agree.)

## The test that is the whole slice

Byte-identity proves the change was harmless. It does not prove the change did anything.
The test that does is a **hostile rename**:

`a_lagged_reader_stays_ahead_of_its_writer_under_a_hostile_rename` takes the production
roster, and for each of the six lagged readers, renames it to an id that is *guaranteed
to lose the alphabet fight* against every writer of the axis it lags on
(`dc:deep/head` → `dc:deep/zzz_head`, and so on — the test asserts the rename really is
hostile before proceeding, so it cannot pass vacuously). Then it schedules and asserts
the reader is still ordered first.

Under the old kernel every one of those six would fail. `dc:deep/zzz_head` would sort
behind `dc:deep/deposition` and start reading this epoch's record — the exact silent
physics change the audit described, now a red test instead of a paragraph.

Its kernel-level twin, `a_lagged_read_outranks_the_id_tie_break`, carries its own
negative control: the identical two-node roster with the lag *undeclared* schedules
`["a-writer", "z-reader"]`, and with it declared schedules `["z-reader", "a-writer"]`.
The first assertion is the pre-slice behaviour, written down so the next reader can see
what "the alphabet decided which epoch you read" actually looked like.

And `every_lagged_read_in_the_roster_is_ordered_before_its_writers` sweeps the whole
invariant across five configs (production, `weather_inventory` on, `head_field` off,
`full_agents` off, `tectonic_history` off) rather than one pass — so a *new* lagged read
added by a future slice is covered the day it is declared.

## The audit, while we were in there

All six `reads_prev` declarations in the roster were checked against their pass bodies:

| pass | lags on | body actually reads | verdict |
|---|---|---|---|
| `dc:deep/expose` | `Recorded` | `grid.strata` (`erosion.rs:859`) | true, now enforced |
| `dc:deep/frost` | `Recorded` | `grid.strata` (`erosion.rs:941`) | true, now enforced |
| `dc:deep/head` | `Recorded` | `grid.strata` (`head.rs:426`) | true, now enforced |
| `dc:deep/weather` | `BioMod` | `grid.bio_weather` (`erosion.rs:1178`) | true, now enforced |
| `dc:deep/diffuse` | `BioMod` | `grid.bio_resist` (`erosion.rs:1284`) | true, now enforced |
| `dc:deep/eolian` | `BioMod` | `grid.bio_resist` (`erosion.rs:1432`) | true, now enforced |

No further lie found — which is the good outcome, and also the one that makes the point:
they were *all* true, and *none* of them were true for a reason.

One genuine under-declaration, reported and deliberately not changed:
**`dc:deep/climate` declares `reads_prev: &[]`** while its comment says it reads the
start-of-epoch topography. That is a real lagged read of the terrain, undeclared. It is
not *wrong* in the dangerous sense, because the ordering it needs is already pinned by a
true forward edge — `dc:deep/forcing` reads `Climate`, and every terrain writer in the
roster is downstream of `forcing`, so climate is provably ahead of all of them by the
graph and not by the tie-break. Declaring it would be free (the edge is implied by the
existing transitive closure, so it provably cannot move the schedule) but would need
three cfg-selected slices for the three terrain-revision rosters, and this slice's
mandate was not to touch declarations. Filed as ROADMAP Owed.

**Not touched, and it is still open:** the second half of the Observed entry —
`dc:field/head` under-declaring its *live* read of the ground surface `R + H`. That one
can move a default-on pass's own output and the entry says to stop if it does; it wants
its own slice and its own flux-record comparison.

## What it looks like in game

Nothing. Deliberately — the world hashes identically, because the order the graph now
enforces is the order it already had.

What changes is the class of bug that can reach the world. Before this, "the groundwater
field reads the strata as they stood at the start of the epoch" was a sentence in a doc
comment, and a rename in a refactor could have made it false while every test stayed
green — and the symptom would have been a subtly different aquifer, showing up as
slightly wrong springs and seeps in some valley thirty hours of walking later, with
nothing in the diff to point at. That path is closed. A lagged read is now a thing the
scheduler *does*, and the way to break it is to be told you broke it.
