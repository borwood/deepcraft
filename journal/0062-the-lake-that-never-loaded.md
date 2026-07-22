# 0062 — the lake that never loaded

*2026-07-22. S15: coarse capacity against a lazily generated, evicting world.
`docs/spikes/S15-results.md`, `crates/dc-worldgen/src/water/coarse.rs`.*

S11 came back GO on "persist bodies, derive voxels," and one of its numbers went
into the notebook and started getting quoted: **415 ms for a capacity scan.**
That is what it costs to walk a sea's container and learn its volume↔level
curve. Expensive, but survivable — you only pay it when something breaches.

The user found the hole in it before anyone else did. S11 ran on
`water/vox.rs`, a deliberately toy one-bit solid/air volume that is entirely
resident in memory. There is no lazy generation anywhere in that harness. So the
415 ms is the cost of walking memory that was simply *there* — and in the real
world, nothing is there until you ask for it. A capacity scan against the real
generator is a **generation storm**, and worse, the chunk store is a bounded LRU
(journal/0051), so the storm **evicts the ground the player is standing on**,
right when they are actively digging the trench that caused it.

> 415 ms is an honest number for a world that does not exist.

And then the user proposed the answer in one sentence: *"if cells / the coarse
regions know roughly their level (and remember if it changes — remembering
player edits) then that math could be simpler."*

This entry is the spike that tried to break that.

---

## The idea, and why it should work

Capacity is **additive**. The volume below a water level `L` is a sum over
regions, so if every coarse cell carries a small hypsometric summary — how many
of its columns have their floor at each height — then a body's whole volume↔level
curve is a sum over cells the game already holds. No walk.

```
V(L) = Σ_cells [ Σ_columns max(0, L − floor(column))      ← the summary
               + Σ_edits  delta_y · clamp(L − y, 0, 1) ]  ← the memory
```

Two things make the sum cheap where the walk is expensive:

- the per-column floor comes from `coarse_surface`, which journal/0022 measured
  at 1.377 µs/column, memoized, **generating no chunks**; and it can be
  **sub-sampled** — 16 samples standing in for 1 024 columns, a 64× compression;
- an **edit is a delta, not a rescan**. Digging removes a known volume at a
  known height, and `set_block` is the single audited voxel-writing path
  (journal/0051) to hang that increment on.

So: a `CapCell` is a histogram of floor planes plus a `BTreeMap<y, i64>` of
signed edit deltas. A `CapSummary` is a sparse field of them. Total: 400 lines.

The interesting part is not the code. It is that the whole thing is a **summary
standing next to an authority**, which this project has a rule about — *"if this
consumer disappeared tomorrow, would this code still exist in this shape?"*
(ARCHITECTURE, 2026-07-21). The answer here is no: the coarse summary exists
because the voxel walk is unaffordable. So it needs a test asserting it AGREES
with the authority, and that test is the spike's whole group 1.

## Building the measurement so it could actually see the question

The brief was explicit that the harness had to use the real generator and the
real chunk store, and to say so loudly if that proved impossible. It did not
prove impossible, but it took a dependency nobody had needed before: dc-worldgen
now has **dc-api as a dev-dependency**. dc-api is headless and wasm-safe and does
not depend on dc-worldgen, so there is no cycle and no convention broken; it just
means the spike's examples and tests can hold a real `HostWorld` with a real
`WorldGenerator` behind it — the same wiring `dc-client/src/authority.rs` boots
the game with, seed 1337, `Extent::Medium`.

Both capacity curves — coarse and exact — are inverted by the *same* bisection.
That was deliberate. If the coarse path had its own inversion, every number
would be a mixture of "the summary is lossy" and "the two formulas disagree,"
and there would be no way to tell them apart.

## What the numbers said

**791 real bodies**, from a 3-column puddle to a 212 000 m² lake, each one a
container of production terrain flooded to a real level, each one walked
voxel-by-voxel through `block_at` for the exact answer and then asked of the
summary.

The decision rule was fixed in advance: within half a voxel (0.45 m), because
that is where the shoreline moves.

> **781 of 791 landed inside half a voxel. The threshold is 26 m² — five metres
> across.** The three genuine failures are a 3-column puddle (0.858 m) and a
> 28-column puddle (0.506 m).

And the cost column is not close:

| | 791 level queries |
|---|---|
| exact voxel walk | 64 104 ms, **6 444 chunks generated** |
| coarse summary | 109 ms, **0 chunks** |

Zero. Not "fewer." The coarse path answered a 1.66 km² lake's volume having
generated no chunks at all, while the exact path would have had to materialize
2 000 chunk footprints of terrain nobody has ever stood in.

The fallback the decision rule creates is bounded, which was the other thing the
brief demanded: the largest body still handed to the exact walk is **23 m²**, and
walking it costs **0.2–3.2 ms and at most 2 chunks**. Everything the coarse path
declines is a puddle you could stand across.

## The prediction that was backwards

The brief predicted the error law — level error is `ΔV / area`, so the estimate
is best for big lakes and **worst for narrow flooded shafts**. The law held: mean
|err| is 0.0521 m over the small half of the area sweep against 0.0135 m over the
large half.

The named worst case did not.

A narrow flooded shaft is **dug**. A dug void does not enter the summary as a
sample of terrain — it enters as an **exact signed integer delta on the audited
write path**. So the mechanism is not merely good there, it is *exact*: a 1×1×64
shaft filled with 32 voxels of water gives coarse level `981.000000` against
exact `981.000000`. Error 0.000000 m.

