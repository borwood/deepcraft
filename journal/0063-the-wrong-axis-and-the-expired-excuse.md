# 0063 — The wrong axis, and the excuse that expired

*2026-07-22. Two fixes that look unrelated and are the same doctrine case twice:
a rule that answered the wrong question, and a reason-not-to-build that stopped
being true while nobody was watching it.*

> blogworthy: **"a decision not to build has a shelf life, and nothing watches
> it expire."** Stubs get an inventory, an heir, and a doctrine. A *deliberate
> non-implementation* gets a well-argued paragraph in a doc comment — and when
> the ground shifts under it, the paragraph is still perfectly argued. That is a
> nastier failure mode than a stub, because it reads as rigour.

## I. Coal was being promoted on the wrong axis

`DeepStrata::promote_coal` turned buried peat into coal when the seam was
**thick enough** — `thickness_m >= 0.4`. It had been that way since S10, and it
looks fine until you say it out loud: *this peat is coal because the swamp
lasted a long time.*

That is not what makes coal. Coalification is a burial process. Overburden
squeezes the water out and heat drives off the volatiles; peat becomes lignite,
lignite becomes bituminous. A ten-metre peat bed lying at the surface is peat. A
ten-centimetre bed under a hundred metres of section is coal.

The uncomfortable part is that the codebase already said so, in two places, and
neither was consulted. `CLASS_ORGANIC_COAL`'s contract in `dc-core` reads *"the
class's **depth axis is the rank axis** — a pack that wants lignite/bituminous/
anthracite members discriminates them on burial depth, which is the real
control."* And burial depth was not an unavailable quantity that a thickness test
was standing in for: it is `Σ` of the thicknesses of the units above, over a
vector we were already iterating. One pass, no allocation, no new state.

So this is ARCHITECTURE.md § *A summary is not an authority* in its purest form.
Not a cheap approximation of an expensive answer — a **cheap answer to a
different question**, sitting next to the real one, passing its tests.

The fix is six lines: walk the record top-down, accumulate overburden, promote
peat once its own overburden crosses a threshold. Burial `> 0` subsumes the old
"not the topmost unit" guard for free — the top unit has nothing above it.

### What moved, and why "moved" is the wrong word

I expected a redistribution: some columns gaining coal, some losing it. Measured
on the production Medium world (`examples/coal_charcoal_probe.rs`):

| | coal-bearing cells | coal units | total coal | thickest seam |
|---|---:|---:|---:|---:|
| **old** — thickness ≥ 0.4 m | 12 892 (4.52 %) | 19 008 | 22 459 m | 15.74 m |
| **new** — burial ≥ 8 m | 2 216 (0.78 %) | 3 888 | 3 773 m | 14.01 m |

Nothing gains. Coal collapses by a factor of ~5.8. And the reason is the
interesting part: **thickness and burial depth are close to anti-correlated in
this record.** A thick peat is a peat that sat at a quiet, slowly-aggrading
surface for a long time — which is precisely the setting that does *not* then
bury it under a hundred metres of section. The old rule was not a noisy version
of the right answer. It was selecting something near its complement.

That is worth remembering the next time a stand-in "seems roughly right". Being
on the wrong axis is not a bounded error.

### The number is still a stub, and here is exactly what it cannot say

There is **no geotherm in this project.** The only temperature anywhere in the
sim is surface air temperature (`climate::air_temp_c`, a latitude gradient minus
a lapse rate). So what landed is burial *depth*, not a pressure/temperature path,
and it therefore **cannot express coal rank** — lignite, sub-bituminous,
bituminous, anthracite are one `Coal` facies with one member. I am saying this
plainly because the `CLASS_ORGANIC_COAL` contract could easily be read as
satisfied now, and it is not; it is merely no longer contradicted.

Nor could the threshold be taken from Earth. Peat→lignite wants 10²–10³ m of
section. Our record is the deep sim's regolith plane `H`, and its burial
distribution looks like this:

```
burial depth of peat-derived units (35 382 total)
  [  0.0,   0.5) m   8 574  24.2 %
  [  0.5,   1.0) m   3 648  10.3 %
  [  1.0,   2.0) m   4 174  11.8 %
  [  2.0,   5.0) m  10 327  29.2 %
  [  5.0,  10.0) m   6 518  18.4 %
  [ 10.0,  20.0) m   1 991   5.6 %
  [ 20.0,  50.0) m     137   0.4 %
  [ 50.0, 100.0) m      12   0.0 %
  [100.0,   inf) m       1   0.0 %
```

