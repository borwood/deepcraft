# S15 results — coarse capacity against a lazily generated, evicting world

Status: spike complete, 2026-07-22. Code is **additive and standalone** in
`crates/dc-worldgen/src/water/coarse.rs`; measurement harness
`crates/dc-worldgen/examples/water_coarse_spike.rs`; invariants
`crates/dc-worldgen/tests/water_coarse.rs`. Nothing in the production
world-generation path calls it, deep time is untouched, the renderer is
untouched. Numbers from the Windows dev box, release profile.

The one dependency added is `dc-api` as a **dev-dependency** of dc-worldgen
(examples + tests only). That is the entire point of the spike: S11 ran against
`water/vox.rs`, a fully-resident toy one-bit volume, so **its 415 ms capacity
scan is an honest number for a world that does not exist**. This one runs
against `WorldGenerator` on the client's own world (seed 1337, `Extent::Medium`,
0.9 m voxels) and `HostWorld` — the bounded LRU over generated-and-untouched
chunks with pinned edited chunks (journal/0051) — with every edit going through
its audited command path.

The question (docs/design/water.md § SPIKE SPEC S15): does a **coarse capacity
mechanism** remove the need to voxel-walk a water body's container, and does the
connectivity claim survive an adversarial attempt to break it?

**Headline: GO.** The coarse-derived level lands within half a voxel for every
body larger than **26 m²** — 781 of 791 measured bodies, up to 212 000 m². The
mechanism costs **zero chunks**; the exact walk it replaces cost **6 444 chunks
and 64 seconds** over the same 791 queries. Incremental maintenance drifts
**1.9 × 10⁻⁵ m over 20 000 edits**. And the connectivity claim **broke**: one
voxel of edit re-levels water 288 m away, through terrain the edit never
touched.

---

## The mechanism

`coarse.rs` is one idea. Capacity is **additive**, so a body's volume↔level
curve is a sum over coarse cells:

```
V(L) = Σ_cells [ Σ_columns max(0, L − floor(column))      ← the hypsometric summary
               + Σ_edits  delta_y · clamp(L − y, 0, 1) ]  ← the memory of edits
```

- A **capacity cell** is 32 columns square — one chunk footprint, 28.8 m, the
  same unit the chunk store generates and evicts.
- Its summary is a **histogram of floor planes** (the y of the lowest air voxel)
  built from **4×4 = 16 sub-samples** standing in for 1 024 columns. That is a
  **64× compression**, and the level cost of that compression is what group 1
  measures.
- The samples come from `WorldGenerator::coarse_surface`, which is memoized and
  **generates no chunks** (journal/0022). Measured here at **2.98 µs/sample**
  (9 409 cells, 150 544 samples, **449 ms, 0 chunks generated**) — slower per
  sample than 0022's 1.377 µs/column because the sub-sample pattern is
  scattered rather than a strided march, and each cell is 16 cold columns.
- An **edit is a signed integer delta by y**, hung on the audited write path:
  `+1` when a solid became air, `−1` when air became solid. Not a rescan, and
  not a float accumulator — which is why group 2's drift is float bisection
  noise rather than accumulation.

Both curves — coarse and exact — are inverted by **identical bisection
arithmetic** (`level_for`), so every number below reads a *mechanism*
difference, never a formula difference.

