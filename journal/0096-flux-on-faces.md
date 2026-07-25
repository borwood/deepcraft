# 0096 — flux on faces, and the divergence a tree cannot count

*2026-07-25. FLOW slice 1: the recording half. Replacing the **output
representation** of the drainage solve — receiver tree → per-chapter flux on 3D
faces — while changing not one voxel of the world. Along the way: the divergence
turned out to come from **time**, not from a better solver; the atom's stratum
slot turned out to be absent more often than present, which vindicated deriving
it; and the design doc turned out to be silent on the one rule the whole
seamlessness claim rests on.*

> blogworthy (lenses: deepsim-reflexions; procgen-against-priors; earth-respect):
> *the primitive was the bug.* A day earlier the corpus concluded that
> `flow_to: Option<u32>` cannot represent the Mississippi delta — not "models it
> poorly," **cannot**. This is what it cost to fix that, and the surprise is that
> the fix needed no new numerics at all: keep the solve, keep every epoch, and
> divergence falls out of the calendar.

## What the slice had to prove

journal/0095 ended with a primitive-level indictment. `recv: Vec<i32>` is one
out-edge per cell — D8-with-one-receiver **is** a spanning tree by construction.
A tree represents convergence. It cannot represent divergence *at all*: no
distributaries, no braiding, no anabranch, no alluvial fan, no delta. And it was
exported as *"the **last** routing"*: we ran the process two hundred times and
kept the final frame.

So the acceptance test wrote itself, and it is a good one because it cannot be
gamed by a green unit test (A-3): **count the divergences on a production
world.** A receiver tree's count is identically zero, forever, by construction.
Any number above zero is proof that the representation — not the plumbing —
changed.

## The thing I expected to be hard, and wasn't

I went in assuming divergence would require touching the solve: an MFD
(multiple-flow-direction) partition, spreading each cell's discharge across all
downslope neighbours by slope weight. That is the textbook answer, and it was
the wrong one here for two reasons. It is a **different solve** — the brief and
flow.md § 7 both say the priority-flood/route/accumulate numerics stay — and,
worse, the record would then describe water the erosion pass never moved. A
record that disagrees with the process it records is a summary wearing an
authority's clothes.

The honest source was sitting in the loop schedule the whole time. A chapter is
**25 epochs** (200 iterations / 8 chapters), and the terrain moves under the
flow every one of them: uplift lifts, incision cuts, transport deposits, the
priority-flood re-fills. A cell's steepest-descent receiver **switches**.
Accumulating each epoch's discharge onto the *face it crossed* and totalling per
chapter therefore records, with no invention whatsoever:

> during chapter K, this much water left this cell eastward **and** this much
> left it southward.

Which is *avulsion* — the actual physical origin of braid plains, alluvial fans
and distributary networks, read at the time resolution the record keeps. A
receiver tree cannot say that sentence at any resolution. A face record says it
for free, and the "per chapter, not just the final epoch" requirement stops
being a storage chore and becomes **the mechanism that produces the divergence**.
The deepest defect and the headline capability turn out to be the same fix.

## The representation

`deeptime/flux.rs`. The atom (flow.md § 1.3) is 16 bytes:

```rust
pub struct FluxEntry {
    pub magnitude: f32,   // M — discharge across the face, summed over the chapter
    pub load: f32,        // L — suspended load that crossed with it
    pub fluid: FluidId,   // f
    pub chapter: u8,      // K
    pub face: FaceKey,    // F
    pub form: FlowForm,   // P — free / bound
    pub cause: FlowCause, // C — the mover
}
```

Faces are **3D**, because lateral-only forecloses most of the hydrosphere
(flow.md § 2.3): eight lateral D8 faces, two vertical (slot ↔ slot), and three
boundary (atmosphere, ocean, base level). Entries are **directed half-faces**,
stored once on the cell they leave, in a CSR index by cell; a cell's *in*-faces
are a **query over its neighbours' stored entries**, never a second copy that
could drift. That is what makes seamlessness structural rather than aspirational
— and it also means a chapter in which flow reversed records both crossings,
which is a down payment on flow.md § 5's "gross energy separate from net flux."

