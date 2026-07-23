# 0074 — the summary that stopped being the author

> blogworthy — lens 3 (deepsim reflexions): the marquee case of "a summary is
> not an authority." For a year the world's surface *material* was decided by a
> function that existed only because the far renderer could not afford the real
> answer — and that cheap answer quietly became the definition of what the ground
> is made of. This slice deletes the definition and keeps the summary, held now
> to a tested agreement instead of a structural coincidence. The disappearing-
> consumer test (`if the far field vanished tomorrow, would this code still
> exist?`) finally answers *yes*, because the surface IS the record's top span.

## The violation, in one sentence

`collapse.rs::surface_sample` computed *what the world is skinned with* by
consulting the **deep-time record's top-0.9 m window** (`surface_class` → the B1
membership dither `draw_class` → a member of the drawn class). The near ground
and the far horizon both read this shared kernel, so they agreed **by
construction** (journal/0055's structural guarantee). But that kernel existed for
one reason: the far field point-samples the surface at a wide stride and cannot
afford to run the strata passes. A far-field summarization need had become the
authority for the world's surface block — spines § S-3's marquee entry, an A-1
stand-in-becomes-the-definition caught in the wild.

Meanwhile every *buried* voxel already had a real authority: the recorded strata
column (`col.strata`), sliced into voxel spans by `crate::fill::ColumnFill` and
quantized once by addressed stochastic rounding (journal/0055). The surface voxel
was the one voxel in the column that did **not** route through it. It was a
parallel computation of "what is the surface made of," reading a *different*
record (the raw 460 m deep units, not the post-strata-pass `col.strata`) by a
*different* mechanism (a class draw, not the fill's own allocation).

## The move

The near surface voxel now **is** the record's top span, through the same fill
machinery as every other voxel. `ColumnFill::build(&col.strata)` slices the
recorded column top-down; its topmost span (`plan(1)`) is the surface voxel,
expressed as the per-column partial — the top-of-column remainder `n =
ceil((elev − h·0.9)/0.9 · 8)` eighths that journal/0055 already computed. A
`Single` top span keeps the per-voxel-column member dither (`dithered_member`,
journal/0058) the buried single voxels use; a `Mixed` top span allocates its
eighths by the same addressed draw, scaled to the partial. `surface_voxel_
contents` is nine lines, and it calls nothing the buried path did not already
call.

The one structural consequence worth stating: **the buried column shifted down
one record span**. Before, `plan(1)` (the record's top 0.9 m) landed on the voxel
*below* the surface (`vy = h−1`), and the surface was a separate deep-record cap
on top — the record sat one voxel too low, papered over at the surface. Now
`plan(1)` is the surface itself and the first buried voxel reads `plan(2)`. The
record reaches the surface it always should have. Heights are byte-identical
(they come from the elevation lattice, untouched); only which voxel expresses
which span moved.

`ColumnFill::plan()` kept its signature — the shift is `plan(depth + 1)` at the
two buried call sites — so every external caller (the coal probes,
`distribution_fill`) is untouched.

## The fate of `draw_class` — it survives, re-homed

The brief asked whether routing the surface through `ColumnFill` makes B1's
`draw_class` redundant. It does **not**, and the reason is the whole shape of the
slice: **`draw_class` was never the near path's tool once the surface routes
through the fill — it is the far field's.** `coarse_surface` still calls
`surface_sample` → `surface_class` → `draw_class` to summarize the deep record's
top-window class into a cheap `(height, block)` for the horizon, because it
cannot afford to build a `StrataRec` and run `ColumnFill` at a 200-column-wide
stride. So `draw_class` is not redundant with the fill's own draw; the two answer
different questions for different registers:

- **near (the authority):** `ColumnFill` of `col.strata`, one draw per voxel.
- **far (the summary):** `surface_class`/`draw_class` of the deep record, one
  class draw per point-sample.

Retiring `draw_class` would leave the far field with no cheap surface answer.
Keeping it as the *authority* is what the slice deletes. So it stays, typed and
documented as a summary, and held to a test that it **agrees** with the
expression rather than a structural guarantee that it **is** the expression. This
is one-machinery-per-register (A-4 respected: the near field has exactly one
expression machinery, the fill), not one-machinery-for-everything. **`draw_class`
was not deleted; the diff shrinkage is the parallel `surface_fill`/`surface_
member` data structures and the near path's whole dependence on `surface_class`.**

## The guarantee that was replaced

journal/0055 gave near/far agreement *for free*: same kernel, so the ground and
the horizon could not disagree — no LOD lie possible. That guarantee is gone. In
its place is the S-7 statistical agreement register the octree node contract
sanctions (docs/design/octree-substrate.md; FF2b's +0.938-within-1 precedent,
journal/0070). Measured, over the two Medium worlds (seeds `0x0D5EED572026` and
`0x539`), on the **geology-surfacing** columns only (a shared Dirt/Stone fallback
is not agreement about anything):

```
near/coarse surface agreement: 317436/346112 = 0.9171
```

**91.71 % of land columns that surface a geology block agree between the ground
expression and the far summary.** The ~8 % that disagree are exactly the two
sources the two paths differ by, and both are named rather than hidden:

1. **The veneer.** The near path routes the recent clastic fan (`clastic_pass`'s
   fluvial term) on top of the deep history; the far window sees only the deep
   record. Where a fluvial fan skins a column its own colour, near and far
   legitimately differ — the near answer is *more* correct (it is the actual top
   of the deposited column), the far answer is the cheap one.
2. **The coherent-source bias.** The far class draw carries B1's coherent-source
   bias — **majority-amplifying**, per corrections #39 (the extraction merge
   corrected the sign: the bilinear source over-weights the majority class, it
   does not pull toward 50/50); the near path draws no class at all. That sign is
   *favourable* to this agreement — the far summary over-picks the same dominant
   class the near ground most often expresses — so it is not where the ~8 %
   disagreement comes from; the veneer (1) is.

The reworked test (`coarse_surface_agrees_with_the_near_column_surface`) asserts a
floor of **0.88** — below the measured 0.9171 with margin, stated as a hypothesis
(corrections #38), and strong enough that a real drift between horizon and ground
trips it loudly. Heights still agree **exactly** (`coarse_surface_matches_near_
column_height` unchanged) — only the *material* register went statistical, which
is precisely where the deliberate unbiased quantization lives (S-7).

## The perimeter, and why it did not move (the cake)

The user's cake observation (spines § 4 carve-out 1): under B1's coherent draw,
minority-class blobs *guillotine* at the 460 m cell perimeter — the noise source
is continuous but the shares it thresholds are nearest-per-cell, so a minority
phase dies where the neighbour's nearest share is zero (chocolate swirls hard-cut
at the slice edge of vanilla cake). This slice was checked against that signature
and is **neutral** to it:

- The **far** field (`coarse_surface`) is byte-for-byte unchanged — it still
  reads `surface_class`/`draw_class`/the coherent source. The perimeter signature
  the user calibrated their eye on at the 10 km horizon is exactly as it was.
- The **near** ground now reads `col.strata`, whose *classes* come from
  `deposit_deep_history` reading the same nearest-per-460 m deep units — so the
  near surface is **also** nearest-per-cell in its class shares (its sub-cell
  variation is the member dither, block-invariant, plus the sub-cell fluvial
  veneer). The near path neither inherits a *new* guillotine nor cures the
  existing one. The cure (boundary source-cell membership dither) stays assigned
  to the `CoarseField` extraction, untouched here as directed.

## The goldens moved, and Small proves the mechanism

`contents_contract` — both Medium worlds moved on **all three** hashes (blocks,
materials, table):

| seed | blocks | materials | table |
|---|---|---|---|
| `0x0D5EED572026` medium | `8A55…66BF` → `4A36…3999` | `A0AA…0D7C` → `ECBD…B0F3` | `93DE…D6F4` → `E025…BBBC` |
| `0x539` medium | `08A9…BD27` → `421C…E4F7` | `6D5E…84AC` → `7A26…C6C4` | `4A1F…15AF` → `C3B9…7030` |
| `0x00C1_1A7E_2026` small | **unchanged** | **unchanged** | **unchanged** |

Blocks moved because the surface block is now `classify` of the top-span partial
(not the drawn deep-record class) and every buried voxel shifted one span.
Materials moved for the same reasons. The **table** moved because the surface
voxel can now be a genuinely *mixed* partial — a mixture state the single-member
surface path could never construct.

**Small did not move, and that is the mechanism proof (corrections #38 — proven,
not celebrated).** The surface-branch removal only re-routes columns that *have* a
strata record. Small's `contents_contract` sample chunks carry **no** record
(empty `col.strata`): `ColumnFill` is empty, so `plan(1)` is `None` → the surface
keeps its unchanged year-zero fallback block, and the buried column falls to the
unchanged legacy soil band. A record-less column is byte-identical under this
slice **by construction**. This is the same blind spot journal/0073's Small
stillness had, generalized: Small is blind to *any* record-expression axis at its
sampled sites, not just the class dither. `block_equals_classify_of_contents`
still passes with absent-contents blocks limited to Air and Stone.

## The tests whose meaning changed (old → new claim)

- `coarse_surface_matches_near_column_surface_block` → **`coarse_surface_agrees_
  with_the_near_column_surface`**. Old claim: near and far are **exactly equal**
  (shared kernel). New claim: near (the `ColumnFill` authority) and far (the
  `coarse_surface` summary) **statistically agree** ≥ 0.88 on geology-surfacing
  columns; measured 0.9171. This is the journal/0055 structural guarantee retired
  and replaced by the S-7 register.
- `surface_member_is_dithered_not_chunk_quantized` → **`surface_voxel_routes_
  through_columnfill_per_voxel`**. Old claim: the near surface *member* (from the
  deep-record class draw) is dithered per voxel, not per chunk. New claim: the
  near surface *routes through `ColumnFill`* — `block == classify` of its top-span
  contents (the S-3 routing proof at the surface voxel), and a `Mixed` top span
  varies the surface contents per position (the shared per-voxel allocation).
  Single top spans turned out **vanishingly rare** post-0055 (the top 0.9 m almost
  always straddles a contact), so the test follows the common Mixed case.
- `surface_class_dither_splits_multiclass_deep_cells` → **`far_surface_class_
  dither_splits_multiclass_deep_cells`**. Old claim: the *near* surface splits
  within a deep cell (draw_class live in the ground). New claim: the *far* summary
  splits within a deep cell (draw_class live in `coarse_surface`, its new and only
  home) — measured 133 of 256 sampled cells surface >1 geology class where the
  retired plurality gives exactly 0.

## Perf

The near path **dropped** the per-column `surface_class` walk (1024× per chunk: a
deep-record access + a walk over the top units + `draw_class` + a member select)
and **added** the per-column surface derivation from `ColumnFill` (a
`dithered_member` or one `allocate_partial`) plus one extra `ColumnFill::build`
per column-collapse. Measured on a warm Medium generator, `generate_chunk_with_
materials` over a 25×25×3 chunk block (`examples/collapse_timing.rs`):

```
before (main collapse.rs):  876.4 µs/chunk
after  (surface-branch removal): 784.6 µs/chunk   (−91.8 µs, −10.5 %)
```

1875 chunks (25×25 columns × 3 chunk-ys), warm generator, Medium seed
`0x0D5EED572026`. **The change is a perf WIN**, and mechanistically so: the near
path stopped walking the deep record and drawing a class per voxel column (the
`surface_class` consult, 1024× per chunk-column) and now reads the record's top
span it already sliced. The two runs are genuinely distinct builds — the FNV
block goldens moved (`8A55…` → `4A36…`) and the before-run recompiled main's
`collapse.rs` (the `Compiling` line), and the ~92 µs gap self-validates against a
stale artifact (a stale before would time identically). The harness's
anti-elision checksum is an order-*independent* block sum, so it is coincidentally
invariant under a span shift — correctness is proven by the goldens and the suite,
not by it.

The clocks are the ratified ones: this is a chunk-load (runtime) cost, reported
per the runtime-perf convention; gen time is not the constraint.

## Files

- `dc-worldgen/src/collapse.rs`: the whole slice — `surface_sample` split into a
  cheap `surface_height` (near path) and the far summary; `surface_voxel_contents`
  added; `mixed_at` gained a partial-eighths parameter; `column`/`generate_chunk`/
  `material_ids` route the surface through `ColumnFill` and shift the buried span
  index; `ColumnRec::surface_fill` replaced by `surface_eighths`; three tests
  reworked.
- `dc-worldgen/tests/contents_contract.rs`: goldens re-baselined (both Medium),
  authorization block added.
- `dc-worldgen/examples/{soil_depth_probe,surface_dither_probe}.rs`: read the
  surface from `ColumnFill`/`surface_eighths` instead of the removed `surface_
  fill`.
- `dc-worldgen/examples/collapse_timing.rs`: new perf harness.
