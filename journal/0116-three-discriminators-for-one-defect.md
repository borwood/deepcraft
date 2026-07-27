# 0116 — Three discriminators, and the one that was never a discriminator

*2026-07-26 · running the tests journal/0115 wrote into the blocker instead of building on
the hypothesis it left there*

> blogworthy: **lens 1 (AI-native development)** and **lens 3 (reflexions in a deepsim
> codebase)**. A brief arrived carrying a mechanism. A mid-flight amendment arrived carrying
> a better one. Both were wrong, and they were wrong in *opposite directions* — one said the
> defect was a time-step artefact, the other said a particular pass was driving it, and the
> measurements say the time step barely matters and that pass is the only thing holding the
> world together. What survived was the *third* thing, which nobody proposed: the shape of
> the answer was already sitting in a counter journal/0114 had printed and not connected.

## The situation as handed over

journal/0115 did something unusual and correct: it re-scoped the top blocker on live-walk
evidence, and then **refused to promote its own explanation**. The stub entry says so in as
many words — *"Hypothesis, explicitly not measured: an explicit scheme past its stability
limit"* — and names the two experiments that would settle it. That is the entire reason this
entry exists rather than a fix slice built on sand.

The hypothesis had a real argument behind it. Creep's flux limiter binds on 89–96 % of cells
(stubs #27), so the hillslope diffusion has degenerated into *"move everything one cell
downslope this epoch"*, and a saturated explicit operator overshoots. Overshoot is what
oscillation looks like. It fits.

Mid-flight the coordinator sent a second mechanism — **isostasy** — with a much sharper
argument. `erosion.rs::isostasy` computes its Airy target from a **flexurally smoothed**
crust and sediment load and then differences it against the cell's own **unsmoothed**
surface:

```rust
let e_eq = isostasy::equilibrium(t_bar[i], h_bar[i], rho_c);  // smoothed inputs
let surf = grid.r[i] + grid.h[i];                             // LOCAL, unsmoothed
grid.r[i] += cfg.iso_rate * (e_eq - surf);
```

A cell whose regolith stands far above its neighbourhood mean would be driven down in
proportion to that excess, and being driven down would collect more creep from its
neighbours. A positive feedback whose gain is set by local regolith excess — and one that
could only bite once the calibration takes mean regolith from 4.6 m to 43.9 m, which is
exactly the regime where the defect appears. It fits *better*.

Three arms, then: D1 (is it an oscillation at all), D2 (does it respond like a stability
limit), D3 (is isostasy driving it). All measured on production-Medium, seed 1337, with the
shipped world carried alongside as the control every time.

## D1 — it is a checkerboard, and the control is beautifully boring

The instrument is the spatial autocorrelation and sign-alternation of the concavity field.
The thing that makes it work is that **all three reference values are derivable in closed
form**, so the numbers mean something before you look at them:

Concavity is `mean(8 neighbours) − self`. Over a **white-noise** surface that operator has
variance `1.125 σ²` and lag-1 covariance `−0.1875 σ²`, so its ACF(1) is `−1/6 ≈ −0.167`.
Over a **perfect checkerboard**, the four diagonal neighbours carry the cell's own sign and
the four cardinal ones the opposite, so `mean(8) = 0`, the concavity field *is* the
checkerboard negated, and ACF(1) = −1 with ACF(2) = +1. Over a **smooth** surface it tends
to +1. Sign-alternation follows: `P(opposite sign) = arccos(ρ)/π`, so 0.55 at white noise,
1.00 at a checkerboard, 0 at smooth.

That last step matters more than it looks. **The discriminator is not "is the
autocorrelation negative"** — an ordinary noisy heightfield is already at −0.167. It is *how
far past −1/6 it has gone.*

| | conc rms | ACF(1) x / y | ACF(2) x / y | sign-flip x / y | d(surf) ACF(1) x / y |
|---|---|---|---|---|---|
| **shipped** | 0.22 m | **+0.377 / +0.267** | +0.446 / +0.500 | 0.225 / 0.249 | **+0.999 / +1.000** |
| **calibrated** | 40.46 m | **−0.867 / −0.912** | +0.556 / +0.706 | 0.678 / 0.732 | **−0.863 / −0.860** |

The shipped world is *positively* autocorrelated at the grid scale — smoother than white
noise, with a first-difference autocorrelation of +0.999, which is what a landscape looks
like. The calibrated world sits at −0.87 and −0.91, alternates sign at lag 2, and its
first-difference autocorrelation is −0.86 against a white-noise reference of −0.50 and a
checkerboard reference of −1.00.

**It is a checkerboard, in both axes, and it is close to a pure one.** journal/0115 inferred
this from symmetric ±50 m tails with an unmoved median. The inference was right.

## D2 — and here the brief's own hypothesis dies

D2 refines the time step at **fixed total simulated time**: `k×` the epochs, `1/k×`
everything the config states per epoch, `k×` everything it states in epochs. The epoch-length
knob the brief named (`myr_per_epoch`) **does not exist** — the register is `iterations`
against per-epoch rates — so the operator had to be assembled by hand, and the honest way to
present it is to say which terms it touches and which it does not. It scales `iterations`,
the four erosion rates (through the same `scale_erosion_rates` the calibration uses, so their
ratios never move), `thickening_scale`, `eolian_deflation`, `wave_erosion`,
`sea_level_period`, `remarch_interval`, and — as exact exponential refinements
`1 − (1−f)^(1/k)` — `iso_rate` and `eolian_deposit_frac`. It deliberately does not touch
`uplift_scale` (inert: `apply_uplift` runs only when `tectonic_history` is off), the chapter
schedule (already normalised by `iterations`), or the dimensionless knobs. One term is left
un-neutralised and named rather than assumed away: `HEAD_PERIOD` and `GEOTHERM_PERIOD` are
constants, not config, so those two coarse-rate field passes fire `k×` more often in physical
time — both plant read-quality fields that no in-epoch erosion pass reads.

The operator carries its own falsifier: applied to the **shipped** arm it must leave the
world alone.

| arm | epochs | conc rms | p10 / p90 / p99 | closed hollows | relief | mean surf | ACF(1)x | gen |
|---|---|---|---|---|---|---|---|---|
| shipped k=1 | 200 | 0.22 | −0.3 / +0.1 / +0.3 | 0 | 5521.9 | 517.9 | +0.377 | 39.8 s |
| shipped k=2 *(control)* | 400 | **0.23** | −0.4 / +0.1 / +0.4 | **0** | **5520.7** | **517.7** | +0.297 | 103.7 s |
| calibrated k=1 | 200 | 40.46 | −50.8 / +52.8 / +106.3 | 1377 (3.1 %) | 5581.0 | 520.4 | −0.867 | 83.9 s |
| calibrated k=2 | 400 | **45.29** | −56.6 / +61.2 / +118.1 | 955 (2.2 %) | 5647.3 | 520.2 | −0.909 | 94.5 s |
| calibrated k=4 | 800 | **38.76** | −42.3 / +49.1 / +111.8 | 280 (0.6 %) | 5796.7 | 518.7 | −0.947 | 175.8 s |

The control holds to three digits, so the operator is sound. And then:

> **Refine the time step four-fold and the roughness does not collapse. It moves by 4 %.**

40.46 → 45.29 → 38.76. Not monotone, not a convergence, not even a trend — the amplitude at
a quarter of the step is 96 % of the amplitude at full step. A scheme running past a CFL
limit does not behave like this; it collapses, and it collapses roughly with the step.

Worse for the hypothesis: **the checkerboard gets *purer* as the step shrinks.** ACF(1) goes
−0.867 → −0.909 → −0.947, and by k=4 the whole lag sequence is the textbook alternation of a
Nyquist mode: `−0.947, +0.827, −0.645, +0.516` in x and `−0.967, +0.903, −0.798, +0.721` in
y. Refining time *cleans up* the oscillation. The landscape meanwhile holds — relief +3.9 %
across a 4× refinement, mean surface −0.3 %.

One real thing does respond: **closed hollows fall 1377 → 955 → 280**. So the *pits* are
partly a time-step artefact even though the *checkerboard* is not, which is a sharper version
of journal/0115's "the pits are the tail". The tail converges. The defect does not.

**The brief's hypothesis is falsified.** This is not an explicit scheme past its stability
limit.

## D3 — isostasy is not the driver; it is the only thing keeping the grid together

The ablation, on the calibrated arm, with the shipped arm carried along:

| | conc rms | closed hollows | deepest | relief | mean surf | corr(conc, h−h̄) |
|---|---|---|---|---|---|---|
| calibrated, `iso_rate` 0.50 *(production)* | **40.46** | 1,377 (3.1 %) | 112.8 m | 5581.0 | 520.4 | −0.831 |
| calibrated, `iso_rate` 0.25 | **62.03** | 2,150 (4.9 %) | 154.5 m | 5636.7 | 522.1 | −0.885 |
| calibrated, `iso_rate` 0.00 | **90.34** | 13,012 (9.8 %) | 190.4 m | 3393.0 | 477.0 | −0.885 |
| shipped, `iso_rate` 0.00 | **19.71** | 6,215 (4.6 %) | 91.0 m | 3520.9 | 484.1 | +0.354 |

Monotone, and in the **opposite direction to the hypothesis**. Turning isostasy *down*
makes the checkerboard *worse*: half the relaxation rate, 53 % more roughness, 56 % more
closed hollows. Turn it off entirely and the roughness is 2.2× production's — and, decisively,
**the same thing happens on the shipped world**, which has no defect at all: `iso_rate = 0`
takes the shipped concavity rms from 0.22 m to 19.71 m and its closed hollows from **0 to
6,215**.

The `iso_rate = 0` rows are not a clean ablation and should not be read as one: with no
isostatic relaxation, crustal thickening never becomes elevation, so relief drops from
5,582 m to 3,393 m and it is a different world. **The clean point is 0.25** — a 1 % change in
relief that already makes the roughness 53 % worse. That single row is enough.

Isostasy in this solve is a **relaxation of the surface toward a flexurally smoothed target
at 0.5 per epoch**. That is a strong grid-scale low-pass filter, and it is doing exactly what
a low-pass filter does. The amendment's reading of the code is correct on every line; what it
missed is the sign of the loop closure. The target is smooth and the cell is pulled *toward*
it.

The correlation test agrees, and its sign is the tell. `corr(conc, h − h̄) = −0.831` on the
calibrated arm — **negative**. Concavity is positive when a cell sits *below* its
surroundings, so a strong negative correlation says cells with excess regolith sit *above*
their neighbours. Not driven down. Held up. Which is the trivial reading of `surf = r + h`,
and that is the point:

> **At |r| = 0.83, the concavity field essentially *is* the local-regolith-excess field.**
> `rms(h − h̄)` is 70.1 m on the calibrated world against 4.0 m on the shipped one.

The crustal floor check (D3c) is a flat null worth recording so nobody re-runs it: **0.01 %
of cells sit on `apply_thickening`'s 1000 m floor, identically on every arm and at every
`iso_rate`.** It is not engaged and it is not a nonlinearity here.

## The follow-up, because a null is only worth something if you can say why the knob was wrong

D2 said the step is not the register. That is a real result and it is also an unsatisfying
one, because it leaves the *reason* unstated — and an unstated reason is how the last two
mechanisms got written into the corpus. So two more measurements, one solve each: split
`surf = r + h` and run the same Laplacian over each summand, and read the **creep flux
limiter's binding fraction** off `TransportLedger` (which needs only `denudation_ledger`
armed). The concavity columns reproduce the sweep's to every printed digit, which
incidentally confirms that ledger really is read-only rather than merely documented as such.

