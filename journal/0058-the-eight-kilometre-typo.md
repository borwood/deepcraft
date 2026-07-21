# 0058 — The eight-kilometre typo, and the skin that quantized

> blogworthy: a station labelled `(82 346, 24 391)` with no unit on it sent a
> walker 8.6 km away, and the world he found there — three voxels of mud where
> the journal promised eighty-nine — read exactly like a generator bug. The
> instrument was fine. The *label* was the defect. The fix is not code; it is
> that a coordinate written without its unit is not a coordinate.

Two things happened in this slice and only one of them was a bug in the world.

## Part one — the site that "didn't match"

journal/0055 reported, for the **loess margin**: `H` = 80.49 m, **89 voxels of
section, 188 spans, 76 mixed**. Then the user walked the live client — seed
1337, `Extent::Medium`, stock boot — dug at the coordinates the entry names, and
found **~3 voxels of mudstone over ~4 of basalt over granite**. Two columns, 180
m apart, both saying the same thing:

```
voxel (82346, 24391): y96–146 granite, y147–150 basalt, y151–153 mudstone, surface y153
voxel (82546, 24391): y120–140 granite, y141–145 basalt, y146–148 mudstone, surface y148
```

Eighty-nine against three is not a rounding disagreement. The candidate
explanations were, in ascending order of seriousness: the journal misreported;
the probe's site index → voxel conversion is wrong; or — the one worth stopping
the world for — **the client's world is not the probe's world**, and two
journal entries were ratified on numbers from a place nobody plays.

It is none of those. It is the unit.

`soil_depth_probe::STATIONS` holds its stations in **world metres**:

```rust
("3 loess margin (2.47 m)", 82346.0, 24391.0),
```

and divides by 0.9 to reach a voxel. journal/0055 then wrote the station down as
`(82 346, 24 391)` with no unit attached, in an entry whose every other
coordinate-shaped number is a voxel. Read as a voxel address it is world
(74 111 m, 21 952 m) — **8.6 km away, nineteen deep cells over**, with an
entirely different record. The loess margin's actual voxel address is
**(91 496, 27 101)**.

`examples/surface_dither_probe.rs` prints both readings of the same label, side
by side, so this cannot be re-litigated from memory:

| reading | voxel | `H` | deep record | expressed |
|---|---|---|---|---|
| **metres** (what the probe meant) | (91 496, 27 101) | **80.492 m** = 89.44 vox | 354 units, 80.49 m | 170 events, **188 spans, 76 mixed** |
| **voxels** (what the walker dug) | (82 346, 24 391) | **2.281 m** = 2.53 vox | 15 units, 2.28 m | 13 events, 102 spans, 4 mixed |

journal/0055 is **exactly right**: 89 voxels, 188 spans, 76 mixed, to the digit.
And the walker is **exactly right too**: 2.281 m of recorded regolith is 2.53
voxels, which is the three voxels of mudstone he dug, sitting on the basement
the igneous pass emplaced. Both observations were correct measurements of two
different places.

### The instrument that ended it: read the generator's own blocks

The back-and-forth above was conducted in two different currencies — the probe's
metres and contents on one side, `world_scan_region`'s **blocks** on the other —
and every comparison across that boundary was arguable. So the probe learned to
speak blocks. `block_column` generates the chunks at a voxel address and
run-length encodes the column downward from the surface: exactly what a live
scan reports, with no player edits in it.

At the four addresses the integrator actually scanned, the generator says:

| scanned voxel | generator, top-down | live client scan |
|---|---|---|
| loess margin (82 346, 24 391) | y153, `Mudstone×3 / Basalt×4 / Granite…` | y153, mudstone 3, basalt 4, granite |
| periglacial summit (−4 586, −3 207) | y1095, `Mudstone×12 / Basalt×3 / Granite…` | y1095, mudstone 12, basalt 3, granite |
| wave coast (95 224, 22 091) | y98, `Mudstone×4 / Basalt×4 / Granite…` | y98, mudstone 4, basalt 4, granite |
| barest land (101 663, 5 073) | y256, `CarbonaceousMudstone×1 / Basalt×3 / Granite…` | thin cap on basement |

