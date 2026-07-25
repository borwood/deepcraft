# 0109 — The tree was never what licensed the traversal

*2026-07-25 · FLOW continuation (b), the MFD solve*

> blogworthy: **lens 2 (procgen against the backdrop of priors)** and **lens 3
> (reflexions in a deepsim codebase)**. The interesting thing here is not that we
> implemented Holmgren 1994 — everybody has. It is that the change we braced for
> (a DAG breaks the traversal) turned out to be a change we did not have to make,
> and the reason why exposed a claim four documents had been repeating to each
> other for two days.

## What was wrong

`flow.md` § 2.6 had already done the hard thinking, back when slice 1 landed the
face-flux record. It drew a line that the whole arc turns on:

- **Temporal divergence** — a cell's flow leaves by the north face early in a
  chapter and by the east face late in it. That is **avulsion**, it is the honest
  physical origin of braid plains and alluvial fans, and the record had 175,320
  instances of it on the shipped world the day it was born.
- **Simultaneous divergence** — two channels flowing *at once*, out of the same
  cell, in the same epoch. That is a **delta**, and it was **structurally
  impossible**: within one epoch the drainage solve handed back exactly one
  receiver per cell, so the cell had exactly one out-edge, and no setting of any
  knob could produce a second.

§ 2.6 also drew the line on the *fix*, and this is the part it would have been
easy to get wrong: the fix is **not** a record change. A record is only allowed to
describe water the erosion pass **actually moved**. Write divergence into the
archive that the solve never performed and the flux record and the mass budget
start telling different stories, which is precisely the drift § 3 exists to
prevent — and it would be the worst kind of green test, one that passes for a
reason unrelated to the claim.

So: a solve change.

## The claim that was not true

§ 2.6 finished its paragraph like this:

> **MFD is a SOLVE change and belongs with the potential/head field**
> (continuation (a)): a head field partitions flux across several receivers
> naturally, where steepest-descent cannot.

That sentence, or a paraphrase of it, was in four places: flow.md § 2.6, flow.md
§ 9's open-question list, the `head.rs` module docs ("a multi-flow-direction
partition is what head *unlocks*"), and the `DeepField::head` row of spines § 3.
It had survived several rewrites of the surrounding text, because it reads like a
**sequencing** fact — the kind of statement nobody re-derives — when it is
actually a **physical** one.

It is false. MFD needs *a potential*; it does not need *that* potential.

For free-phase water, head is `z_bed + depth`, and **depth is zero on dry
ground**. So the driving potential of overland and channel flow is the **free
water surface** — which is exactly what the priority-flood `filled` array already
is, and has been since long before this arc opened: bare ground wherever the land
drains, and a flat spill-level water surface inside every depression. The
partition that shipped descends `filled`. It reads no new plane, declares no new
edge, and `dc:field/head` is not in its call path at all.

And the stronger half, which is what makes this a correction rather than a
shortcut: **using `dc:field/head` here would have been actively wrong.** That
plane is the **bound** regime's potential, and it is deliberately built to cross
surface drainage divides — the module pins the behaviour with a test named
`bound_head_crosses_a_surface_drainage_divide`, because the Great Artesian Basin
and karst piracy are real and a field that could not express them would foreclose
them. § 2.4 says in the same breath that the "never crosses a drainage divide"
rule *binds the free/surface regime only*. Partition surface discharge on the
water-table potential and every river crosses its own watershed.

The document contained both halves. The sequencing note read only one of them.
That is corrections **#54**, and the sentence has been struck at all four sites
rather than merely noted — a correction that lives only in `corrections.md` is a
correction the next author does not meet.

What *was* true, and is worth keeping: continuation (a) was correctly sequenced
before (b) — but because the head field is what made the record's **vertical**
faces honest (journal/0098), so (b) could change the *lateral* solve without the
record's other half still being a stub. The dependency was on the record's
completeness, not on the numerics.

**Bound MFD — Darcy flux partitioned across faces on `dc:field/head` — is real,
unbuilt, and now named as continuation (c)'s, beside the free/bound edge.**

## The partition

Holmgren (1994) with Quinn (1991)'s contour width:

```text
  w_k  ∝  (Δh_k / d_k)^p · L_k
```

Three pieces, each earning its place:

- `Δh_k` is the drop on the **filled free-surface potential**, not on bedrock
  elevation. § 2.4's requirement, satisfied by the plane the solve already had.
