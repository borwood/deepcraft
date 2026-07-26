# 0114 — The world that buries itself

*2026-07-26 · the joint supply + transport calibration — and the ceiling underneath it*

> blogworthy: **lens 4 (respect for earth processes)** and **lens 2 (procgen against
> the backdrop of priors)**. journal/0111 held this world up against the literature and
> found its erosional clock stopped. This is the entry that tries to start it — and
> discovers that **the rates were never the binding constraint**. You can multiply every
> erosion constant in the engine by a thousand and the landscape still will not reach the
> published craton band; it will only bury itself under eight hundred metres of its own
> weathering products. The interesting part is that the *measurement designed to accept a
> calibration* is what refused it, and named the real cap instead.

## The brief, and the number it was aimed at

journal/0111 measured catchment-averaged denudation on the shipped world at
**0.0110 m/Myr** against the ratified 500 Myr Phanerozoic register: 9× slower than the
slowest landscape ever measured on Earth, 493× below the global ¹⁰Be outcrop median,
stripping **5.48 m** where a real craton strips 5–10 km. It also found the shape of the
repair — supply and transport are coupled through the cover taper `exp(−H/H*)`, neither
lever pays alone, and together they paid 132× where their separate gains multiplied give
2.4×.

So the brief was a **calibration, not an architecture**: move the rates until the measured
quantity lands in the published **1–10 m/Myr** stable-craton band, with the balance ratio
`D1/D3` near 1, and ship it behind a flag.

That is not what happened, and the reason is the entry.

## Three things had to be built before a number could be chosen

### The knob could not do the job (stubs #24)

`DeepOverrides::erosion_budget` was documented as *"**the** TERRAIN (erosion) amplitude"*
and scaled `weathering`, `k_transport` and `k_bedrock` — **not** `diffusion`, the process
carrying 96 % of this world's denudation. At 100× it moved the total by 1.4×.

The fix is not a fifth multiplicand in a list. It is the observation that those four
constants are **one clock**, and that they belong behind **one function**:
`field.rs::scale_erosion_rates`. The shipped calibration and the dev knob are now two
consumers of that single function, so their scopes cannot drift apart again — which is a
stronger statement than "someone remembered to add `diffusion`", and it is asserted by a
test that would have failed on every commit before this one.

### Scaling `k_transport` would have destroyed the facies gradient (corrections #59)

This one was nearly missed, and it is the sharpest lesson in the slice.

`COMPETENCE_SCALE = 420` is the one calibration constant Movement 2b added, and its doc
comment defended it at length as *"not a tuning knob"*:

> *"It is fixed by an anchor that already ships: `energy_band` calls a capacity of `0.002`
> the Low/Medium boundary … so `0.002` is already the world's stated 'energy at which sand
> stops moving'. `settle_energy` puts the coarse-clastic reference sheet at ≈0.84, and
> 0.84 / 0.002 = 420."*

The ratio is right. The **form** is wrong. Transport capacity is `cap = k_transport ·
A^m · S^n`, so a bare capacity is not a geomorphic quantity at all — it is a rate constant
multiplied by **a position in the drainage network**. `0.002` is not "the energy at which
sand stops moving"; it is `1.25 × k_transport`, and it read as a physical statement only
because `k_transport` had never moved.

Multiply `k_transport` by 45 and leave those boundaries absolute, and **every depositional
site on the world classifies as High energy**. `litho_of_tag` records coarse clastic
everywhere; the competence ceiling rises far enough to carry basement to the sea. The
facies gradient the Movement 2b probes exist to measure would have been **erased by the
same commit that was supposed to make the world erode** — and the erasure would have
looked like a triumph: *"sand moves now."*

So the thresholds are stated relative to a named `REFERENCE_KT`, and `energy_band` /
`competence_ceiling` take the world's own coefficient. Every ratio is formed as
`k / REFERENCE_KT`, because `x / x` is exactly `1.0` in IEEE-754 — so at the historical
value every threshold is bit-identical to the constant it replaced.

The family this belongs to is worth naming, because it is journal/0111's error one level
down. That entry's lesson was *a closed system cannot detect its own scale error*. This is
**a constant defended by deriving it from another constant in the same system**. A
derivation is only an anchor if the thing it is derived from cannot move — and
`k_transport` was, at that moment, already named on the ROADMAP as due for recalibration.

### The instrument had to be able to say no

`denudation_probe` gained the derivation itself: the shipped world, the **uncalibrated**
world re-measured under today's solve (so the before/after is one variable and not a quote
carried across three merges), and a ladder of uniform multipliers over the raw constants —
each a full 200-epoch world.

It also gained the column that ended up mattering most: **mean regolith thickness**, and
the fraction of cell-epochs in which hillslope creep's flux limiter binds.

## The ladder