The real worst case is a small **natural** depression: sub-cell relief that
sixteen samples cannot resolve, with no edits to correct it. Which is why every
failing row is a natural puddle and nothing dug failed at all.

> blogworthy: *"smallest area ⇒ largest error" quietly assumes every container is
> sampled. The moment part of a container is recorded exactly — because a player
> made it, and the write path audited it — the small end of the area axis splits
> into two populations with opposite behaviour. Ask which term of the error a
> case actually exercises before you name it the worst case.*

Filed as corrections #30.

## The drift that wasn't

Group 2 dug 20 000 voxels through `world/set_block` — submit, tick, read the
receipt's `blocks_changed`, feed each change into the summary as a delta — and
re-walked the container exactly after every 2 000.

The first run showed the error growing from −0.023 m to −0.034 m over 1 000
edits. A clean monotone curve. It looked exactly like drift, and it was the most
publishable-looking thing on the screen.

It was a harness bug. `CapSummary::floor` follows the deepest *dug* voxel, and
the shaft's target y was recomputed from it on every edit — so the shaft walked
itself one voxel deeper per edit and marched straight out of the exact scan's y
window. The **reference** had stopped seeing the edits the coarse path was still
counting. Capture the floor once and the curve flattens:

| edits | err (m) | drift (m) |
|---:|---:|---:|
| 2 000 | −0.020389 | 0 |
| 10 000 | −0.020380 | 8.4e−6 |
| 20 000 | −0.020370 | **1.9e−5** |

**1.9 × 10⁻⁵ m over 20 000 edits** — the same family as S11's 2.4 × 10⁻⁷
conservation drift, and it is not accumulation at all: the deltas are integer
counts in a `BTreeMap`, so there is nothing to accumulate. What moves is the
bisection's own resolution as the curve shifts underneath it. The residual
−0.0204 m is group 1's compression error for that body, held *constant* across
the whole session. The edit path adds no error of its own.

## The falsifier, and the half of the claim that broke

The group the brief cared most about was the connectivity claim: *connectivity
only changes where someone edits, and edits only happen where chunks are loaded
— to breach a lake you must dig, and to dig you must be there.* The job was to
break it.

Two adversarial cases refused to be built, and the reasons are worth more than
the attempts.

**The natural sill.** Two lakes at a common level, separated by a rim that a
rising water level overtops — connectivity changing with *no edit anywhere*.
Searched 1 215 pairs of far-apart basin floors in the production world. **972 of
them were already one body.** The rest flooded past the search cap. Zero usable
separations. The mechanism is not subtle: `generate_chunk` fills solid below the
column height and air above it, so **this world is a pure heightfield with no
caves and no overhangs**, and all the sub-level air of a drainage basin is one
component by construction. Disconnected same-level containers need 3-D
structure. The day water.md's cave families land, this case becomes constructible
at will — and it should be the first thing re-run.

**The absent neighbour.** An edit at a chunk boundary whose neighbour is not
loaded. `HostWorld` has **no absent state**. `block_at` materializes on demand,
always, everywhere. Not constructible — and precisely constructible the moment
the store can *fail* to answer: async streaming, disk-backed regions, a network
authority. That is the save layer journal/0051 says does not exist yet. Flag for
the day it does: **an absent chunk is UNKNOWN, never solid**, or eviction will
manufacture false component boundaries out of nothing.

So the player's version was built instead. Dig a 1×1 tunnel 640 voxels out of a
lake — 615 audited voxel edits, 21 chunks generated — and leave **one solid
plug** at the midpoint. The far half is a sealed tube: its own body,
disconnected, verified solid at the plug.

Then remove the plug. **One `world/set_block`. One chunk. Zero chunks
generated.** And the body it joins reaches **288 metres beyond the edit.**

> **The claim splits, and only the first half survives.**
>
> *"Connectivity only changes where someone edits"* — **true**, and for a strong
> reason: terrain is a pure function of the seed, and an evicted chunk
> re-derives byte-identically, so neither time nor the cache can perturb it.
>
> *"…and therefore the consequence is inside the loaded set"* — **false.** One
> voxel re-levels water 288 m away. Nothing about "you had to be there to dig"
> bounds that: you were there at the plug, not at the far end.

Filed as corrections #31. It is S11's own finding — free water is connectivity,
and connectivity is not local at any radius — measured for the first time against
lazy generation. And it is the argument *for* the coarse mechanism rather than
against it: if the consequence of an edit is unbounded, then the consequence has
to be priced without walking it.

## Eviction, again, against the real store

S11 proved byte-identical reload from 39 persisted bytes against a toy volume.
Redone here: build a body's capacity curve and its derived water, put one edit
inside it, squeeze the chunk budget to 8, then storm the store with 3 000 fresh
chunk positions 900 km away. **3 120 evictions later**, both hashes are
unchanged and the edit is still air in its still-pinned chunk. The second scan
cost 153 ms against the first scan's 254 ms — regenerating 127 chunks was
*cheaper* than the cold first walk, because the generator's own collapse caches
had warmed in between.

## Where it leaves us

GO, and not a narrow one. But the result carries an expiry date, and it is worth
being explicit about it: **capacity below the summary's floor plane comes only
from edits.** In today's heightfield world that is complete. The moment caves
exist it is not — a natural void under a lake is capacity the summary cannot see
— and the coarse cell will need a void axis. water.md's collapse-tier "recorded
conduit capacity → void intervals per column" is exactly the shape that axis
wants.

Which is the one thing worth deciding rather than discovering: **the cave thread
and the capacity mechanism share an axis.** Design it once.
