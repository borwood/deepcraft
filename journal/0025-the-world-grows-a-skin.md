# 0025 — The world grows a skin

*S10, the biotic-layer spike. 2026-07-20, background agent, worktree branch.*

> blogworthy: **a simulation that had to be told what "wet" means.** Four
> separate times, this spike failed because a number that was obviously
> reasonable in my head was nowhere near the number the world actually
> contained. The fix each time was not cleverness — it was going and *measuring
> the distribution the model lives in* before writing the thresholds that judge
> it. The punchline is a coal seam 24 metres thick under a marine band, which no
> one designed.

## What we were asked

ecology.md § 6 set the gate plainly: put the **community vector** and **the six
processes** on the deep-time A tier, and measure two things — what it costs
against the ~14 s world-creation ritual, and whether the record comes out
*readable*. Four target signals, named in advance so we could not move the
goalposts afterwards: **coal seams**, **paleosols**, **charcoal bands**,
**retrogressive surfaces**. Evolution explicitly excluded; S10 exists to prove
the substrate evolution will later ride.

The shape of the answer was never in doubt architecturally. ecology.md had
already done the hard design thinking: species as the primitive, biome as a
diagnosis, `min` over tolerances because Liebig's law is a minimum and not a
mean, and — the load-bearing one — the **lagged coupling rule**. Biology changes
erosion, erosion changes terrain, terrain changes climate and soil, soil changes
biology. The pass graph would refuse that cycle, correctly. So biology reads
*this* epoch's terrain and writes modifiers the *next* epoch's erosion consumes.
That is not a hack; it is how the physics works at these timescales. Writing it
was ten lines in the run loop and it never gave a moment's trouble.

Everything that *did* give trouble came from the same root, and it took four
distinct failures before I recognised it.

## Wrong turn 1: a world where nothing could grow

First full run. Paleosols appeared. Coal did not. Charcoal did not.
Retrogression did not. The community diagnostic said: lichen 10 %, fire-grass
3 %, everything else zero. Available nitrogen: `0.000`.

I had built the nutrient cycle with uptake, litter, fixation, leaching, and
weathering release — and no **return**. Plants drew nitrogen out of the soil
pool and it never came back. Decomposition existed in the model, but I had only
used it to decide how much litter got *buried*; I never fed the decomposed
fraction back into the nutrient pools. Every cell slowly starved.

The fix is one line, and it is the line that makes the whole thing honest:

```rust
cell.p_avail += p_taken * decomp_frac;
cell.n       += n_taken * decomp_frac;
```

The decomposed fraction of this epoch's uptake comes straight back; only the
fraction that *escapes* decomposition is lost, because it has been buried in the
record. Which means the leak in the nutrient cycle **is** the strata record.
That is why a peat bog is nutrient-poor: burial is the drain. I did not design
that correspondence, I just stopped breaking it, and it fell out.

## Wrong turn 2: species that could never exist anywhere

Nitrogen recovered. Forest, grass, n-fixer, peat-sedge, sclerophyll: still
exactly zero, everywhere, forever.

Dispersal was the culprit, and the bug is a nice one. Propagule pressure was
`max(own prior cover, half the best neighbour's, pioneer seed rain)`, with the
seed rain given only to the two designated pioneers. So a non-pioneer species
could only arrive somewhere if it was *already* somewhere adjacent — and since
it started nowhere, it could reach nowhere. A perfectly self-consistent
bootstrap deadlock. The five non-pioneer species were not rare; they were
**impossible**.

Worth noticing what this looked like from outside: the run *worked*. It produced
paleosols and even a few coal seams. The coal was being made by lichen litter
sitting in a wet spot. If I had only checked "do the signals appear," I would
have shipped a world with five species in the roster and two species in
existence.

The fix is a weak background propagule rain — "these species exist in the
region" — an order of magnitude below the neighbour term, so spread stays
neighbour-driven and still makes fronts. Flagged in the results doc, because
ecology.md is silent on world-genesis colonization and this is my choice, not
the design's.

## Wrong turn 3: pioneers that won forever

Now every species *could* exist. Forest still didn't. Lichen held the entire
map.

This one is a genuine ecological modelling error rather than a coding slip. I
had written competitiveness as `suitability × propagule_pressure × incumbency`.
Pioneers have high propagule pressure by definition. So the pioneers were not
just winning the race to arrive — they were winning the *equilibrium*, forever,
which is precisely what succession is not. Connell & Slatyer's whole point is
that pioneers win colonization and later-successional species win the contest
once everybody is present.

So dispersal stopped being a competitive magnitude and became a **saturating
gate** — has it arrived in force, yes or no — and competition got an intrinsic
rank term (shade/stature). Pioneers now colonise fast and then get shaded out,
which is the entire mechanism succession is made of.

## Wrong turn 4: the model did not know what its own world looked like

Forest *still* sat at zero. At this point I stopped guessing and printed the
percentiles of the fields the niches are judged against. The answer was
embarrassing and immediate:

```
land precip percentiles: p05 0.046  p25 0.056  p50 0.059  p75 0.069  p95 0.151  max 0.808
```

Median land precipitation was **0.059**. I had written forest's moisture
tolerance as `moist_lo = 0.40`, grass at 0.22, peat-sedge at 0.66 — numbers that
are perfectly sensible if precipitation is a uniform 0..1 field, and completely
disconnected from the field the orographic march actually produces, which piles
land up in a narrow band around 0.06 with a thin wet tail to 0.81. Every
later-successional species was being asked to clear a bar three to ten times
higher than the world's maximum median. They were not outcompeted. They were
**never eligible**.