| | limiter bound | conc(**r**) rms · ACF(1) x/y | conc(**h**) rms · ACF(1) x/y | surf conc rms |
|---|---|---|---|---|
| shipped k=1 | 88.7 % | 3.42 m · −0.101 / −0.173 | 3.43 m · −0.102 / −0.183 | **0.22 m** |
| calibrated k=1 | **96.0 %** | 23.77 m · −0.546 / −0.508 | 61.95 m · −0.819 / −0.851 | 40.46 m |
| calibrated k=2 | **94.9 %** | 12.59 m · −0.344 / −0.219 | 55.21 m · −0.878 / −0.914 | 45.29 m |
| calibrated k=4 | **94.7 %** | **5.88 m · −0.108 / +0.069** | **42.43 m · −0.929 / −0.945** | 38.76 m |

Three things fall out, and the third is the answer.

**The shipped row is the one to read first, again.** Bedrock and regolith each carry ~3.4 m of
grid-scale concavity — and their sum carries **0.22 m**. They are almost exactly
anti-correlated: *the regolith blanket fills the bedrock's grid-scale hollows.* The mantle
compensates the solve's own noise, which is both physically right and the reason nobody has
ever seen this. On the calibrated world that compensation has broken down: 23.77 and 61.95
leave a coherent 40.46 m residual with a *sharper* checkerboard than either summand.