### What is populated, and what is honestly zero

| | slice 1 |
|---|---|
| **lateral** (cell ↔ cell) | **populated** — the routed discharge, per chapter |
| **boundary: ocean / base level** | **populated** — where flow left the model |
| **boundary: atmosphere (sink)** | **populated, small** — closed-basin termini |
| **boundary: atmosphere (source)** | **not stored, on purpose** |
| **vertical** (slot ↔ slot) | **structurally present, always zero** |

Two of those deserve their reasons stated out loud, because "we left it empty"
and "we did not think about it" look identical from outside.

**The vertical faces are zero because this solve has no vertical term.** It is
pure surface routing: no infiltration, no percolation, no Darcy flux. There is
no honest number to write, and writing a plausible one would be exactly the
stand-in-becomes-the-definition failure (A-1) the inventory exists to prevent.
The variants exist so the shape does not have to move when the head field
(continuation (a)) and the free/bound edge (continuation (c)) land.

**The atmospheric source is not stored because it is derivable.** The solve
seeds one unit of contributing area per cell per epoch — uniform, exact,
predictable. S-2 says store only what derivation cannot predict, so recording
297,025 × 8 copies of the constant `1.0` would be paying real megabytes for
zero information. (`grid.precip` exists and is orographically marched — but the
accumulation does *not* read it, so a precip-weighted source would be a number
the record asserts and the sim never used. A real one arrives with the head
field.)

The atmospheric **sink** is populated, and it is the one place I made an
interpretive call. When priority-flood leaves an interior cell with no lower
outlet, the solve simply stops routing there. Physically, the only way mass
leaves a closed basin is evaporation, and flow.md § 2.3 names the top face as
precisely that sink — so an endorheic terminus is recorded on
`FaceKey::Atmosphere` rather than silently dropped. A playa is a face, not a
missing edge.

## The slot is derived, and it is usually absent

The atom says *at (cell, **stratum-slot**)*. I do not store the slot. `DepUnit`
already carries a chapter stamp and the recorder refuses to merge units across a
chapter boundary, so the slot is `slot_for_chapter(strata, K)` — the index of
the last unit stamped K. S-2 again: don't store what derivation predicts.

Then the test printed a number that changed how I read the whole design. On the
small production world, of the (cell, chapter) pairs that carried flux:

```
slots resolved 29,204   ·   chapters with NO surviving stratum 175,596
```

**Six in seven chapter-fluxes have no slot at all.** Which is correct, and it is
the geology, not a bug: a cell that was net-erosional in chapter K deposits no
unit for chapter K, and a cell whose chapter-K unit was later stripped has an
unconformity where it used to be. Water flowed; the record of the flowing is
gone.

Two consequences fell out of that number.

