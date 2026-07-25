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

Production world, seed 1337, `Extent::Medium`, 545×545 = 297,025 cells, 8 chapters,
`p = 4`. Same pregen, two deep runs.

**The acceptance — SIMULTANEOUS divergence, WITHIN ONE EPOCH:**

| | MFD off | MFD on |
|---|---|---|
| `(cell, epoch)` pairs with ≥2 lateral out-faces | **0** | **7,548,646** |
| distinct `(cell, chapter)` pairs with ≥1 such epoch | **0** | **395,452** (16.64 % of all pairs) |
| most lateral out-faces in a single epoch | **0** | **8** |

The zero is not a small number, it is a **structural** zero — a D8 receiver is one
out-edge, so no world, no cadence and no threshold could have made it anything
else. Concurrent distributaries are now not merely representable but present.

**The control — TEMPORAL divergence (avulsion), WITHIN A CHAPTER**, which the
receiver tree already produced and which must survive rather than be replaced:

| | MFD off | MFD on |
|---|---|---|
| `(cell, chapter)` pairs, **all** face families | 337,554 | 418,911 |
| `(cell, chapter)` pairs, **lateral** faces only | 56,281 | 397,520 |

> **A caption here was wrong and is worth recording.** The probe printed *"the
> LATERAL row is what journal/0096 reported as 175,320"*. It is not: measured with
> MFD off, the lateral row is **56,281** and the all-faces row is **337,554**.
> Neither is 175,320, and the number is **not recoverable** from either, because
> two one-way changes happened underneath it — FLOW (a) added **vertical** faces
> (inflating the all-faces row with infiltration that is not a distributary), and
> 0096's count included **boundary** faces, which the lateral row excludes (a cell
> that routes inland early in a chapter and into the sea late in it *did* diverge
> and appears in neither row). The world has also moved twice since.
> **This was a published claim the gate could not check**, printed beside the number
> it contradicted — the exact failure mode CLAUDE.md names. The caption now prints
> its own measured values and explicitly forbids quoting them as 0096 continued.

**The exponent sweep, and a methodological finding.** `p` is swept on the same
production world:

| `p` | simul `(cell, epoch)` | simul `(cell, chapter)` | record entries | record MiB | deep s |
|---|---|---|---|---|---|
| 1.0 | 7,564,611 | 401,473 | 4,114,409 | 63.91 | 25.0 |
| 2.0 | 7,561,298 | 400,191 | 3,978,535 | 61.84 | 25.2 |
| **4.0** | **7,548,646** | **395,452** | **3,735,571** | **58.13** | **25.5** |
| 8.0 | 7,513,387 | 383,531 | 3,479,103 | 54.22 | 25.3 |
| off (`p → ∞`) | 0 | 0 | 2,897,736 | 45.35 | 21.2 |