**The summary persists nothing new.** The histogram half is a pure function of
(seed, cell) and re-derives at ~48 µs/cell with no chunk generation; the edit
half is already world state (the chunks holding those edits are pinned by
journal/0051's policy). This is the S11 rule — *store only what the derivation
cannot predict* — landing on the capacity curve.

---

## Group 1 — coarse capacity accuracy

**Method.** Survey a window of capacity cells over the production world (9 409
cells = 7.8 km², 449 ms, **zero chunks**), take its local minima as basin seeds
(1 137 of them), and flood each at a ladder of depths and footprint caps. Depth
moves the wetted area *inside* one cell — the sub-cell end, where the summary is
most compressed; the cap moves it across cells — the lake end. Together they
cover four orders of magnitude of surface area against the same real terrain.

For each body: walk **every voxel of the footprint through `HostWorld::block_at`**
(the storm), take the exact volume at the flood level, and ask the coarse
summary to recover the level from that volume alone.

**791 bodies measured.** A log-spaced slice of the table:

| cells | wetted cols | area m² | level | volume (vox) | coarse level | err (vox) | err (m) | exact scan ms | chunks |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 3 | 2 | 938.00 | 3 | 937.047 | −0.953 | **−0.858** | 0.2 | 0 |
| 1 | 28 | 23 | 962.00 | 28 | 961.438 | −0.563 | **−0.506** | 0.2 | 0 |
| 1 | 32 | 26 | 929.00 | 32 | 928.500 | −0.500 | −0.450 | 0.2 | 0 |
| 1 | 142 | 115 | 951.06 | 26 | 951.202 | +0.140 | +0.126 | 0.2 | 0 |
| 1 | 617 | 500 | 939.00 | 620 | 938.965 | −0.035 | −0.031 | 0.3 | 0 |
| 4 | 1 509 | 1 222 | 960.00 | 1 692 | 959.977 | −0.023 | −0.021 | 4.5 | 2 |
| 8 | 6 721 | 5 444 | 932.00 | 14 762 | 932.013 | +0.013 | +0.012 | 7.1 | 2 |
| 16 | 14 760 | 11 956 | 944.00 | 36 686 | 944.009 | +0.009 | +0.008 | 22.8 | 3 |
| 32 | 32 687 | 26 476 | 955.00 | 209 913 | 954.990 | −0.010 | −0.009 | 19.8 | 0 |
| 128 | 101 895 | 82 535 | 947.06 | 334 671 | 947.046 | −0.017 | −0.015 | 107.5 | 1 |
| 256 | 227 444 | 184 230 | 947.06 | 1 230 276 | 947.049 | −0.014 | −0.012 | 368.5 | 15 |
| 256 | 262 144 | 212 337 | 993.00 | 8 816 948 | 992.979 | −0.021 | −0.019 | 677.4 | 128 |

### The decision rule, applied

Fixed in advance: *if the coarse-derived level lands within half a voxel
(0.45 m) above some surface-area threshold, the coarse path ships and exact
scans are reserved for bodies below that threshold.*

> **Threshold: 26 m² of water surface (32 columns).** Every one of the 781
> bodies at or above it lands inside half a voxel. Only **3 rows in 791** exceed
> the rule at all, and they are puddles: 2 m² (3 wetted columns, err 0.858 m)
> and 23 m² (28 columns, err 0.506 m).

**The fallback is bounded, and small.** The largest body still handed to the
exact path is **23 m²** — an exact walk of **0.2–3.2 ms generating at most 2
chunks** (3.2 ms is the cold-generator first touch; warm it is 0.2 ms). There is
no open-ended fallback here: everything the coarse path declines is a puddle you
could stand across.

### The predicted accuracy property — HELD, but not for the predicted reason

The brief predicted level error behaves as `ΔV / surface area`: best for big
lakes, worst for narrow flooded shafts. Measured over the 791 rows, mean |err|
is **0.0521 m in the small-area half and 0.0135 m in the large-area half** —
error falls with area, and the `ΔV/A` column tracks the level error row for row.
So the law holds.

**But its named worst case is exactly wrong.** A narrow flooded shaft is *dug*,
and a dug void enters the summary as an **exact integer delta**, not as a
sample. Measured: a 1×1×64 shaft cut through the audited write path (64 opened
voxels), filled with 32 voxels of water — coarse level `981.000000`, exact
`981.000000`, **error 0.000000 m**.

The mechanism's real worst case is a **small natural depression**: sub-cell
relief that 16 samples cannot resolve, with no dug voxels to correct it. That is
why the failing rows are 3-column and 28-column puddles rather than shafts. See
corrections.

### Cost

| | 791 level queries |
|---|---|
| exact voxel walk | **64 104 ms**, **6 444 chunks generated** |
| coarse summary | **109 ms** (138 µs/query), **0 chunks** |
| building the summary | 163 968 surface samples, **0 chunks** |

~590× on wall clock, and the chunk column is the one that matters: the storm
this replaces is what evicts the ground the player is standing on.

---

## Group 2 — incremental maintenance through the audited write path

**Method.** One 64-cell basin, water level 999. Dig 20 000 voxels through
`world/set_block` — submit, tick, read the receipt's `blocks_changed` — and feed
every audited change into the summary as a delta. After each 2 000-edit batch,
re-walk the container exactly and compare.

| edits | coarse level | exact level | err (m) | drift (m) | exact rescan |
|---:|---:|---:|---:|---:|---:|
| 2 000 | 998.94671 | 998.96936 | −0.020389 | 0 | 181.6 ms |
| 6 000 | 998.88543 | 998.90808 | −0.020384 | 4.2e−6 | 183.9 ms |
| 10 000 | 998.82416 | 998.84680 | −0.020380 | 8.4e−6 | 181.8 ms |
| 16 000 | 998.73225 | 998.75488 | −0.020374 | 1.5e−5 | 184.6 ms |
| **20 000** | 998.67097 | 998.69360 | **−0.020370** | **1.9e−5** | 183.8 ms |

- **Drift over 20 000 edits: 1.9 × 10⁻⁵ m** — the same family as S11's
  2.4 × 10⁻⁷ conservation drift, and it is not accumulation: the deltas are
  integer counts in a `BTreeMap`, so there is nothing to accumulate. What moves
  is the bisection's own resolution as the curve shifts.
- **It does not break.** Extrapolating the observed rate (9.5 × 10⁻¹⁰ m/edit)
  against the half-voxel rule gives **~4.7 × 10⁸ edits**, and even that is an
  artifact of the extrapolation rather than a real ceiling.
- The residual **−0.0204 m** is group 1's compression error for this body, held
  constant across the whole session. That is the point: the edit path adds no
  error of its own.
- **Order-independent** over a shuffled 500-edit batch: byte-identical summary
  hash, asserted in `tests/water_coarse.rs` as well as measured.

### A wrong turn worth recording

The first run of this measurement showed err growing −0.023 → −0.034 m over
1 000 edits, and it looked exactly like drift. It was a harness bug:
`CapSummary::floor` follows the deepest dug voxel, and the shaft's target y was
recomputed from it every edit, so the shaft walked itself one voxel deeper per
edit and straight out of the exact scan's y window. The *reference* had stopped
seeing the edits the coarse path was still counting. Capturing `floor0` once
fixed it. Recorded because a plausible-looking drift curve is exactly the shape
a spike is most tempted to publish.

---

## Group 3 — the connectivity falsifier

The claim under test: **connectivity only changes where someone edits, and edits
only happen where chunks are loaded** (to breach a lake you must dig, and to dig
you must be there). The job was to break it.

### Falsifier A — the natural sill: NOT constructible today, and the reason matters

Searched 1 215 pairs of far-apart basin floors for two bodies at a common level
separated by a sill that a rising water level would overtop. Result:

- **972 of 1 215 pairs were already ONE body** at a common level;
- the remaining 243 flooded past the 900-cell cap before any sill separated
  them;
- **0 usable separations.**

The mechanism is not subtle. `WorldGenerator::generate_chunk` fills solid below
the column's surface height and air above it — **the world is a pure heightfield
with no caves and no overhangs** (ruin posts are the only exception, and they
are above ground). So at any level, all the sub-level air of a drainage basin is
a single component *by construction*. Disconnected same-level containers require
3-D structure.

**What would make it constructible: caves.** The moment water.md's cave families
land, a flooded conduit and a surface basin at the same level, separated by
rock, become routine — and then a rising level overtopping an internal sill
merges two bodies **with no edit anywhere**. That is the case to re-run this
group against, and it is scheduled work, not hypothetical.

### Falsifier A′ — one voxel, an unbounded consequence: **CONSTRUCTED**

So the player's version was built instead, against the real store. Dig a 1×1
tunnel 640 voxels out of a lake (615 audited voxel edits, **21 chunks
generated**), leaving **one solid plug** at the midpoint. The far half is then a
sealed tube — its own body, disconnected, verified solid at the plug.

Then remove the plug: **one `world/set_block`, one chunk touched, zero chunks
generated** — and the body it joins reaches **288 m beyond the edit**.

> **The claim splits, and the second half is false.**
>
> - *"Connectivity only changes where someone edits"* — **TRUE**, and for a
>   strong reason: terrain is a pure function of the seed, so unedited geometry
>   cannot change connectivity. Eviction and re-derivation are byte-identical
>   (group 4), so even the store cannot perturb it.
> - *"…and therefore the consequence is inside the loaded set"* — **FALSE**.
>   One voxel re-levels water 288 m away. There is no halo that bounds it, and
>   nothing about "you had to be there to dig" bounds it either: you were there
>   at the plug, not at the far end.

This is the same non-locality S11 named for free water, now measured against
lazy generation: the *edit* is local, the *consequence* is graph reachability.
Which is precisely why the coarse mechanism matters — the consequence has to be
priced without walking it.

### Falsifier B — an edit at a chunk boundary whose neighbour is absent: NOT constructible

`HostWorld` has **no absent state**. Reading across a chunk boundary at
(9 599, 40, 9 600) generated one chunk, then the neighbour generated one more.
`block_at` materializes on demand, always, everywhere — lazy generation is total
and transparent.

**What would have to change for it to become constructible:** a store that can
*fail* to answer — asynchronous streaming, disk-backed region files, a network
authority. That is exactly the save layer journal/0051 says does not exist yet.
When it lands, "the neighbour is absent" becomes a real state and this falsifier
becomes constructible at will. Flag it now: **the connectivity index must never
be allowed to infer a component boundary from a chunk's absence.** An absent
chunk is unknown, not solid.

### Falsifier C — a container over never-generated ground: **the normal case**

A 2 000-cell body — **1.66 km² of water surface** — answered its volume
(56 192 872 voxels at level 993) having generated **zero chunks**. The exact
path would have had to generate **2 000 chunk footprints** of terrain nobody has
ever stood in.

Combined with the 972-of-1 215 result above, the honest statement is that a
body's container spanning never-loaded terrain is not an adversarial case at
all — **it is the default**. Every lake in this world is like that.

---

## Group 4 — eviction identity against the real evicting store

S11 proved byte-identical reload from 39 persisted bytes against a toy volume.
Redone against `HostWorld`:

1. Build a 64-cell body's capacity curve and its derived water; apply one edit
   inside it (so the pinned-chunk half of the policy is exercised).
2. Squeeze the budget to 8 chunks and storm the store with 3 000 fresh chunk
   positions 900 km away.
3. Re-walk and re-derive.

| | before | after 3 120 evictions |
|---|---|---|
| exact capacity curve | `0xd3616773f7b2f8c3` | `0xd3616773f7b2f8c3` |
| derived water | `0xd25b942d8484b54b` | `0xd25b942d8484b54b` |
| the edit | air | air (chunk still pinned, `edited: 1`) |

**Byte-identical on both.** 127 chunks were regenerated to answer the second
scan; it cost **153.5 ms against the first scan's 254.5 ms** over 1 703 936
voxels — regeneration was *cheaper* than the cold first walk, because the
generator's own collapse caches were warm by then.

Double-run and cell-order-independent summary: byte-identical.

---

## Determinism

- Coarse summary **double-run byte-identical**, and identical under reversed and
  shuffled cell orders (`summary_hash`, asserted in `tests/water_coarse.rs`).
- Edit application **order-independent** — integer deltas in a `BTreeMap`, so
  there is no ordering freedom to lose. Proven over a shuffled 500-edit batch
  and a 200-edit unit test.
- Capacity sums iterate `BTreeMap` and `Vec` in fixed order; `level_for` is a
  fixed 64-round bisection. No wall clock, no ambient randomness; every sample
  is a pure function of (seed, column).

---

## Design choices where the notebook was silent — each FLAGGED

1. **FLAG: the capacity cell is 32 columns (28.8 m), sub-sampled 4×4.** Both
   numbers are knobs (`CAP_CELL`, `CAP_SUBSAMPLE`) and both were chosen, not
   derived: 32 because it is the chunk footprint the store already evicts, 4×4
   because it is 1.6 % of the columns. The measured threshold (26 m²) is a
   function of both. Per the user's standing doctrine — *where there's a cell
   range, there's a knob* — they ship adjustable.
2. **FLAG: a body's footprint here is a set of capacity cells**, delineated by
   flooding the coarse summary. That is a *cell-granularity* container, so a
   basin narrower than 28.8 m is not representable as its own body. S11's
   air-component container is the finer story; the two need reconciling before
   this ships (a body is an air component; its capacity is a sum over the cells
   that component touches).
3. **FLAG: capacity below the summary's floor plane comes only from edits.** In
   today's heightfield world that is complete. **The moment caves exist it is
   not** — a natural void under a lake is capacity the summary cannot see, and
   the coarse cell will need a void/conduit axis. water.md's collapse-tier
   "recorded conduit capacity → void intervals per column" is exactly the shape
   that axis wants; this is a real dependency between the cave thread and this
   mechanism, and it should be designed once rather than twice.
4. **FLAG: level inversion is a 64-round bisection**, not a closed form. It
   measures 138 µs for a 256-cell body. Cheap, but it is O(cells × rounds) and
   would want a prefix-sum or a cached monotone index before a body reaches 10⁵
   cells.
5. **FLAG: the exact fallback path is defined but not implemented as a policy.**
   The spike measures both paths; it does not build the dispatcher that picks
   one. The threshold (26 m² of wetted surface) is the input that dispatcher
   needs.
6. **FLAG: `Pinned` bodies were not re-measured here.** S11's finding stands —
   a pinned reservoir needs no capacity curve at all, so its coarse cost is
   also zero. Nothing in S15 changes that; it just makes the *finite* case
   affordable too.

---

## Recommendation — **GO**

**Ship the coarse capacity mechanism; keep the exact walk as a bounded fallback
below 26 m².**

1. **The decision rule passed with room.** 781 of 791 bodies inside half a
   voxel, threshold at 26 m² — five metres across. The three genuine failures
   are 2 m² and 23 m² puddles.
2. **The cost argument is not close.** 0 chunks against 6 444; 138 µs against
   81 ms per query. The storm this deletes is the one that evicts the ground the
   player is digging, which is the reason the spike exists.
3. **The fallback is bounded**: ≤3.2 ms and ≤2 chunks at the largest body it
   ever receives.
4. **The incremental path adds no error** — 1.9 × 10⁻⁵ m over 20 000 edits, and
   the residual is group 1's compression, unchanged across the session.
5. **Eviction is invisible** to both the curve and the derived water,
   byte-identically, against the real bounded LRU with an edit pinned inside it.
6. **The accuracy property is real** and points the right way: the mechanism is
   most accurate exactly where the exact walk is most expensive.

**Not user-owned; nothing here needs ratification.** The knobs (cell edge,
sub-sample rate, threshold) are engineering calibrations of the same status as
S11's halo — they ride as measured. The one thing that *is* a design question is
flagged above: **the cave families and this mechanism share an axis** (void
capacity below the surface), and that should be decided once, in water.md, not
discovered twice.

---

## Falsified along the way

Filed in `journal/corrections.md`:

1. **"The narrow flooded shaft is the coarse mechanism's worst case"** (this
   brief, and water.md § SPIKE SPEC S15) — **corrections #30.** It is its *best*
   case: a shaft is dug, and a dug void is an exact integer delta on the audited
   write path, not a sample. Measured error 0.000000 m. The real worst case is a
   small **natural** depression, where sub-cell relief is unresolved and there
   are no edits to correct it — 0.858 m at 2 m².
2. **"Connectivity only changes where someone edits, and edits only happen where
   chunks are loaded"** — **corrections #31.** The first clause is true; the
   inference drawn from it is false. One voxel removed from a plug re-levels
   water 288 m away, and 972 of 1 215 far-apart basin floors are already one
   body through terrain nobody ever loaded. The edit is local; the consequence
   is graph reachability, and nothing bounds it.

Recorded here only, because it never became a claim outside this document:

3. **"A capacity comparison against the real generator will be dominated by
   caves and overhangs the column height cannot see."** It is not, because
   there are none: `generate_chunk` is a pure heightfield fill. That makes
   today's comparison a clean measurement of *compression* alone — and it means
   this whole result carries an expiry date stamped "when caves land" (design
   choice 3).