- `d_k` is the **true flow-path length** — `1` for a cardinal neighbour, `√2` for
  a diagonal. The old single-receiver rule picks the steepest **drop** and can get
  away with ignoring this; a slope-weighted partition cannot. Skip it and every
  diagonal is over-weighted by `√2`, and the whole drainage net acquires a
  systematic X-shaped bias. There is a unit test that pins the difference: a
  4 m diagonal drop (slope 2.83) must lose to a 3 m cardinal one (slope 3.0),
  and `route_cell` — the D8 rule — is asserted in the same test to disagree.
- `L_k` is Quinn's **contour width**: the length of the cell boundary the flow
  actually crosses, `0.5Δ` cardinal and `0.354Δ` diagonal, normalised to `1` and
  `1/√2`. It is the width of the gate, not the steepness of the drop, and it is
  why a diagonal receives less than a cardinal at *equal* slope.

`p` is the **convergence exponent**, and it is the one knob. It acts on the
**ratio** of slopes, so its physical meaning is *how strongly does this landscape
concentrate flow*:

- `p = 1` is Quinn's maximally dispersive form — a sheet-flow limit.
- `p → ∞` is single-receiver D8, **exactly**. The old solve is a *limit* of the
  new one, not a deleted alternative, which is what makes "how much does MFD
  change the world" a continuous question with a `p`-sweep for an answer.
- The default is **4.0**, in Holmgren's calibrated 4–6 band.

What a reader should expect `p = 4` to do to the landscape, stated as a
falsifiable prediction rather than a citation: on **steep** ground a 2:1 slope
ratio becomes a 16:1 share ratio, so a gorge — whose sidewalls are nowhere near
its channel's slope — stays a single thread. On the **low-relief** surfaces where
distributaries physically live (alluvial fans, braid plains, delta tops) the
competing neighbours are within a few percent of one another, `4` barely
separates them, and the flow genuinely splits. Collapse onto one receiver is a
**limit, not a threshold**: at `p = 16` two neighbours within a few percent still
share. That is worth saying out loud because the intuitive reading of "a large
exponent picks the steepest" is wrong in exactly the regime the slice is for.

One thing in the partition is **not** physics: a **1 % representational floor**.
A share below one percent is dropped and the survivors renormalised. Without it
every land cell records a trickle into every downslope neighbour and the archive
pays megabytes to store noise no consumer can distinguish from zero. The steepest
share is at least `1/8` before the floor, so a survivor always exists and the
floor can never turn a cell into a sink.

## The crux: what a DAG broke, and what it did not

MFD turns the flow graph from a **tree** into a **DAG**, and three things in the
erosion pass were written against a tree.

### 1. The traversal order — the one we braced for, and the one that did not move

Drainage accumulation and stream-power transport are strictly serial: a receiver
must see every upstream contribution before it is processed. Today that is done
by walking the priority-flood pop order in reverse, and the obvious reading is
that the *receiver tree* is what makes that legal.

It is not. `self.order` is the priority-flood pop order, which is strictly
ascending in `(filled, index)` — each cell is pushed exactly once, with its final
`filled` value, and popped in heap order. Reversed, it is strictly descending.
And **every routed edge descends `filled`**, single-receiver or multi, because
`partition_cell` weights only neighbours with `drop > 0`. So every out-edge points
to a cell strictly later in the reversed scan, and a cell is processed only after
all of its contributors — which is a topological order of the **DAG** for exactly
the reason it was one of the tree.

The tree was a *convenient* traversal. What actually licensed it was the
**potential ordering**. MFD needs no new topological sort; it needs the
observation that the old one was never about the tree. There is a test that pins
this directly rather than trusting the argument
(`every_routed_edge_descends_the_free_surface_potential`): if it ever fails, the
flow graph has a cycle and the traversal is no longer topological.

### 2. Mass down a DAG — the one that is silent when it breaks

The single-receiver chain conserved mass trivially: `qs_out` had exactly one
destination, so `qs[rc] += qs_out` moved the whole of it. Under MFD it has
several, and the budget survives on **one** condition — the shares must sum to
the whole with **no residue**.

They do not, naively. Normalised `f64` weights sum to `1 ± 1 ulp`, so
`Σ (w_k · q) ≠ q`. Per hop that is invisible. Down a thousand-hop chain it
compounds, and — worse — it is **unattributable afterwards**, because `qs[j]` is a
sum over every contributor with no unique factorisation. A mass leak here does not
crash, it just makes the continents slightly wrong.

The fix is that **the last weighted direction takes the residual**:

```text
  share_d = w_d · q          for every weighted d except the last
  share_last = q − Σ(earlier shares)
```