**The bedrock solve converges. It was never the problem.** Under 4× refinement `conc(r)` goes
**23.77 → 12.59 → 5.88 m** — very nearly `1/k` — and its autocorrelation walks back from
−0.55 to −0.11 / +0.07, i.e. to no oscillation at all. That is exactly what a well-posed
explicit scheme does when you refine its step. **There *is* a genuine time-step artefact in
this world, it is in the bedrock, and D2 converged it away.**

**The regolith does the opposite.** `conc(h)` falls only 61.95 → 42.43 m, and its
autocorrelation goes **−0.819 → −0.878 → −0.929** — *toward* a pure checkerboard. And even the
32 % amplitude drop is not convergence: the refinement does not hold the cover fixed (mean
regolith falls 41.41 → 36.59 → 24.45 m, because refining changes the splitting error against
the nonlinear `exp(−H/h*)` weathering taper). Normalise by the cover the operator is actually
moving and the roughness **grows monotonically**: `conc(h)/h̄` = 1.50 → 1.51 → 1.74, and
`rms(h − h̄)/h̄` = 1.69 → 1.74 → 2.06.

> The surface total looked flat under refinement (40.5 → 45.3 → 38.8) because it is the sum of
> a converging bedrock artefact and a sharpening regolith checkerboard. **Two defects were
> hiding inside one statistic, and one of them was cancelling the other's convergence.**

