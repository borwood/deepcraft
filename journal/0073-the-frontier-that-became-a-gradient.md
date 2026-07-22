# 0073 — the frontier that became a gradient

*Shape-teacher #2 of the threshold-quantization migration (audit § B1). The
plurality verdict at `collapse.rs::surface_class` becomes a class-membership
dither: the surface class is **drawn** from the record's top-window metre
shares, not awarded to the winner. And a measured wrong turn on the way — the
audit prescribed a per-voxel white-noise draw; the far field, which
point-samples this class at a coarse stride, doubled its mesh, and the fix was
to make the draw spatially coherent (the move the codebase had already made once,
for the member dither).*

> blogworthy — lens 4 (respect for earth) × lens 3 (deepsim reflexions): a soil
> frontier in nature is not a fence, it is a zone where two soils interfinger and
> the fraction of each shifts across it. The engine drew it as a fence because it
> took a vote. And lens 2 (procgen against priors): the "obvious" unbiased dither
> (white noise) is *wrong for a signal a coarse consumer point-samples* — a
> classic sampling-theory trap the far-field mesh cost caught for us.

## The exhibit

From altitude, looking at the 10 km horizon, the far field was a **checkerboard**
— sharply bounded dark-material regions, axis-aligned, edges razor-straight for
hundreds of metres, tiling to the skyline (`0070-lit-10km-horizon-vantage`, and
the mechanism-clean single square in `0070-fb-high-vantage-patch-check`). The
FF2b agent filed it in ROADMAP Observed and named its heir correctly: this is the
surface-material sibling of the `regolith_at_voxel` mosaic — a simulation-
resolution edge reaching the eye as an analytic boundary, the thing S-4 forbids.

The square is 460 m on a side because that is the deep-time cell. Here is the
machine that drew it. `surface_class` (the shared near/far kernel's class
consult) walked the recorded column's topmost 0.9 m, summed how many metres each
**content class** held in that window, and returned **the class with the most
metres** — a plurality. `record_at_voxel` samples NEAREST, so that plurality is
constant across the whole 460 m cell; the cell next door, whose window was a
hair's-breadth different mix, voted a *different* winner and painted itself a
different colour uniformly. Two solid fields meeting at a cell edge: a fence.

The user's S-4 sharpening (walk-0071): *smoothing a verdict preserves the shape
of the cell that voted it.* You cannot feather this square away — the thing with
the square shape is the **decision**, taken once per cell from a bulk summary.
The audit classified the site **B**: the cause (a strata unit list) is
non-interpolable, so the legal cure is not "threshold late on the continuous
cause" (there is none) but **dither membership**.

## The move

Same window, same metre-shares. Instead of `argmax`, an **addressed draw**:

```rust
by_class.sort_unstable_by(|a, b| a.0.cmp(b.0));            // canonical order
let u = interp_select_draw(self.seed, SALT_GEO_CLASS, 0, ccx, ccz, fx, fz);
return draw_class(&by_class, acc, u);                       // inverse-CDF sample
```

`draw_class` lays the class shares end to end on `[0, 1)` and returns the one the
draw lands in — inverse-CDF sampling: for a *uniform* draw, `P(class i) =
share_i` exactly (unit-tested to 1e-2, the same proof shape as
`fill::allocation_is_unbiased_over_the_draw`). A window that is 55 % coarse-
clastic and 45 % fine now skins the two in shifting proportion, so the fence
between two cells that voted different winners dissolves into an interfingered
gradient. The two solid fields become one zone whose local mix tracks the
underlying share.

Design questions I owned, and how they resolved:

**Address — the voxel column.** The surface is one voxel per column, and the
draw is addressed at the world column `(vx, vz)`, the argument the shared
`surface_sample` kernel feeds both the near ground *and* the far horizon. So near
and far inherit the **identical** class by construction:
`coarse_surface_matches_near_column_surface_block` still passes **exactly**, not
merely in distribution. One world answer, unchanged. Any address the far path
could not reproduce would have re-opened the LOD seam the instant it dithered.