Seed 1337, `Extent::Medium`, 200 epochs, ~44 k land cells. Every row is a full world.

| uniform × | D1 | D3 | D1/D3 | D1/D4 | mean surf | relief | **mean H** | land | creep-lim |
|---|---|---|---|---|---|---|---|---|---|
| 1 (raw) | 0.0110 | 0.0107 | 1.03 | 0.03 | 517.4 | 1286.7 | **4.62** | 44267 | 88.7 % |
| 10 | 0.0765 | 0.0449 | 1.70 | 0.18 | 517.4 | 1288.5 | **8.81** | 44263 | 98.5 % |
| **45 — SHIPPED** | **0.4142** | **0.1271** | 3.26 | 0.91 | 521.8 | 1346.4 | **43.87** | 44077 | 96.0 % |
| 100 | 1.3377 | 0.2767 | 4.83 | 2.82 | 532.2 | 1522.0 | **118.79** | 43646 | 94.5 % |
| 300 | 10.5825 | 0.7108 | 14.89 | 20.04 | 589.2 | 1961.5 | **359.47** | 40898 | 92.5 % |
| 1000 | 57.1331 | 1.3857 | 41.23 | 92.58 | 712.5 | 2800.4 | **782.08** | 36511 | 89.4 % |

The row at 100× is *in the published craton band*. It is also carrying **119 metres of
mean regolith** and has grown its relief by 18 %.

## What the `mean H` column says, and why it is the finding

The calibration's whole hypothesis was that a **uniform** scaling would hold cover
thickness fixed. Production and removal of regolith both scale, so the steady-state `H`
should sit still, the taper — the only term in the system carrying an absolute length —
should never engage, and export should follow the multiplier.

**It does not.** Cover thickens at every rung, monotonically, faster than linearly. The
two levers are not symmetric, and the asymmetry has a name:

> **Transport has a ceiling that supply does not.**

`diffuse_scale_cell` clamps a cell's outflow to the regolith it actually has. The
`creep-lim` column counts how often that clamp bound — **over cells that had regolith to
move**, because the predicate `potential outflow > available cover` is trivially true at
`H = 0` and a denominator of every cell would have scored the bare ocean floor as
transport-limited and reported a saturation that was really an absence. *(The first draft
of that counter did exactly that; it is the kind of thing this repo catches, and it caught
itself.)* On the cells that matter it binds nearly always, **before any calibration at
all**: the hillslope pass had already stopped being a diffusion and become *"move
everything one cell downslope this epoch"*.

Raising `diffusion` cannot make a conveyor go faster, which is exactly what the
single-lever contrast measures: **100× on transport alone buys 1.6×**.

And if material moves one cell per epoch, then only the **shoreline ring** exports, and
what it exports is whatever cover it is holding. So:

> **Catchment export is proportional to mean regolith thickness.**

Check it against the table. `D1 / mean H` reads 0.0024, 0.0087, 0.0094, 0.0113, 0.0294,
0.0730 down the ladder. Across the four rungs that bracket the shipped value — 10× to
100× — it is **0.0087 to 0.0113, flat within 30 %**, while `D1` itself moves **17×** over
the same span. It drifts upward past 300× for a reason that is not a counterexample but
another symptom: by then the land area has shrunk 8 % and relief has grown 52 %, so the
shoreline ring is a larger share of a smaller, steeper continent.

The price is legible straight off the table without extrapolating: **the first row inside
the craton band is 100×, and it carries 119 m of mean cover.** The middle of the band
costs several hundred.

**That is not a landscape. It is a world burying itself in its own weathering products
because it has no way to ship them out.**

The irony is precise: journal/0110's null was *"rivers do nothing here"*, and journal/0111
corrected it to *"rivers do nothing partly because nothing does anything"*. This entry
corrects it again. Rivers doing nothing **is** the cause after all — not of the slow rate,
but of the ceiling on it. On Earth the long-distance sediment router is the channel
network. Here it moves **0.02 %** of the yield, and the hillslope conveyor that moves the
other 99.87 % cannot drain a continental interior.

## The number that was not chosen

`EROSION_CALIBRATION = 45`, and it is picked by four criteria — three of them published
bands — that all land together. None of them is an appearance.

1. **The shape is preserved.** journal/0111's conclusion was *"the shape is right and the
   clock is wrong"*, so a multiplier that moves the shape has stopped being a calibration
   and become a redesign. Relief within 5 %: **+4.6 %** at 45×, +6.3 % at 50×. **This is
   the binding criterion**, and 45 is the largest measured row that clears it.
2. **Mean regolith lands in the published deeply-weathered-shield range, 30–60 m**
   (Yilgarn, Guiana and Brazilian saprolite profiles): **43.9 m**. The pre-calibration
   4.6 m was below even the *typical* shield range; above 45× it leaves the range upward.