**And the limiter is measured, not inferred, to be deaf to the step: 96.0 % → 94.9 % →
94.7 %.** A 4× refinement of the time step moves the binding fraction by 1.3 points. That is
the mechanism the null was asking for, stated as a measurement:

> **A flux limiter that caps export at *the cover the cell has* makes the transfer a function
> of inventory, not of `rate × dt`.** Refining `dt` cannot reduce a transfer that `dt` does
> not set. D2 turned the one knob the defect is structurally deaf to — and the same
> measurement shows the knob working perfectly on the part of the solve that *is* a function
> of `dt`.

## What actually survives

**The defect is structural, it is in the hillslope-transport pass, and the register is the
flux limiter — not the clamp, not the clock, and not isostasy.**

One qualification the data insists on, because it is the difference between a mechanism and a
slogan: **saturation alone is not sufficient.** The limiter already binds on **88.7 %** of the
*shipped* world's cells, and the shipped world's `conc(h)` autocorrelation is −0.10 — no
checkerboard at all. What the calibration adds is not saturation, it is **cover**: mean
regolith 4.58 → 41.41 m. A saturated conveyor handing over 4.6 m of blanket produces a ±3.4 m
wobble the surface absorbs; the same conveyor handing over 41 m produces a ±62 m one it does
not. **The oscillation amplitude scales with the inventory the limiter surrenders**, which is
precisely what "inventory, not `rate × dt`" predicts, and it is why this could only ever have
appeared once the world was made to erode.

