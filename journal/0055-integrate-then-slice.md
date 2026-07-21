# 0055 — Integrate the column, then slice it

> blogworthy: the whole slice is one line of arithmetic in the wrong order.
> The generator computed `Σ round(tᵢ / 0.9)` where honesty required
> `round(Σ tᵢ / 0.9)`, and that single transposition was deleting
> three-quarters of the world's sediment — not because 0.9 m voxels are too
> coarse to hold it, but because the rounding question was being asked 379 times
> instead of once. Errors that should have cancelled compounded instead. The fix
> is not more resolution; it is *asking later*.

## The question that found it

The forms conversation reached this from the user's own framing, and the framing
was the diagnosis: *"going fractional first and preserving simulation output `H`,
then generating chunks as distributions of the cell distribution, means subvoxel
boundaries become mixed blocks and we get more partials remainders on the
surface... why is nothing mixed?"*

Nothing was mixed. In the entire world, the only heterogeneous voxel a player
could find was inside a placer fan — because that is the one place a pass
deliberately substituted a second material into a stratum's eighths. Everywhere
else, every voxel held eight eighths of one thing.

Three gates caused that, and the first is load-bearing: `deposit_deep_history`
rounded **each recorded unit independently** and dropped it if it did not reach a
whole voxel, carrying no remainder forward. So a contact between two beds always
landed exactly on a voxel boundary, by construction. Straddling was impossible.
The second and third gates (one event per voxel, one member per event) were
consequences.

## The measurement that made it urgent

journal/0053 had already found the sieve and quantified it at "~75 % of the
pile". Measured properly this time, over 44 265 subaerial deep cells of the
production Medium world:

```
recorded          : 4.753 m/cell mean
OLD Σround(tᵢ/0.9): 1.152 m/cell — 75.8% of the pile LOST
cells expressing NOTHING under the old rule: 21309 (48.1% of land)
```

**Nearly half the land in the world recorded a sediment column and expressed
none of it.** The tour's dune field is the extreme case and the clearest one:
379 recorded units summing to 7.99 m, averaging 0.021 m each, every single one
rounding to zero. Eight metres of sand, expressed as nothing.

## The rule

Metres survive to the voxel boundary. Quantization happens **once**, at contents
construction, from the units overlapping a voxel's own 0.9 m span.

`StrataEvent::thickness_vox: u8` became `thickness_m: f32`, and a new module
`crate::fill` owns the single quantization. The allocation, exactly:

1. Each overlapping material's true share in eighths is `8 · overlap / coverage`.
2. Every material takes its **guaranteed whole eighths** (`floor`).
3. The leftover — always exactly `8 − Σ floor` eighths — goes to the materials
   whose **fractional remainders win against one addressed draw**.

Step 3 is *systematic sampling* over the remainders: lay them end to end on a
line, offset by a single uniform `u ∈ [0,1)`, take every integer crossing. Two
properties fall out, and both are load-bearing. The crossings of a segment of
total length `L` under a `[0,1)` offset number **exactly** `L`, and `Σ remainders`
is exactly the integer leftover — so the voxel is always filled to exactly eight
eighths, with no clamping and no fixup pass. And `P(material i takes an extra
eighth) = remainder_i` — so it is **unbiased**.

That last property is the whole argument. Deterministic flooring is a *biased*
estimator: it always loses mass, and it loses it in the same place every time —
the 0.19-of-an-eighth lamina is deleted **everywhere**, forever. Stochastic
rounding trades bias for variance, which is the right trade for a record. The ash
band should exist *somewhere* rather than uniformly nowhere. This is the user's
own instinct — *"to remain honest we could dither the material eviction across
voxels... one will evict a different partial than its neighbor so there's a fair
distribution"* — generalized from the >8-materials tie case to the whole
allocation. There is no special case for the tie condition. It falls out.

All the arithmetic is fixed point (20 fractional bits per eighth), so "exactly
eight eighths" is an integer identity rather than a floating-point hope.

The draw is `draw_f64(seed, SALT_GEO_FILL, world voxel x, y, z)`. World position
only: not chunk order, not chunk `y`, not iteration order of any collection.
Classic error diffusion would give a visibly better dither and is **forbidden**,
because it is sequential — the same rule that governs every other draw in the
generator. `tests/distribution_fill.rs` proves it by generating the same chunks
forward and backward from two generators with different cache warmth.

## Wrong turn 1 — the cost of doing it honestly

The first shape of this was to fill each voxel from the raw recorded units. The
dune field has 379 of them in eight metres — about 43 competing per voxel, each
needing a member resolution. That is 172 hashes per voxel before anything is
built.