The same audit exposed available phosphorus as strongly bimodal (p05 0.0006,
p50 0.137, p95 0.504) — which turned out to be a *feature*, since that spread is
exactly the chronosequence, but I had set the P demands against an imagined
spread too.

Recalibrating the roster to the measured distributions took one pass and the
world came alive: all seven species present, mean cover ~0.40, and every one of
the four signals appearing.

This is corrections #6's lesson wearing different clothes — "the cell equals the
chunk" was true of the *wrong* cell — and corrections #10's — read the
instrument in the right units before diagnosing. Here it was: **do not write a
threshold against a field you have not measured.** I have left the measured
percentiles in a comment above the roster so the next person to touch those
numbers knows what world they are calibrating against.

## The mechanism worth keeping: soil is an overprint, not a layer

One structural discovery deserves its own section, because it is the difference
between a record that is legible and one that is landfill.

My first instinct was that soil formation *deposits* organic material — a layer,
on top, each epoch. It is the obvious reading of "litter/peat → depositable
organic material." It is also wrong, and the record told me so immediately: unit
counts went from 72 k to **665 k**, and a single column read like this, two
hundred times over:

```
0.135 m  [Sa/A/L   ]  subaerial
0.005 m  [Sa/A/L·So]  organic soil horizon   <-- PALEOSOL
0.126 m  [Sa/A/L   ]  subaerial
0.002 m  [Sa/A/L·So]  organic soil horizon   <-- PALEOSOL
```

Erosion's recorder was depositing a mineral sliver every epoch; biology was
depositing an organic sliver every epoch; their tags differed, so nothing ever
merged. A 314-unit column recording, in effect, one fact.

Real pedogenesis does not stack a layer. It **alters the material already at the
surface** — that is what a soil profile *is*. So the recorder got
`overprint_top`: add the organic mass to the topmost unit, retag it, and merge
it down into an identically-tagged predecessor. A surface that stays stable for
a hundred epochs now records **one thick horizon** instead of a hundred
laminae. Unit count fell from 665 k straight back to 71 k — within 0.2 % of the
biology-free run.

Two refusals keep it honest: a charcoal bed is never retagged (a fire bed is a
preserved event, not a substrate to be overwritten), and a unit flagged as an
unconformity never merges downward (that would erase a time gap). And because
the overprint only ever adds thickness once and moves thickness between units,
`sum(units) == H` survives exactly — there is a test.

The companion insight arrived with it: a soil horizon requires a **depositional
hiatus**. Where mineral sediment is arriving faster than a threshold, litter is
diluted into the mineral unit and no profile develops. That is textbook pedology
— soils form when deposition pauses — and it is *also* the thing that keeps
floodplain columns sane. When the physically correct mechanism and the
cost-control mechanism turn out to be the same mechanism, that is usually a sign
the model is pointing the right way.

## What came out

All four signals, at the A tier, on seed `0x0D5E_ED57_2026`:

- **Coal** — 1 133 columns, thickest seam **24.03 m**, over a marine section and
  under fire-bearing alluvium.
- **Paleosols** — 22.9 % of columns; the best of them is a genuine **cyclothem**,
  soils and peats alternating with marine bands, each soil carrying the climate
  measured when it formed (`Sa/H` humid vs `Sa/A` arid).
- **Charcoal** — 20.2 % of columns; one upland column carries **19 fire beds**
  down its length, all tagged arid, because that is where things burn.
- **Retrogression** — 23.5 %; the example is an ancient 1148 m surface with soil
  at its 2 m cap, available P at 0.0097, and its rock-phosphorus pool drawn down
  to 0.44 — a Walker & Syers chronosequence that nobody scripted. Erosion
  rejuvenates slopes and floodplains by exposing and delivering fresh mineral P;
  only the surfaces erosion never touches age into starvation. The *geography*
  of retrogression is right, and that is the part I would not have known how to
  fake.

Cost: the ritual goes **15.17 s → 25.19 s** (1.66×) and the world *keeps* an
extra **6 MiB** — everything else is a transient spike that is dropped when the
run ends. The ON/OFF ratio actually *falls* as grids grow (1.80 → 1.66), because
the biotic step is a flat per-cell pass with no heap and no global dependency
chain while erosion carries the priority flood's `n log n`. And since
`DEEP_MAX_WIDTH` already caps the deep grid at any extent, the +10 s is the same
at Large as at Medium.

There is a falsifier for the thing that actually justifies the cost:
`biology_changes_the_landscape_it_grows_on`. Root cohesion resists hillslope
creep and root acids accelerate weathering, so a biotic run's bedrock surface
measurably differs from an abiotic one. Biology here is an earth-process term,
not an annotation. That test is the one I would most want to survive future
refactors.

## The honest gap

The record contains coal. **The world does not.** Collapse maps deep units to
material classes through `deep_class`, which reads environment and energy only;
an organic unit collapses today as ordinary clastic, and there is no coal
material in the roster. A player cannot yet dig into that 24-metre seam.

That is worth stating plainly because it is exactly the kind of thing a spike
can paper over. S10 proved the substrate — the community vector is affordable,
the six processes run deterministically, the record is legible, and biology
moves the landscape. It did not deliver the payoff, and the payoff is one
material-tier slice away.

The other thing S10 changed is that a question in ecology.md § 5 stopped being
hypothetical. Adding an organism pack now changes **terrain**, because biology
is an erosion term. Materials packs never did that. The doc filed it as "needs
deciding before biology couples to erosion" — and biology now couples to
erosion, so it needs deciding.