3. **Denudation enters a published terrestrial band.** D1 = **0.414 m/Myr**, inside the
   **0.1–1 floor band** (McMurdo Dry Valleys, hyperarid Atacama). journal/0111's headline
   was that this world sat below *every* published band. It no longer does.
4. **The landscape approaches topographic steady state.** `D1/D4` — export over rock
   uplift — is **0.91**, against **0.027**. journal/0111's sharpest sentence was that
   erosion removed 2.7 % of what uplift added and therefore *had no authority over the
   topography of this world*. It now has essentially all of it.

The world's own Airy ceiling corroborates that the band was the right target even though
it is out of reach. Compensation returns `(ρ_m − ρ_c)/ρ_m = 15.2 %` of each eroded metre
as a surface drop and rebounds the rest, so a landscape in topographic steady state
denudes at `U / 0.152 ≈ 6.6 U`; with the measured `U ≈ 0.41 m/Myr` that is **2.70 m/Myr** —
inside the craton band, computed from two densities and a measured uplift that nobody
chose for this purpose. **The rates can be raised to meet it. The router cannot carry it.**

## The instrument that stopped agreeing with itself

journal/0111 reported `D1` as the headline on a specific licence: two instruments that
share no arithmetic — a boundary-flux ledger and a per-cell rock-removal plane — agreed to
2.4 %. **That licence does not survive the calibration.** At 45× they differ by 226 %, and
the arithmetic does not close: the run removes 63.6 m of bedrock per land cell and stores
39.2 m more regolith, while `D1` claims 207.1 m of export.

The cause is in the ledger, not the world: `creep_to_sea_m` is a **gross** land→sea edge
flux, and the sea stand cycles ±35 m four times over the run, so cover ferried across the
shoreline, stranded by the next regression and ferried across again is counted each time.
At 1× the fluxes were too small for it to show. At 45× they are not.

So **D3 is the sound instrument after calibration** and D1 is an upper bound, and the
probe now says so — derived from the measured gap, not asserted beside it. On D3 the move
is 0.0107 → **0.1271 m/Myr**, a factor of **11.9**.

That correction is left standing rather than fixed here: netting the shoreline flux is a
change to the ledger's definition, and doing it inside a calibration would have meant
moving the instrument and the thing it measures in the same commit.

## Outputs checked, never targeted