Two things fixed it. First, **aggregate by member before allocating**: 43
overlapping units collapse to three or four distinct members, because a class has
few members and adjacent beds keep picking the same one. Second, **run-length
coalesce adjacent units that resolve to the same member inside
`deposit_deep_history`**. That is lossless for expression — contents depend on
the member, not on how many recorder units contributed it — and it is *not* the
heir stubs.md § 12 filed ("amalgamate adjacent sub-voxel units in the record"),
which merged *unlike* beds and lost them. It merges only beds that would express
identically anyway. The dune field's 379 units become 355 events; the loess
margin's 354 become 170.

There is one deliberate omission and it is worth stating plainly: **the mixed
path does not run the per-voxel-column member dither.** It uses the event's
canonical member. Partly that is cost — up to eight member re-selections per
voxel is unaffordable — but the real reason is correctness. The dither re-picks
only *within* a class, and `classify` breaks a tie between two classes on the
lower material id; a dither that swapped mudstone for siltstone could therefore
flip which class wins a 4–4 tie, and the block would stop matching the contents.
Single-event voxels keep the dither, where it is provably block-invariant.

## Wrong turn 2 — the veneer took the placer with it

journal/0053 defined the clastic veneer's budget as `round(H/0.9) − expressed`.
The prediction was that honest expression would drive that to zero on its own.
It did — `0.00` voxels at all seven of journal/0053's named sites. The
subtraction is left in the code, written in metres now, because it is still the
honest statement of "what the record could not carry" and it still fires if a
class fails to resolve a member. Nothing was surgically deleted.

What that broke was not obvious in advance. With the residue at zero, the veneer
*is* the fluvial fan and nothing else — and the veneer was still rounding its
budget to whole voxels. A modest river's fan is a fraction of a voxel, so it
rounded to nothing, so no graded coarse body was deposited, so **every placer in
the world disappeared**. `placer_follows_the_sorted_gradient` went from green to
"no placer deposits found in the sample". The same rule fixed it: the veneer
keeps its metres too, and `crate::fill` quantizes them once with everything else.

That in turn forced a second thing. A thin fan is now a *mixed* voxel, and the
first cut of the mixed path dropped per-event ore riders. So the placer
substitution had to be carried into mixed voxels: the ore takes `min(grade,
allocated)` of its host event's own eighths, which is exactly the substitution
`contents_for_event` performs, scaled to whatever share of the voxel the host
won. Gold pans again, and now it pans out of a voxel that also holds the mud the
fan is cutting through — which it did not before.

## The surface, folded in

The slice was planned as buried strata only, with the surface voxel as a second
slice, because that voxel is where the year-zero veneer's block rule lives and
retiring it is user-owned. The user overruled the split mid-slice and was right:
*"fold them together, we don't have to have this problematic of deciding which
material to skin the world with when the record already says. we already wanted
the veneer gone."*

The constraint that shaped the implementation is `surface_sample` being the
kernel **shared** by the near ground and the far horizon, deliberately, so that
near and far are the same function at different strides (ARCHITECTURE.md § One
world-answer surface). Derive the record-skinning only in `column` and the ground
turns to mudstone while the horizon stays green — the LOD boundary becomes a
visible lie. So the record consult went *inside the kernel*:

- `surface_sample` reads the content class holding the most **metres** in the
  recorded column's topmost 0.9 m. Metres, not bed counts — the same `round Σ`
  discipline one level up. `coarse_surface` inherits it for free.
- `column` resolves that class to a member under its own formation context and
  builds the surface voxel's **contents** as partial fill; the block is
  `classify` of them. The two agree *by construction*, because every member of a
  class shares a block twin — the same property the buried fill has relied on
  since journal/0052.
- **The top-of-column remainder is expressed at last.** `h = floor(elev/0.9)`, so
  the surface voxel is filled from its floor up to the real ground:
  `ceil((elev − h·0.9)/0.9 · 8)` eighths, at least one. Loose material with no
  structure gets `StructureShape::None` — which is exactly
  `VoxelContents::is_loose_only`, journal/0010's dormant partial-height render,
  lit up by real data for the first time. Height does not move: the "at least
  one" is not a fudge, it is the statement that `floor` guarantees there *is*
  ground in that voxel, and rounding it to zero would silently delete a voxel of
  world.
- **Grass is not expressed at all** (ratification 4). The fallback vocabulary is
  Dirt/Stone; the Grass branch is gone from the kernel.

**The fallbacks, and why each is legitimate rather than a surviving stub.** The
**border wilds** keep the year-zero synthesis because no deep-time run exists out
there and there is no recorded cause to consult — stubs.md § Genesis, the one
place inventing a number is affirmed. **Subaqueous columns** keep it because
`clastic_pass` returns early below sea level, so there is no clastic record to
read; subaqueous sedimentation is the carbonate milestone. **Columns whose record
rounds under half a voxel** — journal/0053's 0.2 % bare-rock case — are skinned
by their **basement**, named the way `igneous_pass` names it from the column's
tectonic province, so near, far, and the buried record all say the same rock. The
world's barest land now reads **Granite**, not painted Dirt. Only a column with
neither a record nor a province falls all the way through, and that is absence of
a record, not a rule standing in for one.