**Tie handling falls out.** The retired plurality needed a `c < bc` tie-break to
be order-independent; the dither needs none. The only determinism obligation is
that the CDF be built in a canonical order, so I sort classes by id before
drawing — the same key the old tie-break used, one layer down.

**The window is unchanged** (top 0.9 m, the `acc >= voxel_m/2` bare-rock gate). I
changed the verdict mechanism, nothing else — which keeps the blast radius
legible: `surface_class` feeds *only* the surface skin, not erosion, not the
strata record, so terrain geometry and every buried voxel are byte-untouched;
only the `vy == h` voxel per column moved.

## The wrong turn: white noise doubled the far field

The audit was specific — dither "with a position-addressed draw **à la
`SALT_GEO_FILL`**," i.e. `draw_f64(&[seed, salt, vx, vz])`, per-voxel white
noise. I built exactly that first. It passed every dc-worldgen test: unbiased
(256/256 deep cells split), near/far exact, invariant green, goldens moved as
predicted. Then the **dc-client** far-field cost test failed:

```
farmesh::tests::horizon_sweep_measures_the_far_field_cost:
    per-tile mesh unexpectedly large at 1.2 km
```

Measured: the 1.2 km far-tile mesh had **doubled, 21.5 → 43.5 MiB** (242 KB/tile,
over the 200 KB budget). The mechanism is a sampling-theory classic. The far
field does not render every voxel — it **point-samples** `coarse_surface` (hence
`surface_class`) at a wide, level-dependent stride. Point-sampling white noise at
a coarse stride is aliasing: you don't get the local *mean* of the class field,
you get a fresh coin flip per far cell. And the shares are **constant across a
whole 460 m deep cell** (nearest record), so within one cell the far field became
a random mosaic of classes drawn from one distribution — pure speckle, at the
coarse scale, which the stepped mesher cannot merge. Every adjacent far cell
differing in block means a face it cannot coalesce. Hence double the mesh.

Runtime perf is sacred (CLAUDE.md, user 2026-07-22: *"our game is getting slow"*).
Doubling far-field memory to fix a colour seam is not a trade I get to make
silently. So I looked at what the codebase *already knew*.

The surface **member** dither (journal/0058) had faced the sibling of this
problem — its first cut drew one member per 32×32 chunk and the world quantized
into 28.8 m albedo patches — and its fix was **not** white noise. It was
`interp_select_draw`: a bilinear interpolation of four chunk-**corner** hashes,
so the selection field is C0-continuous and a class's member contact *wanders*
like a facies boundary instead of snapping to a grid, coherent over a chunk. The
class dither wants the same coherence for the same reason, plus one the member
dither never had to care about: the member dither can afford white noise because
it never moves the block (all members of a class share a block twin), so the far
*mesh* never saw it. The class dither moves the block, so the far mesh is
exactly what it stresses.