*The period-2 reading is the strongest interpretation of the data, and it is still an
interpretation:* a donor-cell scheme that moves *everything* downslope must flip-flop — A
gives all its cover to B, B is now higher and gives it back — which is a checkerboard in `h`,
independent of `dt`, damped only by whatever low-pass sits downstream, which on this world is
isostasy. Every number above is consistent with it and none of them isolates it. It is written
here in the same shape journal/0115 wrote *its* hypothesis, for the fix slice to test first —
because the entire value of this entry came from the previous one refusing to promote its own
guess.

## What went into the gate, and what deliberately did not

The structural half of the pair now has a guard. `the_shipped_solve_has_no_grid_scale_
oscillation` asserts the shipped concavity's lag-1 autocorrelation stays above −0.5 and its
sign-alternation below 0.85, on both axes — **bounds taken from the closed-form references,
not from yesterday's run**: −0.5 sits 3× past white noise's −1/6 and 2× short of a
checkerboard's −1; 0.85 sits between white noise's 0.55 and a checkerboard's 1.00. That is
corrections #61's *"pair every aggregate criterion with a neighbour-relative one"* stopping
being advice in a doc.

The both-arms test earns it: at `Extent::Small` the calibrated arm reports ACF(1)
**−0.847 / −0.797** and sign-flip **0.619 / 0.626**, so the new guard is not vacuous at gate
size. Total for the example's four tests: **10.7 s**, of which the new one is one extra Small
census, ≈2.7 s.

The **sweeps are not gated** and should not be — they are 12 and 5 minutes of production-Medium
solves, and their output is a report to be read, not an invariant to be held. They live behind
`--sweep` and `--fields` so the default report stays what it always was.

One thing is worth flagging rather than quietly fixing: `walk_tour_0115.rs` is now **1,199
lines**, well past the provisional 700-line threshold, and the file-size hook said so on every
edit. The discriminators *are* a separable concern — but a Rust example is its own crate root,
so a sibling example cannot call this one's `census()`. Splitting would mean either copying the
census (anti-shape A-1, and the census is the exact thing three instruments already got wrong
by copying in journal/0115) or moving it into `src/`, which this brief put out of scope.
**Recorded as an open extraction, not resolved.**

## The thing this entry is really about

Two mechanisms arrived from two different places, hours apart, both argued from the code,
both plausible, both specific. **They were opposites**: one said the fix was to shrink the
step, the other said the fix was to stop a pass from running. Acting on either would have
made the world worse — `iso_rate` in particular is the only reason the *shipped* world has
zero closed hollows, and a slice that "fixed" it would have put 6,215 holes in a world that
currently has none.

Neither was caught by argument. Both were caught by the same cheap thing: **running the
experiment on the arm you expect to be boring.** The shipped control is what proved the
refinement operator sound; what proved isostasy a damper rather than a driver (0 → 6,215
hollows on a world with no defect); what gave D1 its calibrated reference; and — the one
nobody would have predicted — what revealed that bedrock and regolith roughness normally
*cancel*, which is the fact that makes the calibrated arm legible at all. Four of the five
load-bearing facts here come from the row that was expected to be a formality.

> *A discriminator that only runs on the broken arm can tell you the defect is there. It
> takes the boring arm to tell you what it is.*

And a smaller one, from D2, which is the part I would put in a brief template:

> *A null from a well-built experiment is not "no information". It is a constraint on where
> the mechanism can live — but only if you go on to ask why the knob you turned was the
> wrong knob.* Stopping at "refining the step did nothing" would have left this entry with
> one dead hypothesis and no successor. Two more solves, on a counter that had been printing
> the answer since journal/0114, turned it into a named register.