1. **Storing the slot would have been actively wrong.** A stored index does not
   know that its unit was eroded away — it goes stale and points at a stranger's
   stratum. The derivation answers `None`, which is the truthful answer and is
   *itself* the unconformity signal flow.md § 1.2 wants ("an unconformity is not
   missing data — it is a flow signature").
2. **The flow record must be keyed by chapter, not by slot.** Had I keyed the
   archive per (cell, slot), 86 % of the flux would have had nowhere to live and
   would have been silently discarded — the exact defect this arc set out to
   fix, re-committed one layer down.

## The design doc is silent on the rule everything rests on

flow.md § 2.2 says a face is *"shared by construction"* — cell A's east face **is**
cell B's west face — and hangs the whole seamlessness guarantee on it. That
holds at the **cell-pair** level and stops there. Adjacent columns do not have
aligned slot indices, and § 1.2 has already forbidden correlating by slot index
because surfaces are diachronous. So *what pairs with what?* The document does
not say.

I implemented the rule I can defend and I am flagging it rather than smuggling
it: **lateral faces pair by CHAPTER.** Cell A's out-face F in chapter K is the
same physical face as cell B's in-face `opposite(F)` in chapter K; each column
then binds that chapter to its own slot independently. Chapter is a *time*
surface, shared globally, and the flow being paired is contemporaneous surface
flow across a shared boundary — so this is the § 1.2 discipline applied, not
violated.

**The residual is real and it is the user's call.** The *bound* regime probably
needs a different pairing: an aquifer's flux crosses a cell boundary at a
**paleo-elevation**, not at "the same chapter's stratum," and regional
groundwater genuinely crosses surface divides (§ 2.4). Slice 1 records no bound
flux, so nothing is forced yet — but this rule must not be assumed to extend
there by default. Filed with continuation (c).

## The A-4 fold: one merger where there were two

Routed into this slice deliberately. The 2026-07-24 spine audit had caught two
fact-mergers 300 lines apart: `inventory.rs::commit_chapter` merged a drained
edge into `facts.last_mut()` — *consecutive* edges only — and
`weather_inventory.rs::coalesce_facts` re-implemented the same merge generalized
to non-consecutive slots, called in a loop over every slot after every commit.

The reason the second one existed is instructive. `weather_bedrock_epoch` fires
three agents per epoch in rotation — chem, biotic, frost, chem, … — so **no two
successive edges on a slot ever matched**, last-only merging never merged
anything, the facts grew three per firing into the hundreds per cell, and a
reaper had to be written to walk behind and clean up. The fix is one line of
lookup: search the slot instead of checking its tail. `coalesce_facts` is
deleted.

They are equivalent, and the argument is short: the search merges into the
*earliest* match, which is exactly the first-occurrence order the post-hoc sweep
preserved, and `Σ fraction_m` — the only quantity the composed band reads — is
invariant under either grouping. The weathering suites stayed green **by name**
across the fold. It also takes an O(slots) post-pass off a per-cell, per-epoch
path, which is the sort of thing "one mechanism, not two" quietly buys.

## The acceptance, on a production world

`examples/flux_record_probe.rs`, seed 1337, `Extent::Medium`, production flags.
545 × 545 = 297,025 deep cells, 8 chapters, 200 epochs.

```
DIVERGENCE  (cell,chapter) with >=2 out-faces :  175,320   (7.378 % of pairs)
CONVERGENCE (cell,chapter) with >=2 in-faces  :   60,915   (2.564 % of pairs)
max out-faces on one cell/chapter             :        6
max in-faces  on one cell/chapter             :        8
```

**175,320.** A receiver tree's count of that column is zero — not "small", not
"needs tuning": zero, by construction, forever. Seven percent of the world's
cell-chapters are junctions where flow left by more than one face. Deltas,
braids, fans and distributaries are *representable* for the first time.

The single largest one is worth reading, because it is a story rather than a
statistic — cell (436, 345), ~200 km east and ~159 km north of the grid corner,
in chapter 0:

```
  out     W  magnitude    303.0
  out    SE  magnitude  16185.0
  out   sea  magnitude     23.0
  in      W  from 188460  magnitude 16160.0     <- the trunk arriving
  in     SE  from 189007  magnitude   280.0
  in     NE  from 187917  magnitude    42.0
  ...and four more at 1.0 (their own rainfall)
  chapter-0 stratum slot: Some(2)

  same cell, final chapter 7:
  out   sea  magnitude     25.0
```

A trunk stream arrives from the west carrying sixteen thousand cell-units, and
over chapter 0 it leaves through **three different faces** — mostly southeast,
partly back west, and partly straight into the sea, because the sea-level
sinusoid put this cell underwater for part of the chapter. That is a shoreline
distributary. And by chapter 7 the same cell is simply *ocean*: 25 units of its
own rainfall and nothing else. The trunk is gone. **That is the paleo-history
the receiver tree threw away two hundred times.**

## Cost, measured

A sibling agent's projection landed mid-build with a warning worth repeating:
`FactLedger`'s `Vec<Vec<…>>` shape is **98.8 % empty inner Vecs**, 89 % of its
heap being empty Vec headers. A per-(cell, slot) flow record would have repeated
that exactly. The layout here is the opposite by construction — a flat entry
array plus a `(n + 1)`-long `u32` index, both exact-sized, and only faces with
non-zero flux stored at all.

```
flow record total             40.66 MiB   (39.53 entries + 1.13 CSR index)
  2,590,372 entries x 16 B
per cell                     143.54 B     (8.721 entries/cell)
per cell per chapter          17.94 B     (1.090 entries/cell/chapter)
FACE SPARSITY                  8.386 %    (13.627 % of the lateral-only rectangle)
DeepField without the record 162.57 MiB
DeepField with    the record 203.23 MiB   = 1.25x
```

**8.386 % face sparsity** is the number the S19 cost model was missing, and it is
now measured rather than assumed. The record costs a quarter of the world's
existing residency.

### The one lever, sized but deliberately not pulled

**79.38 % of all entries — 31.37 MiB of the 40.66 — are marine sink faces
carrying nothing but the cell's own seeded rainfall.** A subsea cell that nothing
drains into records `area = 1.0` per epoch, which is the constant the solve puts
there; by exactly the argument that keeps the atmospheric *source* out of the
record (S-2: store only what derivation cannot predict), those entries are
derivable from their own absence. Dropping them would leave **9.29 MiB** — a
record that costs 5.7 % of the DeepField instead of 25 %.

I did not take it. Not because it is wrong, but because it changes what an
absent entry *means* — from "no flux" to "no flux, or flux equal to the seeded
constant" — and that is a semantics decision the record's future consumers have
to live with. The brief said report the number even if it is large and do not
silently truncate; so here is the number, and here is the lever, sized, for
whoever owns the call.

### The load channel is live

Worth stating because a permanently-zero field looks identical to an unwired
one: **494,296 entries carry load — 99.86 % of all lateral faces** — totalling
1,864.8 m of transported material, with a largest single-face load of 0.031 m.
It is *bulk* load only; the composition (which materials, in what proportion —
what a placer streak in channel gravel is actually made of) is a named seam whose
heir is Movement 2b's material-aware transport. An honest bulk number now; a
fabricated composition never.

## What is deliberately NOT here

Nothing expresses this at runtime. No refinement, no carving, no channel. The
world stays **honestly river-less** rather than gaining a second fake beside the
first one, and `RiverSeg`/`carve_rivers`/`BANK`/`RIVER_REACH` are untouched —
their retirement is continuation (e), not this slice. `pregen/hydrology.rs` is
untouched. `recv`/`area`/`lake` still ship, now with one *test* consumer that
pins them as a **shadow** of the flux record rather than a rival authority
(`the_receiver_export_agrees_with_the_final_chapters_faces`): the final routing
must be among the final chapter's faces, or one of the two is lying about the
same solve.

## Shapes

Rides **S-9** (base + facts — the record is the archive, the surface planes are
its views), **S-2** (store the flux; derive the slot, derive the channel later,
never store the atmospheric constant), **S-4** (coarse cause → fine expression,
with shared faces constraining the fine side), the **field/cellular split** of
material-behavior.md § 5 — which survived first-principles re-derivation
untouched — and the north-star **pass-runner** (`dc:deep/flow_record` declares
`{reads: [Routed, Energy], writes: [FlowFlux]}` over plain data, opaque ids and a
bare `fn` body; no closures cross the seam).

Discharges **A-4** (the merger fold, and the spines § 3 row for the unconsumed
`recv`/`area`/`lake` is updated with what superseded it). Guards **A-3** (the
acceptance is a divergence count on a production world, not a green unit test)
and **A-1** (the honest empties are asserted *as* empty, by name, so the slice
that populates them must delete the assertion on purpose).