So the class draw now reads the **same coherent bilinear field, at a distinct
salt** (`SALT_GEO_CLASS`). The class forms sub-chunk patches whose *composition*
shifts across the 460 m frontier — the checkerboard dissolves into interfingering
at ~30 m granularity, which meshes cheaply at every scale. Measured after the
switch: 1.2 km far mesh **24.4 MiB** (+13 % over the pre-change 21.5, versus
white noise's +100 %), and the far-cost test is green across 1.2 / 3 / 5 / 10 km.

The cost of coherence, stated honestly: the bilinear value is not uniformly
distributed (four averaged uniforms bunch toward 0.5), so a 55/45 window renders
a few points closer to 50/50 than its true share. This is the **identical bias
the member dither already lives with**, and it flattens mixes slightly rather
than distorting which classes appear. The genuinely unbiased end-state — the far
field *summarising the share vector* rather than point-sampling a per-voxel draw
— is the `CoarseField<T>` extraction's job (audit Part 2), where the octree doc
already specifies near/far agreement becomes **statistical** by design. Until
then, coherent-and-slightly-biased beats unbiased-and-mesh-doubling. **This is a
LOUD PLEA / NEEDS RATIFICATION** — it deviates from the audit's white-noise
prescription, with the measured far cost as the argument.

## What held

`block == classify(contents)` is untouched (`block_equals_classify_of_contents`
green; absent-contents blocks Air and Stone only). Block, member and contents all
still flow from the one drawn class, together. And because `surface_class` feeds
only the skin, terrain geometry did not move — the block fingerprint moved on
surface voxels, the material fingerprint on surface contents, nothing else.

## What this cost, in goldens

Both Medium worlds moved on all three hashes; the Small world held.

| seed | before (outcrop-rule goldens) → after (B1 coherent dither) |
|---|---|
| `0x0D5E_ED57_2026` medium | blocks `18BD…C969` → `41A4…9046`; materials `2FB1…5986` → `224E…E14E`; table `35AE…D8CD` → `5961…3C89` |
| `0x539` medium | blocks `9A74…A767` → `0A3C…646B`; materials `505A…DEFE` → `B9A6…B19B`; table `D8C3…3081` → `FC52…E8B9` |
| `0x00C1_1A7E_2026` small | **unchanged** (`5B85…2E1D` / `3222…0F75` / `D0A3…310C`) |

The Small stillness was stated as a **hypothesis** before the run (corrections
#38: a control that never moves is evidence only about the axes exercised). It
held, and the mechanism is exactly #38's: the contents-contract sampler places
Small's chunks mostly in the border wilds / basement, so its sampled columns
never surface a deep-record class and the dither path is never exercised there.
Small is blind to *this* axis (surface-class expression at in-grid, record-
surfacing columns), which its sampled columns are not — not because it has no
deep record.

## Frontier statistics

- **Before (plurality):** across any deep-cell boundary the surface class is a
  **step** — 100 % class A up to the 460 m grid line, 100 % class B past it. The
  share "curve" is a cliff on the grid.
- **After (coherent dither):** within each cell the class is an interfingered
  patchwork whose areal fraction tracks that cell's shares (with the bilinear
  bias above); across a boundary the fraction shifts from cell-A's mix to
  cell-B's mix with no cliff. Quantified by the liveness guard
  `surface_class_dither_splits_multiclass_deep_cells`: **139 of 256** sampled
  deep cells surface more than one geology class *within a single 32-voxel
  window* — where the plurality gives **exactly 0** (a deep cell's shares are
  constant, so plurality → one class → one block for the whole cell). The draw
  itself is share-exact to 1e-2 for a uniform input
  (`draw_class_is_unbiased_over_the_draw`); the coherent source adds the stated
  bias on top.

## Perf

- **Collapse path (`surface_class` draw):** the far-field derivation probe reads
  **2.513 µs/column** over 160 000 columns (well within the test's <3 s / 160 k
  ceiling; the added op per call is one bilinear draw + a ≤6-element sort,
  sub-percent of the per-column cost). No collapse-time regression.
- **Far-tile mesh (the one that bit):** 1.2 km **21.5 → 24.4 MiB (+13 %)**; the
  full sweep 1.2/3/5/10 km = 24.4 / 38.7 / 74.1 / 227.3 MiB, per-tile cost and
  per-frame meshing budget both within `horizon_sweep`'s asserts. (White noise
  was 43.5 MiB at 1.2 km, over budget — rejected.) The +13 % is the real cost of
  the class now varying *within* a deep cell where before it was uniform; it buys
  the checkerboard's dissolution and is the floor of what any honest sub-cell
  gradient costs the far mesh.

## The move-B API lesson for `CoarseField<T>`

Both shape-teachers now exist for the type extraction to read. B1 teaches
`sample_dithered` — including its own wrong turn:

- **The payload is a share vector, not a winning class.** The categorical case is
  that a coarse cell holds a **share vector over `T`** (metres per class), and
  the dithered read inverse-CDF-samples it (`draw_class`). Storing "the winning
  class" is the plurality bug frozen into a type. A node stratum's move-B payload
  is a `ShareVec<Class>`, reduced child→parent by summing metres.
- **`sample_dithered` must take a coherence scale, not just an address.** This is
  the lesson white noise cost us. `sample_dithered(pos, seed, salt)` with a
  per-position white-noise draw is only correct for a consumer that reads at the
  field's own resolution; a consumer that point-samples at a *coarser* stride
  aliases it. The type must either (a) carry a coherence length the draw
  interpolates over (so coarse consumers see the low-frequency content), or —
  the genuinely right answer — (b) offer a **`summarize(region) -> ShareVec`**
  read for coarse consumers distinct from the fine `sample_dithered`, so the far
  field asks for *the mix over its cell* and dithers (or renders dominant) at its
  own resolution, and near/far agreement becomes statistical rather than exact.
  The shared-kernel exact-agreement B1 preserves is a *near-field* virtue; at the
  far field it is what forced white noise's alias into the mesh.
- **Address granularity is load-bearing, not incidental.** Near/far agree exactly
  only because both call the same `pos`-addressed draw; the type must take the
  fine world position as the address and nothing else for the fine read.
- **`fill.rs` taught the unbiasedness proof shape.** `allocate` and `draw_class`
  are the same inverse-CDF-over-an-addressed-uniform, and their tests are the
  same test. But B1 adds: unbiasedness is a property of the *draw*, and the
  *source* of the draw (white noise vs interpolated field) is a separate axis the
  type must expose, because it trades bias against a consumer's sampling rate.

What A1 (`Interpolable`/`sample`) and B1 (`Member`/`sample_dithered`) share: the
coarse producer never hands the fine consumer a raw cell verdict. A1 blends the
cause and thresholds it late; B1 dithers membership. Neither can express "paint
this cell's winner across the span" — the square the migration exists to make
inexpressible.

## The screenshots

Same seed (1337 Medium), same fixed sun, same high vantage as the 0070 set — the
valid before/after CLAUDE.md licenses. `--fullbright` is the instrument: this is
a material-colour question, so the pure-data control is the one that can see it.

- **`0073-fb-high-vantage-patch-check`** vs `0070-fb-high-vantage-patch-check`:
  **the checkerboard is gone.** Where 0070 showed "a sharply bounded grey-
  material square at ~deep-cell scale, edges razor-straight for hundreds of
  metres," the same dark material (mudstone) now forms **ragged, organic,
  interfingered blobs** — amoeba-edged patches at ~30 m granularity with no
  axis-aligned boundary anywhere. The frontier between the dark material and the
  terracotta ground reads as an interfingered gradient, exactly the S-4 cure.
- **`0073-fb-10km-horizon-vantage`** and **`0073-lit-10km-horizon-vantage`** vs
  `0070-lit-10km-horizon-vantage`: at the 10 km scale the vista that was "a
  checkerboard of them to the skyline" is now a **fine mottled mix** — dark
  interfingered patches, terracotta dominant, and scattered cream (sandstone)
  specks that are the minority-class dither expressing a class the plurality
  would have erased. It blends toward tonal variation at the hazed horizon
  rather than tiling into squares.

**Verdict: the frontier reads as a gradient, not squares — acceptance met.**

**Other appearance deltas, flagged:** (1) The scattered cream sandstone specks
are *new* — the minority class now surfaces where a plurality suppressed it; this
is the dither working, not a defect. (2) Small rectangular far-voxels persist at
the coarse LOD scale (14.4 m coarse voxels at 10 km) — that is the far field's
inherent voxelisation, present in 0070 too, unrelated to the class frontier.
(3) No geometry/shape change (terrain heights are byte-identical), no new holes,
no seam at the near/far boundary.