**Block for block, run for run, y for y, at four sites spread across the world.**
There is nothing left of the divergence hypothesis. The client's world and the
probe's world are the same bytes.

### And there is no shortfall — the expression rule holds everywhere

The integrator's sharpest version of the worry was that *every* site
under-expresses against `round(H / 0.9)`, worst where the record is thickest —
which would be a systematic fill bug, not a coordinate slip. Measured at both
readings of all four labels:

| address | `H` | `round(H/0.9)` | sediment blocks | Δ |
|---|---|---|---|---|
| loess margin, metres | 80.492 m | 89 | **90** | +1 |
| loess margin, voxels | 2.281 m | 3 | **3** | 0 |
| summit, metres | 15.363 m | 17 | **18** | +1 |
| summit, voxels | 9.699 m | 11 | **12** | +1 |
| wave coast, metres | 8.760 m | 10 | **11** | +1 |
| wave coast, voxels | 2.852 m | 3 | **4** | +1 |
| barest land, metres | 0.168 m | 0 | **0** | 0 |
| barest land, voxels | 0.483 m | 1 | **1** | 0 |

The error is never negative. It is **0 or +1, everywhere**, and the +1 is not
slop — it is journal/0055's top-of-column remainder, stated in the block tier.
`h = floor(elev / 0.9)` guarantees there *is* ground in the surface voxel, so
that voxel is filled from its floor up to the real ground and gets **at least one
eighth**. A partial voxel is still a solid block, so a block scan counts it as
one whole voxel of pile. The contents know it is a fraction; blocks cannot say
so. **The generator never under-expresses; the block tier over-reads by exactly
the one partial voxel it cannot represent.**

And the loess margin at its own address expresses **90 blocks** against
journal/0055's reported **89 voxels of section**. That entry was right to the
digit, twice over.

The apparent "systematic shortfall" was one comparison made across the unit
boundary in both directions at once: `H` taken from the metres address, blocks
counted at the voxel address. Every site disagreed because every site was two
places.

### The tell: the error is proportional to how far from the origin you stand

Mid-investigation the integrator sampled a **second** live site — the
periglacial summit, voxel (−4586, −3207) — and found it *agreed* with
journal/0053: ~12 voxels of mudstone against a predicted 13. So the discrepancy
was **site-specific**, which he correctly read as evidence against "the world is
different" (a different world disagrees everywhere at once).

It is better than evidence against. It is the units error's own fingerprint.
Reading a metres label as a voxel address displaces you by exactly
`1/0.9 − 1 = 11.1 %` **of the coordinate's own magnitude**:

| station | label | as metres → voxel | as voxel → metres | displacement |
|---|---|---|---|---|
| loess margin | (82 346, 24 391) | (91 496, 27 101) | (74 111, 21 952) | **8.6 km** — 19 deep cells |
| periglacial summit | (−4 586, −3 206) | (−5 096, −3 562) | (−4 127, −2 886) | **0.6 km** — barely one deep cell |

At 82 km from the origin the misreading lands in a different geological story
entirely. At 4.6 km it lands next door, on terrain that looks the same and
carries a similar pile — `H` = 15.36 m at the true summit against 9.70 m at the
misread one, which quantizes to a number close enough to pass for agreement. A
genuine world divergence has no reason to scale with `|x|`. A units error can do
nothing else.

The integrator later withdrew the "site-specific" framing himself, on a third
sample, and was right to withdraw it: the displacement is not cleanly monotone in
the apparent shortfall, because what you find 10 km away is a *different column*,
not a scaled version of the one you meant. The direction of the inference held
anyway — but the thing that actually settled it was measuring blocks against
blocks, above, rather than reasoning about which mismatch pattern looked more
systematic. **When two parties are arguing from different instruments, build the
instrument that speaks the other one's units.**

### The client world *is* the probe world, and the proof is free

The hypothesis that mattered was the third one, so it deserves better than a
code read. The code read says: the client boots
`Authority::new_with(BENCH_SEED, 2, &GenOptions::default())`, which is
`Pregen::run_with(WorldParams { seed: 1337_i32 as u64, extent: Extent::Medium },
&DeepOverrides::default())`, and `Pregen::run` is *defined* as `run_with(params,
&DeepOverrides::default())`. Same seed, same extent, same overrides.