**Does sand move?** It did not before, anywhere on the world (corrections #55). Now the
largest competence ceiling reaches **3.24** against sand's 0.840 threshold, and **0.095 %
of land cells** clear it. That is a non-null, and it is a small one; the median cell is
still 8× below the threshold. It also did not come from the competence constant, which is
invariant under the calibration by construction — it came from the landscape, whose
steeper trunks concentrate more `A^m·S^n`. *`facies_probe`'s caption said "no cell on this
world can carry sand" as flat prose; it is now derived, because it stopped being true.*

**Does a facies gradient appear?** **No. Still null**, and honestly so: fluvial transport
is **0.021 %** of sediment routing (0.109 % before — it went *down*, because creep grew
too), and the energy-band fining ratio is unmoved at 75.0. The gradient does not express,
for exactly the reason journal/0110 gave, and the calibration did not change it. Nothing
was tuned to make it appear.

**stubs #26, the hybrid-`p` `χ` thresholds.** Still standing, and its own note asked
whether this calibration would let them be derived. It does not: `χ = A · S²` is built from
drainage area in **cells** and a dimensionless slope, and neither carries `k_transport`, so
the calibration does not give `χ` the physical scale that entry hoped for. The debt is
recorded, unchanged.

## The thing it broke, and why it is in this entry rather than a footnote

A calibration that only made numbers better would be suspicious. This one has a cost, it
is the largest single cost, and it is not in the shipped world's appearance — it is in
another arc's **falsifier**.

`tests/geotherm.rs` carries journal/0093's central physical claim: *coal relocates onto
warm crust*. It is measured on a deliberately non-production "warm reference" world,
because the shipped world grows no coal at all (corrections #51). Both worlds, same
fixture, same session:

```text
UNCALIBRATED  coal 42.4 vs peat 31.0 C/km   (ratio 1.37)   4,858 coal / 36,969 peat
CALIBRATED    coal 34.7 vs peat 34.2 C/km   (ratio 1.02)   1,742 coal /  7,978 peat
```

**The mechanism is confounded, not broken, and the confound is exactly this slice's
doing.** Burial temperature is `surface_T + gradient × depth`: gradient and depth are two
multiplicative routes to the same onset. Before the calibration, overburden had almost no
range — p50 2.2 m, p90 10.6 m, max 94 m — so essentially all the variance in `T` came
from the gradient, and the pooled comparison was **implicitly depth-controlled without
anyone choosing that**. Multiply deposition by 45 and overburden reaches 451 m; units
become 4.3× fewer and correspondingly thicker; a unit on cool crust now reaches the onset
by being buried deeply. The two populations stop being comparable and their raw means
become a Simpson's-paradox trap.

Stratified by burial depth, the relocation is still there — and where it fails is
informative:

| overburden band | coal | peat | |
|---|---|---|---|
| 0–0 m | 26.6 | 33.4 | inverted |
| 0–0 m | 26.1 | 29.8 | inverted |
| 0–1 m | 31.7 | 33.1 | inverted |
| 1–2 m | 36.4 | 33.8 | holds |
| 2–3 m | 37.0 | 36.0 | holds |
| 3–4 m | 36.6 | 35.3 | holds |
| 4–7 m | 41.3 | 37.2 | holds |
| 7–21 m | 41.8 | 35.7 | holds |
| 21–72 m | 35.8 | 33.2 | holds |
| 72–451 m | 31.6 | 35.4 | inverted |

Six for six through the middle. It inverts at both **degenerate** ends: at ~zero
overburden, where a thick unit's own half-thickness is supplying the "depth" and burial
is not really the variable; and in the deepest band, where 58 % of candidates cook
whatever the gradient is and the survivors are selected by surface temperature instead.

**What was done about it, and what deliberately was not.** The assertion was restated to
the form it always meant — *at comparable burial depth, does coal sit on warmer crust?* —
which is strictly stronger against a confound than the pooled means were. It pins the
sign and a majority of depth bands. **It is no longer sensitive to the magnitude, and
that is a real loss of guard strength.** Lowering the old 5 % margin until it passed
would have hidden the same loss behind a smaller number, and that is the move this repo
keeps catching, so the loss is written into the test's doc comment and into this entry
instead.

It is left **open for the user**, with three options and only one of them free: accept
that thick basins make burial the dominant control on coal rank — which is what Earth's
coal actually does; revisit the multiplier; or stop counting a unit's own half-thickness
as its burial depth, which is a defect in the *candidate model* that only became visible
once beds got thick. Nothing here touches the shipped world, whose production guard still
passes and which has no coal to relocate.

## What this entry is really about

The brief said *"this is a calibration, not an architecture"*, and it was right about the
defect and wrong about the fix — for a good reason, which is that the evidence available
when it was written pointed exactly there. journal/0111's sensitivity sweep showed two
levers that pay together, and the natural reading of two coupled levers is that you scale
both. That reading is what this slice tested, and it is the `mean H` column — a diagnostic
added *because* the hypothesis needed a falsifier, not because anyone expected it to fire —
that refused it.

The general shape, and it is the third instance this week: **the quantity that limits a
system is rarely the quantity someone is holding a knob for.** journal/0111 found a knob
that could not reach the eroding process; this entry finds that reaching it does not help,
because the process was already saturated. Both were invisible until something *outside*
the argument was measured — a published band, and then a cell-epoch counter.

And the shipped world is better. Denudation is 38× faster, bedrock erosion 12× faster,
erosion has gone from 2.7 % of uplift's authority over the topography to 91 %, and the
land no longer sits below every erosion rate ever measured on Earth. It is still four
times too slow, and now we know precisely what has to change for that to stop being true.

## What it cost, and the one thing it paid

**Gen time: nothing.** Both arms timed in the same process, same machine: **32.91 s
uncalibrated, 32.74 s calibrated** — a difference of −0.17 s, which is noise. The
calibration multiplies constants; it adds no work.

**Residency: it paid, and substantially.** Recorded units across the whole field went
**6,695,982 → 2,750,828, a 59 % drop** — at 16 B per `DepUnit`, about **107 MiB → 44 MiB**.
Thicker beds merge across epochs under the same tag and species, so the archive gets
*shorter* as the world gets busier. That is a pleasant surprise and worth stating as a
mechanism: **unit count is driven by how often the merge key changes, not by how much is
deposited.**

**And it has an appearance price on the same axis.** Fewer, thicker beds means fewer voxel
spans straddle a contact: `distribution_fill`'s sample went from over 100 mixed spans to
**52 of 4943 (1.1 %)**. Cut faces read blockier. That test's bound was a bedding-thickness
snapshot and failed here; it has been **re-derived** to the size of the defect it actually
names (independent rounding gives *exactly zero* mixed spans), rather than lowered until
green — but the magnitude is recorded, because a walk should look at it.

Every golden moved — the provider surface/record pair and all three contents-contract
worlds — and the pre-calibration world stays reachable and pinned by name
(`GOLDEN_SURFACE_UNCALIBRATED`, `tests/calibrated_rates.rs`, the `--uncalibrated` launch
flag). Five new gate tests plus one in the probe. **An appearance walk is owed**, and for
once the A/B is a single launch flag on the same seed.