Thirteen units in the whole world lie under 50 m. An Earth-calibrated threshold
would give a world with no coal in it — technically honest, entirely useless, and
not what the axis fix was for. `COAL_BURIAL_M = 8.0 m` is calibrated to *this*
record's own distribution, at roughly its 90th percentile, under a statement of
shape rather than of target: **coal is what happens to the peat that got buried
deepest.** It is `stubs.md` § 14 now, with a geotherm named as its heir.

(A pleasing accident, not a goal: 0.78 % of columns is the same order as the
0.55 % S10 originally reported for coal, back when the world was a different
shape for other reasons.)

### Coal is still diggable, and the margin is the honest thing to report

`the_measured_coal_seam_is_coal_a_player_can_dig` — the shipped test that walks
the world, finds its strongest coal seam, and digs it — **passes unchanged**.
But its census moved hard: record seams over 3 m went **7 647 → 88**, and the
strongest seam that surfaces as diggable coal went **19 → 16** collapse-voxels
against a floor of 15. One voxel of margin.

I left the floor at 15 rather than lowering it, and I want the reason on the
record: the claim that test defends is *a player can find and dig a coal seam*,
and that claim still holds. If a later slice makes coal rarer again, the honest
move is to print the census and re-baseline out loud — not to slide the number
down a voxel at a time until the test stops meaning anything.

## II. Charcoal's excuse expired eleven days before anyone noticed

`geology::deep_class` had a paragraph explaining why the recorder's `Charcoal`
facies deliberately did **not** get a content class:

> a fire bed is a thin event bed (measured mean ~0.035 m over 158 310 beds, and
> **none** of them survives the 0.9 m voxel quantization), so a charcoal band
> cannot exist in a voxel column and a charcoal member would be dead content.
> The honest representation is an inclusion (pore/debris partial) — filed, not
> built.

It is a good paragraph. It measures, it reasons, it names the honest alternative
and files it. It is also, as of journal/0055, **false in its load-bearing
clause** — and I did not trust that, so I measured it before building on it.

The bed thicknesses hold up. Re-measured on today's world: 102 113 charcoal beds
across 34 192 columns (12.0 % of readable), mean **0.0289 m**, max **0.0400 m**,
and **zero** of them reach even one eighth of a voxel (0.1125 m) on their own.
The max is not a coincidence — fire residue is `soil × FIRE_CHAR_FRAC`, the soil
pool is capped at `SOIL_MAX = 2.0`, so `0.04 m` is a hard structural ceiling. A
charcoal bed can never be more than **0.356 of an eighth**. Ever.

What expired is *"cannot exist"*. That was true of a generator that asked
`round(tᵢ / 0.9)` **per recorded unit** and dropped everything under half a
voxel. Since journal/0055 the generator asks the quantization question **once per
voxel span**, and answers it by unbiased **addressed stochastic rounding**: a
material's fractional share of a span wins a whole eighth with probability equal
to that share. A 2.9 cm bed claims `8 × 0.0289 / 0.9 ≈ 0.257` of an eighth and
therefore takes a real one about a quarter of the times it is asked.

The inclusion that comment called the honest representation and filed as unbuilt
**had been built**, by a slice aimed at sediment mass conservation, and nothing
went back to re-read the filings.

### What charcoal actually measures out to

Routed to `CLASS_ORGANIC_CHARCOAL` with a `dc:geo/charcoal` member and
`MaterialId::CHARCOAL`, then measured through the generator's own `ColumnFill`
and addressed draw over 819 sampled recorded chunk columns / 79 370 voxel spans:

- **313 spans (0.394 %) carry at least one charcoal eighth**;
- **314 of 634 960 allocated eighths (0.0495 %) are charcoal**.

The shipped falsifier (`charcoal_reaches_the_voxel_as_an_inclusion_never_as_a_
stratum`) re-measures this over its own sample and lands on **265 of 68 977
spans, 0.384 %** — and asserts the *other* direction too: **the most eighths any
one voxel gave to charcoal is 2**, out of eight. That second assertion is the one
that keeps the fix honest. A fire bed is capped at 0.04 m by construction, so a
charcoal-dominant voxel would mean the expression path invented mass, and
"charcoal now exists" would be a bug wearing a feature's clothes.