## The numbers

**The sieve**, over 44 265 subaerial deep cells:

| | recorded | expressed | loss |
|---|---|---|---|
| old `Σ round(tᵢ/0.9)` | 4.753 m/cell | 1.152 m/cell | **−75.8 %**, one-way |
| new `round(Σ tᵢ/0.9)` | 4.753 m/cell | 4.745 m/cell | **−0.2 %** net, 0.227 m mean abs |

Cells expressing *nothing*: **21 309 (48.1 % of land) → only those whose whole
column rounds under half a voxel.**

**The dune field** (107 183, 9 672), the station that expressed zero: 379 units /
7.99 m → **9 voxel spans, all nine mixed**, from 355 coalesced events. **The loess
margin** (82 346, 24 391): 80.49 m of `H` → **89 voxels of section**, 188 spans,
76 of them mixed. The 8-voxel veneer cap that used to truncate 14.3 % of land is
simply not in the path any more.

**Mixture-table cost — the slice's main risk**, since dithering deliberately
makes neighbours differ. Over 128 sampled buried chunks, seed 1337 Medium:

| | before | after |
|---|---|---|
| distinct interned mixtures | 11 | **325** |
| mixture table bytes | 103 (9.4 B/entry) | **3 160** (9.7 B/entry) |
| distinct mixtures per chunk (min/med/max) | 1 / 4 / 6 | **1 / 25 / 54** |
| sidecar | 0.330 B/m³ | **0.621 B/m³** |
| distinct materials in sample | 7 | **10** |
| S8 combinatorial cap `C(k+8,8)−1` | 6 434 | **43 757** |
| observed as % of cap | 0.17 % | **0.74 %** |

The table grew **30×** — and it is three kilobytes, shared region-wide. The
sidecar grew **1.9×**, to 0.621 B/m³ against S8's 0.14–0.47 B/m³ for real deposit
chunks and its 0.94 B/m³ for a gradient *engineered* to maximize states. It is
**17.7× under** the 10.97 B/m³ raw dense material grid the whole scheme exists to
beat. S8's structural argument holds exactly as written: the eighth quantization
caps the state space combinatorially, and with ten materials in play we are using
under one percent of the cap. **This is affordable, and it is affordable for the
reason S8 predicted, not by luck.**

Two honest notes on that table. The *contents-bearing voxel count* is unchanged
in the buried sample (3.05 M both sides), so that 1.9× is purely bits-per-voxel —
it is the dither's cost, cleanly isolated, not "more of the world got recorded".
And chunk generation went **11.9 → 17.0 ms/chunk** (+43 %), which buys the mixed
allocation *and* the surface skinning; the ritual (`Pregen::run`) is unchanged at
~17.4 s, and `DeepField` is unchanged at 147.80 MB — this slice adds no pregen
state at all.

**What the world is skinned with**, 12 135 land samples at a 907-voxel stride,
through `coarse_surface` (the far-field kernel, so this is literally the horizon):

| before | after |
|---|---|
| Grass 81.8 % | Mudstone 91.4 % |
| Dirt 18.2 % | CarbonaceousMudstone 6.3 % |
| | Coal 2.0 % |
| | Peat 0.3 % |
| | Granite 0.02 % |

Far-field cost 22.1 → 22.9 µs/sample: the record consult is a grid index and a
walk over the top few units, and it is free.

## What a player sees

Standing on the ground, at journal/0053's named stations:

| site | surface block | surface fill | sediment above basement |
|---|---|---|---|
| 1 deflation basin | Mudstone (siltstone) | 5/8 | 12 vox / 10.66 m |
| 2 dune field | Mudstone (mudstone) | 7/8 | 9 vox / 7.99 m |
| 3 loess margin | Mudstone (siltstone) | 2/8 | 89 vox / 80.49 m |
| 4 periglacial summit | Mudstone (mudstone) | 8/8 | 17 vox / 15.36 m |
| 5 wave coast | CarbonaceousMudstone | 2/8 | 10 vox / 8.76 m |
| 7 barest land | **Granite** | 2/8 | 0 vox / 0.17 m |

The world is brown and grey. That is pre-ratified verbatim — *"it'll be a mostly
brown world for a bit"* — and the razor-straight grass/dirt frontier that made
journal/0049's walk hard to read is gone, because nothing is painting by
threshold any more.