Monotone in the right direction, as the gate test demands — but **only barely**:
0.7 % across `p ∈ [1, 8]`. That is a finding about the *instrument*, not about the
knob. The acceptance counter is a **binary predicate** ("did this cell use ≥2
faces"), and the 1 % floor guarantees a second receiver survives across almost the
whole range, so the count is nearly saturated. The quantities that actually track
`p` are **record entries** (−15 % from `p=1` to `p=8`) and **residency**. Anyone
tuning `p` should watch those, not the headline.

**Cost.** Deep run **21.18 s → 25.74 s, a difference of +4.56 s** (+21.5 %) on the
production world; gen time is not a constraint and nothing here is on a runtime
path. Record residency **45.35 → 58.13 MiB (+12.78)**; `DeepField` **156.16 →
169.00 MiB (+12.84)**. Record entries ×1.29. Load-carrying faces 494,296 →
1,364,127 (×2.76).

## What moved, and what a player will see

**I wrote the section below before the production numbers came back, as a
prediction. The measurement contradicted the important half of it, and the
prediction is kept above the correction rather than quietly edited, because the
gap between them is the most useful thing in this entry.**

### What actually moved (measured)

| | value |
|---|---|
| cells whose elevation moved by > 1 m | **1** (0.00 %) |
| mean \|Δ elevation\|, whole grid | 0.006 m |
| mean \|Δ elevation\|, **subaerial only** (44,264 cells) | **0.027 m** |
| mean surface / min / max | **identical to 2 d.p.** |
| land cells whose **argmax receiver** moved | **31,523 — 71.22 % of land** |
| mean relative \|Δ drainage area\| | **0.3799** |
| **peak drainage area** | **1,245 → 84 cells** |
| total suspended load on faces | 1,864.8 → 1,564.2 (−16 %) |

Read that table twice, because it says two opposite things at once.

**The routing changed almost completely.** Seven land cells in ten now send most
of their water somewhere different, and the average cell's catchment changed by
38 %. **The landscape did not change at all.** Mean elevation is identical to two
decimal places; exactly **one** cell in 297,025 moved by more than a metre.

Both are true, and the reason is the regime: incision on this world is small
against ~600 m of tectonic uplift, and only 44,264 of 297,025 cells are subaerial
at all. The erosion pass is nowhere near being the thing that shapes this terrain.
So MFD moved **where the water goes** — which is what it was for — and had almost
no purchase on **what the ground looks like**.

**So the goldens moved for a reason a player cannot see.** A 2.7 cm mean change
on land is enough to flip a surface voxel here and there and to change which
clastic unit lands in which cell, which is why every golden moved; it is nowhere
near enough to change a silhouette. The honest statement is:

> **This is an appearance change on paper and a null in the viewport.** It should
> be announced, and it should **not** consume a walk unless a tour-map first finds
> a station where something is visibly different. On these numbers I expect the
> tour-map to return a null, and a null honestly reported is the result.

### The one number that should worry a reader: peak drainage area 1,245 → 84

This is the classic MFD artefact and it deserves to be named rather than buried.
Dispersing at every cell compounds down a chain, so accumulated area never
concentrates: the largest catchment on the world fell **15×**. The single-receiver
world's biggest river drained 1,245 cells; the MFD world's biggest drains 84.

That is *why* the terrain barely moved, and the two facts are one fact. Stream
power is `Q^m·S^n`; MFD lowers **both** factors at every cell — `Q` because the
area is shared, `S` because the energy slope is the share-weighted mean rather
than the steepest. So **MFD at fixed coefficients is systematically less erosive
than D8**, which is also why total suspended load fell 16 % while the number of
load-carrying faces nearly tripled. Nothing leaked — `mass_is_conserved_with_mfd_on`
holds over the whole run — **less material was mobilised**, which is a different
statement.

The standard answer in the literature is that pure MFD is used on **hillslopes**
and something convergent is used in **channels**: `p` is made a function of
accumulated area or slope, or the solve switches to single-receiver above a
channel-initiation threshold. We have shipped **uniform `p` everywhere**, which is
the simplest correct thing and is honestly the wrong long-run shape. It is flagged
for ratification, not fixed here — and it interacts with `k_bedrock`/`k_transport`,
which were calibrated against D8 and are now effectively weaker.

### The prediction I wrote first, kept for the record

### The Earth mechanism, and the tier it is faithful at

Flow divergence on a real landscape has **two** distinct origins, and until today
this engine could only produce one of them:

- **Avulsion** — a channel abandons its bed and takes a new one. Sequential, not
  simultaneous: at any instant there is one channel; over a century there are
  several beds. This is what builds an alluvial fan's radial pattern and a braid
  plain's anastomosis, and the receiver tree already produced it, honestly, as an
  artefact of the terrain moving under the flow between epochs.
- **Distributary splitting** — the flow physically divides at a node and *both*
  branches run at once. This is what a delta is. The Mississippi's birdsfoot, the
  Okavango, the Ganges–Brahmaputra: several channels carrying water in the same
  season. A single-receiver solve cannot produce it at any resolution, at any
  cadence, with any threshold, because the primitive is one out-edge.

MFD is the standard, decades-old answer, and the tier it is faithful at should be
stated honestly: it is a **partition rule on a 460 m grid**, not a hydraulic
solve. It does not know about levees, bed aggradation at the bifurcation node, or
the discharge-ratio instability that makes real avulsions episodic rather than
smooth. What it gets right is the thing the primitive was foreclosing: **where the
slope field is nearly flat and multi-directional, the water goes several ways at
once, and the sediment goes with it.** That is enough to make a delta a delta
rather than a single channel that happens to wander.

### The chain from routing to what you walk on

Routing is upstream of erosion, so the change propagates through the whole
deep-time ritual, in this order:

1. **Drainage area** is now spread rather than concentrated. Downstream of every
   near-flat junction, the trunk gets less and the neighbours get more.
2. **Stream power** is `Q^m · S^n`, so a spread `Q` means **less incision per
   channel** on the flats and — because the residual has to go somewhere —
   incision that is *less* concentrated in a single thread.
3. **Transport capacity** spreads with it, so suspended load is delivered across a
   fan of cells rather than down one line. Deposition follows.
4. The **strata record** therefore receives its clastic units over a wider
   footprint, with more cells receiving *some* sediment and fewer receiving a
   lot.
5. **Vegetation** reads the drainage/moisture field, so the biotic layer's valley
   test sees a different set of cells.

### What a player sees, per axis

- **Landforms.** The headline. Where a river meets a flat — a fan head at a
  mountain front, the mouth of a valley, a coastal plain — the single incised
  thread becomes **several shallower ones**. Gorges and mountain valleys should
  look **unchanged**, and that is a prediction the exponent makes, not a hope:
  at `p = 4` a 2:1 sidewall-to-channel slope ratio is a 16:1 share ratio, so a
  gorge stays one thread. **If a walk finds gorges braiding, `p` is too low.**
- **Strata.** Wider, thinner clastic sheets in the depositional lowlands instead
  of narrower thicker ones; more cells with *a* fluvial unit in a given chapter.
  A road-cut on a fan should read as more interbedding, less single-channel fill.
- **Vegetation.** Follows moisture and valley position, so riparian bands should
  **widen and fray** at fan and delta positions rather than tracking one line.
- **Resources.** Placers and any transport-deposited concentration follow the
  load, so they spread with it — which is the honest physical answer (a real fan
  *does* scatter its heavy minerals) but it means a prospecting signature that
  was a line becomes a field.

**This is an appearance change and it is the user's to bless. It needs a walk**,
and the walk wants a tour-map first: the strongest fan/delta exemplar under MFD,
and a gorge as the control that must *not* have moved.

> **How that prediction scored.** The *direction* was right and the *magnitude* was
> wrong by orders. Routing did change on 71 % of land; drainage area did spread
> (−38 % mean relative, and the peak catchment collapsed 15×). But every visible
> consequence I listed — braided fan heads, wider riparian bands, thinner clastic
> sheets — was predicated on the erosion pass being strong enough to *express* the
> routing, and on this world it is not. I reasoned from the mechanism and never
> asked how much authority the mechanism had over this particular terrain. The
> table above is the answer, and it took one probe run to get.
>
> **The lesson generalises past this slice:** "routing is upstream of erosion, so
> the world will move" is a *sequencing* argument, and sequencing arguments say
> nothing about magnitude. It is the same shape as the "head unlocks MFD" error at
> the top of this entry — a claim about *order* smuggled in as a claim about
> *substance*. Twice in one slice.

## The gate, and what it cost

Full workspace gate, clean-built (`cargo clean -p dc-worldgen --release`) with the
`Compiling` / `Checking` lines verified to name **this worktree's** checkout:

- `cargo fmt --all --check` — clean, 0 diffs
- `cargo clippy --workspace --all-targets --release -- -D warnings` — **exit 0**
- `cargo test --workspace --release --no-fail-fast` — **exit 0, 78 binaries,
  759 passed, 0 failed**, zero `FAILED` / `panicked` / `error` lines anywhere in
  the log

Against the pre-slice baseline of **742 passed / 76 binaries**, the delta
reconciles exactly:

| added | tests | where |
|---|---|---|
| `deeptime::erosion::mfd_tests` | 6 | lib unit (131 → 137) |
| `tests/mfd_routing.rs` | 9 | new binary |
| `examples/mfd_probe.rs` `mod gate` | 2 | new binary (`test = true`) |
| | **+17** | **76 → 78 binaries, 742 → 759** |

**Added gate wall-clock: 22.7 s** — `mfd_routing` 13.16 s + `mfd_probe` gate
9.58 s; the six unit tests are on an 8-cell hand-built patch and are free. Both
gate suites run at `Extent::Small`, and the doc comments say why each invariant is
scale-free (a per-cell predicate; a set inclusion; an order-preserving
transformation).

**A build-hygiene note that cost an hour and is worth the next agent's time.**
Two agents share one `CARGO_TARGET_DIR`, and cargo gives the *same* artifact hash
to the same package in two different worktrees. A sibling rebuilt `dc-worldgen`
from a branch without MFD while my run was in flight, and the result was a **false
RED** with a *coherent-looking* signature: `providers_golden` reported
`0x176D…006A != 0x6F83…8C36` — the **computed** value was the old one and the
**constant** was the new one, i.e. the mismatch pointed *backwards*. That
direction is the tell. A real golden move computes the new value and finds the old
constant; a poisoned artifact computes the old value against your new constant.
The other symptom was the probe failing to compile against a `DeepConfig` with no
`mfd` field — in *my* file, at *my* line numbers.
The cheap defence turned out to be structural rather than procedural: **include a
test target that only exists on your branch** (`--test mfd_routing`). If it runs,
the artifacts are yours. A file mutex that a sibling can overwrite is not a mutex;
a target the sibling cannot possibly have is a proof.

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