Small. Real. Not zero — and the brief explicitly authorized me to report "still
effectively zero" and stop, so it is worth being precise about why 0.39 % is a
result rather than a rounding error. Charcoal is now *findable*: roughly one
voxel in 254 of recorded ground has burned wood in it, concentrated in the 12 %
of columns whose history includes fire. A cut face through a fire-bearing
floodplain has black specks in it, in the right places, at the right rate. That
is what a 3 cm lamina is *supposed* to look like in a 90 cm voxel. Expressing it
as a band would have been the lie.

### Three decisions inside that, each of which could have gone the other way

**1. Charcoal is `loose`, not structural.** `fill::is_loose` now includes it, so
a charcoal eighth rides in the debris multiset the way a placer gold grain rides
in gravel. If it were structural, `classify`'s structure-first rule would let a
single eighth claim the voxel's block identity — one speck of charcoal turning a
mudstone into something else. Inclusion is not a description here, it is the
implementation.

**2. It gets a block twin (`Block::Coal`) even though it never dominates a
voxel.** This looks like dead content and nearly is. It exists for one case: the
top-of-column **partial** fill, where the surface voxel may hold only one or two
eighths in the first place, and a charcoal eighth can genuinely be the dominant
material. Without a twin, `block_twin` falls through to `Block::Stone`, and a
burned surface horizon would occasionally read as grey rock. Sharing coal's black
carbon band is the honest summary in today's block vocabulary; growing that
vocabulary is a separate content decision.

**3. The `deep_class` ↔ `litho_of_tag` mirror gets its first deliberate
exception.** The two routings are asserted equal over the whole tag space,
because if erosion thinks a bed is sandstone while the collapse tier builds it
out of mudstone, the world's *shape* stops explaining the world's *rock*.
Charcoal now breaks that equality on purpose: the collapse tier expresses the
carbon, and deep time keeps reading the clastic host.

I went back and forth on this and the argument that settled it is a question of
what each tier is asking. `deep_class` asks *what fills these metres*. `litho_of_tag`
asks *what rock resists this erosion agent across a 460 m cell*. A 3 cm lamina
has an answer to the first question and no answer at all to the second — the
enclosing mud has that one. Making charcoal a `Litho` would hand an entire
erosion cell the strength of its thinnest lamina whenever a fire happened to be
the last recorded event, which is not a fidelity gain; it is a category error
with a world-reshaping blast radius. The mirror's *purpose* survives intact,
because the rock is still the host. The exception is asserted by name in
`litho_routing_matches_the_collapse_tier`, not skipped — it is as pinned as the
rule it breaks.

## What this cost, in goldens

Both fixes are deliberate behaviour changes, so fingerprints moved and are
re-baselined with authorization notes in place:

- `contents_contract.rs` — both Medium worlds moved on all three hashes (blocks,
  materials, mixture table). The **Small** control did not move, for the third
  slice running: no deep-time record, so no peat to promote and no fire bed to
  express.
- `providers.rs::GOLDEN_RECORD` moved; **`GOLDEN_SURFACE` did not.** That pair is
  the informative one. Promotion runs at finalize, after every erosion epoch, so
  it can change which units are *labelled* coal and cannot move a metre of
  ground. The provider slice's byte-identity proof is intact underneath one
  authorized behaviour change.
- `dc-core` member count 13 → 14; the fullbright palette / WGSL array size
  25 → 26 materials (the one dc-client edit this slice needed — that guard exists
  precisely so adding a material cannot silently desync the shader).

Two corrections filed: `corrections.md` § 32 (the expired charcoal excuse) and
§ 33 (the coal axis, *including* a wrong prediction in the brief that set this
work — it said coal would "move", and coal did not move, it collapsed).

## The thread through both

The coal rule and the charcoal comment fail in mirror-image ways. The coal rule
was **code that answered a different question than the one it was named for**,
next to an authority that could have answered the real one. The charcoal comment
was **prose that answered the right question correctly, and then went stale**
because the world changed underneath it.

The first has a doctrine and an inventory. The second has nothing — and it is
arguably the more dangerous, because a well-reasoned "we deliberately did not
build this, here is why" reads as evidence of care, and it keeps reading that way
long after the why has stopped being true. The only defence I know of is the one
the brief used on me: **verify the reasoning before building on it, even when —
especially when — it is your own.**