In a cut face, the change is bigger than the colour. Before: bands of one
material, each an integer number of voxels tall, meeting on voxel lines. Now:
bands that end where the record says they end, with the boundary voxel holding
both materials in proportion, and neighbouring boundary voxels holding *different*
proportions — because the draw is per position. journal/0010's material dither
has been shipped and tested for weeks and rendered exactly one thing: placer
fans, because nothing else was ever mixed. It now renders every contact in the
world. The loess margin shows 76 mixed spans in one column.

Digging, the deflation basin and the dune field go from "a couple of voxels of
dirt on stone" to 9–12 voxels of genuinely interbedded fill, and the loess margin
gives 89 voxels of section before basement rather than 37. And at the barest
place on the map the shovel hits granite immediately — and the horizon behind it
says granite too, because both came out of the same function.

## The fidelity trace

**What Earth mechanism, at what tier.** Two, both at the "faithful in kind, not
in calibration" tier this project works at.

The first is *stratigraphic condensation*. Real sedimentary columns are mostly
thin beds; a metre of section is hundreds of events, and any finite sampling
resolution — a core, a hand specimen, a voxel — averages them. What a geologist
records at 0.9 m resolution *is* the composition of that interval, not the
thickest bed in it. That is precisely what the allocation now computes.

The second is *bioturbation and mixing* as the reason a mixed voxel is not a
lie: thin beds genuinely homogenize toward the surface, which is why real soil is
not laminated. journal/0053 used that to justify amalgamating the residue into
one body. The honest version is what shipped here — the beds mix *in place*,
each voxel carrying the proportions its own interval holds, rather than the whole
residue collapsing into one homogeneous mantle.

The thing that is *not* faithful, and is knowingly so: a mixed voxel loses the
internal **order** of what it mixes, so a contact renders as speckle rather than
as a banded contact. The user ratified that as non-blocking — *"it's lost order
information in presentation but it's far more honest than it was before and
presentation can be reconsidered later."*

## The goldens moved, twice, and they had to

`tests/contents_contract.rs` says: if a golden moves, the world moved, and that
is a bug until a journal entry says otherwise. This entry is that authorization,
for the largest move the goldens have taken.

| | blocks | materials | table |
|---|---|---|---|
| 0x0D5EED572026 medium | `5863…4D62` → `385D…DC40` | `9B67…BDBC` → `FA60…98C9` | `BD39…8067` → `AA01…7AF5` |
| 0x539 medium | `54CD…852A` → `E6E4…5B42` | `89F9…B38C` → `4206…1196` | `4123…82D8` → `E70C…18CB` |
| 0xC11A7E2026 small | unchanged | unchanged | unchanged |

The mixture table moved **for the first time** — journal/0053 left it untouched,
because that slice changed thickness, not composition. This one constructs states
the generator could not previously build. The **Small world did not move at all**,
which is the control: it runs no deep-time record, so there is nothing to slice
differently, and its sampled surfaces were already Dirt/Stone.

`block_equals_classify_of_contents` passed unchanged throughout — including on
the new mixed voxels and on the newly-contents-bearing surface voxel. And its
report shows the **absent-contents exception shrinking**, which is what that list
is supposed to only ever do:

```
before: absent-contents blocks {Air: 2041579, Stone: 2097283, Dirt: 81920}
after : absent-contents blocks {Air: 2041579, Stone: 2097283}
```

The 81 920 Dirt voxels were the surface-veneer stub. They carry real contents
now. What is left is air and unrecorded basement.

Three existing tests changed their claims, none of them by loosening a bar:

- `adding_a_member_diversifies_but_never_inflates_its_class` compared events
  positionally. Adding a member changes which adjacent runs coalesce, so the two
  sets legitimately hold different numbers of events describing the same column.
  It now compares the **class profile** — same classes, same order, same metres —
  which is the invariant that was always meant.
- The two coal-thickness helpers in `tests/organic.rs` summed per-event voxel
  counts. They now sum metres and divide **once**, which is this slice's own
  discipline applied to the test.

## Cost

No new pregen state: `DeepField` is byte-for-byte the same 147.80 MB and the
ritual is unchanged. `StrataEvent` went from `u8` to `f32` thickness (net +3 B
per event), and per-column event counts rose where records are thin and
interbedded — 53 events at the deflation basin, 170 at the loess margin, 355 at
the dune field, against the low tens before. That is bounded by the recorder's
own run-length merge and further cut by same-member coalescing, and the column
cache already evicts at 8 192 entries.

Probes: `cargo run --release -p dc-worldgen --example soil_depth_probe` (the
sieve, the sites, what each is skinned with) and `--example mixture_cost_probe`
(the table cost and the world-wide skin distribution — written against the public
API only, so it runs against the pre-slice tree for the "before" column).