so the split is exact **by construction** rather than exact-to-an-ulp per hop.
The same three lines appear in both `accumulate_area` (splitting drainage area)
and `transport` (splitting suspended load), and in both the value written to the
record's per-face plane is *the same `share` variable* that was added to the
neighbour — not a re-derivation from the weights. That is the § 3 discipline made
structural: there is one arithmetic, so the flux record and the mass budget
**cannot** drift apart.

### 3. The incision clamp — the one that needed a decision, not a derivation

Bedrock incision is clamped so a cell can never be cut below its receiver. The
clamp is not aesthetic: without it a runaway knickpoint digs a hole its own outlet
cannot drain.

With several receivers there are three candidate generalisations, and they are
genuinely different rules:

- clamp at the **highest** receiver — arbitrarily stricter than today, and it
  would suppress incision on exactly the divergent cells the slice is about;
- clamp at the **share-weighted mean** receiver — smooth, and wrong: the cell can
  end up below its lowest outlet and become a pit anyway;
- clamp at the **lowest** receiver — the cell may be cut down to, but not below,
  the surface of its lowest outlet.

The third is the one that preserves what the clamp is *for*. The guarantee is "the
cell can still drain", and a cell with several outlets still drains as long as it
stays above the lowest of them. And it has the property that makes it the right
generalisation rather than a new rule: in the single-receiver limit the D8
receiver **is** the lowest neighbour, so it reduces to today's clamp **exactly**,
not approximately.

### 3b. The energy slope, which is not a smoothing choice

`exchange_cell` needs one slope for the stream-power law, and the cell now has
several. The share-weighted mean `Σ w_k S_k` is not a convenience: stream power is
`Q·S`, so the total power released by a split discharge is
`Σ Q_k S_k = Q · Σ w_k S_k`. The weighted mean is the slope that keeps the cell's
energy budget equal to the sum of the budgets of the flows leaving it. Any other
choice silently creates or destroys erosive work.

Everything above the split — entrainment, incision, deposition — is untouched, and
that is deliberate: `exchange_cell` is *one* function serving both the D8 and the
MFD chain. Two copies of a mass budget is the exact drift § 3 forbids.

### How it was proved, not asserted

Four tests, chosen so that each would fail for a different reason:

- `the_partition_leaves_no_residue` — for every cell, the per-face discharge plane
  sums to the cell's own accumulated drainage area (relative bound, because the
  planes are `f32` and areas span six orders of magnitude). This is the *local*
  statement: no cell loses water at its own junction.
- `mass_is_conserved_with_mfd_on` — the whole-world falsifier,
  `Δ(ΣR + ΣH) = uplift + biotic`, over a full 200-epoch run. This is the *global*
  statement, and it is the one that catches a leak the local test cannot: a share
  written to the record but never added to the neighbour would pass the residue
  test and fail this one.
- `every_routed_edge_descends_the_free_surface_potential` — the traversal
  licence, checked on the live final epoch rather than the chapter-aggregated
  archive (which sums 25 epochs of a *moving* terrain and structurally cannot
  answer the question).
- `no_interior_cell_is_cut_below_all_of_its_neighbours` — the clamp, read off the
  final terrain as an absence of pits.

Plus the identity pair, which is what keeps the whole thing honest:
`mfd_off_leaves_the_single_receiver_solve_untouched_and_on_moves_the_world`
asserts *both* halves — off is byte-identical to pre-slice `main`, and on is
different. A test that asserted only the first half would pass against a flag that
does nothing.

## The number

<!-- NUMBERS -->

## What moved, and what a player will see

<!-- WORLD -->

## `recv` after MFD

`recv` is on a retirement path already (continuation (e)), and MFD changes what it
*is* rather than merely superseding it.

Before MFD, `recv` was **a routing** — the solve genuinely sent the cell's whole
discharge there — that the flux record replaced with a richer one. After MFD it is
the **argmax share**: a projection of the partition, computed only because two
consumers still want one arrow per cell (the biotic layer's valley test and the
`DeepField` drainage export).

That is not meaningless. The argmax of a partition is a well-defined quantity and
it still answers "which way does most of the water go". But it fails
ARCHITECTURE.md's test in the failing direction — *if those two consumers vanished
tomorrow, would this plane exist in this shape?* No. So it has stopped being a
superseded authority and become **a summary wearing an authority's clothes**, which
is A-1, and its spines row has been upgraded accordingly. It stays pinned as a
shadow by two agreement tests, no new consumer may read it, and continuation (e)
should now **delete** it rather than supersede it.