The measurement says something better, and by the end there were three of them.
The walker reported his surface heights at the voxels he actually stood on:
**y153**, **y148**, and — at the summit — **y1095**. The headless probe, built
from `Pregen::run` with no client in the loop, independently answers **surface
y 153**, **y 148** and **y 1095** at those same three voxels. Three columns,
three exact agreements, kilometres apart. **The world a player walks is the
world we measure.** No divergence, no correction to the ledger's substance —
only to the way it writes an address.

### And the mapping round-trips

The remaining suspect was `soil_depth_probe::idx_to_voxel`, the inverse of
`DeepField::deep_coords` that names the coordinates of the *scanned* sites (6
and 7, the world's thickest and thinnest regolith). If it did not round-trip,
every scanned coordinate in journal/0053's table would point somewhere other
than the cell whose numbers are printed beside it — and the live tour has been
driven off that table.

It round-trips exactly. Over **37 597 deep cells**, `regolith_at_voxel` at each
cell's own computed voxel address reads back that cell's own `H` with **0
disagreements, worst |ΔH| = 0.000000 m** — ground truth by construction, since a
cell centre must return its own node value. The mapping is not the defect and
journal/0053's scanned coordinates are sound. Only the *unit* on the printed
number was ever ambiguous.

The lesson goes in `corrections.md` #28, because the falsified claim here is
mine (the suspicion), and because the *mechanism* — a coordinate with no unit on
it in a codebase that carries three coordinate systems (metres, voxels, chunks)
— will recur unless it is written down.

## Part two — the skin that quantized to the chunk grid

The user's screenshot (`journal/assets/0056-surface-quantized-per-chunk.png`):
the ground in 28.8 m patches of salmon and cream with hard rectilinear edges,
and dig one voxel down and the same materials are a natural gradient.

That silhouette is a known ghost. corrections #6 named it once already — the
**chunk-line family cutover** — and journal/0011 built the 3c-2 boundary dither
specifically to kill it: bilinear interpolation of the per-chunk-column
selection hash to a voxel's fractional position inside its chunk, so a class's
member partition wanders like a facies contact instead of snapping to the chunk
grid. The buried fill has used it ever since, per voxel column, via
`dithered_member`.

journal/0055 folded the surface voxel into the record and, in doing so, gave it
its own member resolution — in `column`, *outside* the shared kernel:

```rust
let u = interp_select_draw(self.seed, SALT_GEO_SELECT, 4, cx, cz, 0.5, 0.5);
```

`0.5, 0.5` — **the chunk's centre, once per chunk**, memoized in a `member_of`
list and reused for all 1 024 voxel columns of the footprint. So the whole
32×32 patch got one member. Mudstone and siltstone are both `clastic/fine`, they
share a `block_twin`, and they differ in albedo — hence *same block, different
colour, chunk-shaped*. The block-level fill contract never noticed, because
nothing about the block was wrong.

The fix is one line of position, moved into the right function.

### Why it goes in the shared kernel

`surface_sample` is the kernel `column` (the ground) and `coarse_surface` (the
horizon) both call, deliberately, at different strides — ARCHITECTURE.md § One
world-answer surface. journal/0055 put the *class* consult inside it for exactly
that reason: derive the skinning only in `column` and the ground turns to
mudstone while the horizon stays green, and the LOD boundary becomes a visible
lie. The member resolution had been left outside, and being outside is *how* it
ended up per-chunk — `column` is the only caller that has a chunk.

So the member moves in with the class. `SurfaceSample` gains
`member: Option<GeoMemberIdx>`, the kernel draws it at the voxel's own
`(fx, fz)`, and `column` shrinks to just building the contents. Same salt, same
tag, same interpolated field — the chunk centre the old code sampled is still
one point of it.

### Why this is safe, and why it is *not* the mixed-voxel case

The fill contract is `block == classify(contents)`. Dithering the member changes
the contents. It cannot change the block, because **every member of a vanilla
class shares a `block_twin`** — mudstone and siltstone both classify to
`Mudstone`. That is the same property `class_block` has relied on since
journal/0055, and it is why `coarse_surface` can answer a block without ever
resolving a member.

It is worth being precise about how this differs from journal/0055's judgment
call 2, which deliberately withheld the dither from **mixed** voxels. There the
hazard is real: `classify` breaks a 4–4 tie between two classes on the lower
material id, so a within-class swap in a mixed voxel *can* flip which class wins
the tie, and the block would stop matching the contents. A surface voxel holds
one member's eighths and nothing else. There is no cross-class tie to flip.
**That case is untouched and stays untouched.**

`collapse::tests::surface_member_is_dithered_not_chunk_quantized` asserts both
halves, because either alone is satisfiable by a broken kernel: that a healthy
fraction of chunk footprints express more than one member (the dither is live),
and that every filled surface column's block is the `block_twin` of the member
the dither chose (the block did not move with it).

### The numbers

169 chunk columns around the loess margin, distinct surface members per 32×32
footprint:

| | 1 member | 2 members |
|---|---|---|
| before | **169 (100 %)** | 0 |
| after | 22 (13 %) | **147 (87 %)** |

The 22 that still read one member are not a failure: a footprint whose class has
one member, or that sits well inside one member's share of the interpolated
draw, legitimately reads one member. That is a facies contact being 28.8 m away,
not a facies contact snapped to a grid line.

**Far-field cost — the thing that could have made this not worth it.**
`coarse_surface` now runs one addressed draw and one `select` per sample it
never ran before. Over 48 400 samples at a 907-voxel stride, seed 1337 Medium:
**26.7 → 27.1 µs/sample, +1.5 %.**

Then honesty about that number, because it is small enough to deserve it. Four
subsequent runs of the identical binary on this machine gave 27.1, 27.5, 27.9 and
28.2 — a run-to-run spread of **±1 µs, comparable to the delta itself**. So the
defensible claim is not "+1.5 %"; it is **"the added draw is at or below this
machine's measurement noise, and certainly under 5 %."** That is still the
answer the decision needed — the far field can afford to stop lying about the
chunk grid — but it is not a number anyone should quote to two significant
figures. (journal/0055's 22.1 → 22.9 came from `mixture_cost_probe` on a colder
path and is not comparable to these absolutes; only the before/after within one
probe is.)

## What was deliberately not done

**The surface voxel is still a single member, not a mixture**, even where the
record's top 0.9 m is genuinely two classes. Making it mixed is a separate and
costlier question — it may force `coarse_surface` to run the eighth allocation,
trading either exact near/far agreement or far-field cost — and it was scoped
out of this slice. The honest note is that the "too expensive" claim behind that
scoping has still **never been measured**; the +1.5 % measured here for one draw
is weak evidence that a full allocation would also be affordable, and someone
should measure it rather than inherit the assumption.

## The goldens moved

`tests/contents_contract.rs`: the material sidecar changes wherever a surface
voxel's member changed, which is most of the world's surface. The blocks do
**not** move — that is the within-class invariance, asserted rather than hoped
— and the mixture table moves because the new member combinations intern new
states.

| | blocks | materials | table |
|---|---|---|---|
| 0x0D5EED572026 medium | `385D…DC40` **unchanged** | `FA60…98C9` → `CDBA…FC2C` | `AA01…7AF5` → `D8D5…F931` |
| 0x539 medium | `E6E4…5B42` **unchanged** | `4206…1196` → `3C69…8C8E` | `E70C…18CB` → `69F5…597B` |
| 0xC11A7E2026 small | unchanged | unchanged | unchanged |

Read that table as the argument rather than as bookkeeping. **The blocks did not
move**, in either recorded world — that is the within-class invariance,
independently confirmed by a fingerprint over 80 chunks that knows nothing about
the reasoning. The **materials** moved, because the sidecar is where the member
lives. The **table** moved, because single-member partial fills at the new
members intern combinations the per-chunk pick could not construct. And the
**Small world did not move at all**: it runs no deep-time record, so it has no
record-derived surface member to dither. Same control as journal/0055, still
holding.

`block_equals_classify_of_contents` passed unchanged, and its absent-contents
exception did not widen: 3 725 458 contents-bearing voxels of 7 864 320, and
`{Air, Stone}` — air, and unrecorded basement.

This entry is that authorization.
