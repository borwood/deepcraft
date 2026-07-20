# 0029 — Limestone makes cliffs *and* caves

The user, reading walk cross-sections, called our mountains dismal: no cliffs,
no benches, no caprock, nothing standing out from anything else. The diagnosis
had four causes, and the user ordered the first one taken: *erosion is
lithology-blind.* `erosion.rs` incised every cell with one global `k_bedrock`,
so granite and mudstone eroded at exactly the same rate. There was no
differential erosion anywhere in the world — and differential erosion is where
nearly all landform drama comes from. A hard bed makes a cliff by outlasting the
soft bed beside it. If every bed erodes identically, the world is a smooth ramp
no matter what it's made of.

The recorder already knew which unit outcropped at each deep cell's surface,
every epoch. Erosion simply never asked.

So the milestone read simple: ask.

## The trap that is the whole point

The obvious model is one number per rock — call it *erodibility* — high for
granite, low for mudstone, multiply the incision rate by it. Two hours of work.
It is also a design mistake expensive enough that the user flagged it in the
brief before I could make it.

**Limestone is mechanically competent and chemically soluble at the same time.**
It stands in vertical cliffs *because* it is strong, and it hosts the world's
cave systems *because* it dissolves. Those two facts are simultaneously true, and
they pull a single erodibility number in opposite directions:

- make limestone erodibility *low* (hard) and you get the cliff, but karst is
  now impossible — nothing dissolves a rock the model calls resistant;
- make it *high* (soft) and you get the cave, but limestone can never hold a
  cliff again.

There is no value of one number that gives both. A one-number model doesn't
approximate karst badly — it *forecloses* it. And the day someone writes the
dissolution agent, the coupling has to be torn out and rebuilt, because the
scalar it was built on cannot express what that agent needs to read. We have no
carbonate in the roster yet, so nothing dissolves today. That is exactly why now
is the moment to get the shape right: the cost of the wrong shape is invisible
until the rewrite.

The resolution is that **resistance is not a property of a rock. It is a property
of a *(rock, agent)* pair.** A material resists *abrasion* by one amount,
*dissolution* by another, *frost* by another, *wave attack* by another. So:

- The property sheet grew a `solubility` axis alongside its existing mechanical
  `extraction_resistance`. Every rock in the roster is a silicate, an organic
  rock, or a loose clastic, so every `solubility` is honestly `0.0` — a test
  (`nothing_in_the_current_roster_dissolves`) asserts it, and names the karst
  milestone as the commit that is *supposed* to make it fail.
- A new module, `deeptime::lithology`, defines an `Agent` enum (Abrasion,
  Dissolution, FrostIce, Wave) and a `LithoResistance` struct with **one field
  per agent**, each derived from the property-sheet field that governs *that*
  agent: abrasion from smash resistance, dissolution from the inverse of
  solubility, frost/ice from competence discounted by permeability (frost needs
  pore water to freeze), wave from competence keyed hard on cohesion (sea cliffs
  fail along joints, not by grain hardness).
- `Agent` is *not* `#[non_exhaustive]`, and every consumer matches it
  exhaustively. Adding a fifth agent is a compile error at every site that has
  to answer for it. That is the structural guarantee that a new agent cannot
  silently inherit the mechanical answer — the thing a scalar model does by
  default.

Only the mechanical agent is wired to live erosion. The other three axes are
populated and dormant. When the dissolution agent lands it consults
`LithoResistance::dissolution` and gets a *different answer* than the mechanical
agent gets from the same rock — proven today, on a carbonate-shaped test sheet
the roster doesn't yet contain
(`a_limestone_can_be_cliff_forming_and_cave_forming_at_once`). That test is
unsatisfiable under a single scalar. It passing is the non-preclusion
requirement, cashed.

> blogworthy: the limestone problem — why "how erodible is this rock?" is the
> wrong question, and a one-number answer quietly deletes every cave in the
> world you haven't built yet.

## Which rock, and the shield for free

At the deep-time tier there is no material yet — the recorder tags units by
measured *environment*, and members resolve at collapse time. So the chain is
`DepTag → Litho → reference MaterialId → property sheet → LithoResistance`, and
`litho_of_tag` mirrors `geology::deep_class` **exactly** (a test sweeps all 72
tags and asserts the two agree). The rock that resisted erosion is the rock a
player will dig; if the two routings ever drift, you get a resistant ridge built
out of mudstone, and the world's shape stops explaining the world's rock.

The reference member per class is *fixed*, not sampled from the live registry.
That is deliberate: it means adding an organism or material pack can never move
terrain (the ROADMAP's long-standing "pack-addition blast radius" worry). Packs
diversify what fills a class; they don't renegotiate how fast the class erodes.

The exposed lithology is the top of the record — and *an empty record means
basement.* That one fallback is where the falsifiable prediction lives: strip a
column past its whole sedimentary history and the next thing erosion meets is
igneous basement, the hardest thing in the world. Nobody writes a shield rule;
resistant cratons fall out of the fallback. Measured, it holds: basement
outcrops stand ~5,750 m proud of covered terrain, and turning coupling on *widens
the exposed basement area* (729 → 758 cells) as soft cover is stripped faster
around the hard cores.

## The wrong plane, and where the pace actually lives

My first wiring coupled bedrock incision and cover entrainment — the two
"obvious" fluvial terms the brief named. I ran the probe expecting cliffs.

The world was statistically **unchanged.** Relief +0.0%. Slope sd +0.0%. The
control's residual drift was exactly zero (byte-identical OFF↔OFF, as it must
be), so the null wasn't noise — the coupling genuinely did almost nothing.

Two things were wrong, and finding them is the real content of this entry.

**First, I had tempered the abrasion resistance by cohesion**, reasoning that a
poorly-cemented rock sheds clasts. But mudstone is *more* cohesive than sandstone
(0.95 vs 0.85) — it's sticky, not strong — so the tempering pulled the two rocks
from a healthy contrast toward parity, actively cancelling the differentiation
the milestone exists to create. Cohesion is the right modifier for the *wave*
agent and the wrong one here. Abrasion now reads the smash resistance straight.
(This is the same lesson as corrections #6, in miniature: I reached for a
plausible modifier without measuring which direction it moved the contrast.)

**Second, and the mechanism worth remembering:** on a hillslope the rate-limiting
step is *not* incision or entrainment. Hillslope diffusion is flux-limited by the
regolith actually available — on any real slope it exports everything there is —
so the landscape's lowering rate collapses to the rate at which bedrock is
*converted into regolith*, which is the **weathering** phase. I had coupled the
two terms that don't set the pace on most of the land, and left the one that does
at its uniform global rate. Coupling weathering is what made the world respond.

That also turns out to be the *correct* long-run home for the coupling, not a
patch. In-place weathering is not one process; it's the sum of every agent's
attack on rock that hasn't moved yet. Today that sum has one term (mechanical),
so the weathering multiplier is the abrasion susceptibility. When dissolution
lands, the same line becomes a sum over agents, and a limestone will weather
*fast* through the chemical term while resisting the mechanical one. The karst
story arrives by adding a term, not by rewriting the phase.

With weathering coupled, the shipped calibration gives clean differential
erosion: along one 250-change transect, mudstone cells sit +3 to +12 m above
their carbonaceous-mudstone neighbours, cell by cell, alternating exactly with
the outcrop pattern. The sharpest single contact flips a contour: a mudstone cell
that sat 14.6 m *below* its soft neighbour with coupling off stands 1.1 m *above*
it with coupling on — a bench inverted into existence by nothing but the rock.

## The amplitude question is the user's, and I can prove it's separate

The shipped calibration's contrast is real but modest in aggregate: relief
+2 m, steep-cell fraction +0.5 pp, slope_sd +1.1%. The reason is not the
resistance model — it's that at the shipped erosion rates, the whole landscape
only removes a few metres against hundreds of metres of uplift. There isn't much
erosion for *any* resistance model to differentiate.

To prove the model isn't the bottleneck, the probe runs a **headroom
experiment**: the same coupling with every erosion rate scaled 10× together
(relative rates unchanged). The contrast scales right up with it — relief +20 m,
steep +0.8 pp, a hard bed standing **44.7 m** proud of its neighbour where it
stood 39.5 m *below* with coupling off. A correct model waiting on an amplitude
decision, not a broken one. How hard the world should erode is cause 3 of the
dismal-mountains diagnosis ("conservative amplitude") and the user's call to
make; this milestone closes cause 1 and hands cause 3 a working lever.

Every differencing number above is reported next to its control's residual drift
(zero, by byte-identity) and against a **uniformly-weakened** control — coupling
off, but with the global rates scaled by the ON run's abundance-weighted mean
multiplier. That control isolates *differential* erosion from "coupling just
erodes a bit less overall": ON still beats uniform on slope_sd and steep
fraction, so the signal is the differentiation, not the mean. (Corrections #15's
corollary, honoured — a differencing measurement means nothing without the drift
of its control beside it.)

## Composition order, stated

Two lagged biotic modifiers already touched erosion (S10): `resist` (root
cohesion, damps diffusion) and `wmult` (biotic weathering acceleration). The
lithic factor composes with them **deliberately**, and the order is load-bearing
because f64 multiplication is not associative — written the wrong way round,
turning the new coupling off would not reproduce the S10 result bit for bit.

- **Weathering:** `weathering × (wmult × litho_sus) × taper`. The two modifier
  layers combine with each other first — biology on the left, lithology on the
  right — then scale the base rate. With lithology off, `wmult × 1.0 == wmult`
  exactly.
- **Diffusion:** `diffusion × (1 − resist) × litho_sus`. Biology first
  (root cohesion is a property of the living slope), lithology multiplied onto
  the right.
- **Fluvial (entrainment + incision):** scaled by `litho_sus` directly; no
  biotic term here.

Unlike the S10 modifiers, which are read *one epoch late* to break the
biology↔erosion feedback cycle, lithology is computed **fresh at the top of each
epoch** (a new `expose` phase) — the record it reads is only rewritten at the end
of the step, so there is no cycle to break.

## The feedback: bounded on purpose

Differential erosion is self-reinforcing: erode soft rock, expose hard rock, slow
down. That is what carves benches — and it is also what could park a cell in a
runaway or a permanent stall. The susceptibility is clamped to `[1/max, max]`
(`erodibility_max`, default 5×), and a stress test at 4× contrast over 80
iterations confirms the guard: nothing goes non-finite, late-run activity stays
within an order of magnitude of early (no runaway), and the world hasn't frozen
(cells still moving). The clamp is the stability mechanism, not a tuning fudge.

## Cost, and off-by-default

On the shipped parallel path (production config, 297,025 cells, biology on) the
deep-time sim goes **13.89 s → 14.39 s** — a **+0.5 s** marginal cost for the new
`expose` phase and the per-cell multiplies. The test suite grew ~503 s → ~541 s,
almost all of it the new erodibility suite's own runs (13 tests, ~10 s of sim
plus the neutral-identity double-runs); no `--ignored` gating.

It ships **off by default** and provably byte-identical when off — the same class
of flip as the S10 biotic layer. `production_config` inherits `erodibility:
false`, so every world created today is bit-for-bit what it was. Turning it on
changes `DeepField`, and therefore terrain shape, for every world made
afterward. The off-path proof is stronger than comparing off against itself: the
suite runs the *coupled* code with contrast zero, so every new multiplication
executes and must still land on the uncoupled result to the bit
(`a_neutral_coupling_is_byte_identical_to_no_coupling`, and again with biology
on). Determinism holds all the way through: double-run byte-identical, and
scalar↔parallel byte-identical with the coupling on, with and without biology.

## Knobs

Per the standing "where there's a cell range, there's a knob" doctrine, every
tunable is a `DeepConfig` field, not a baked constant: `erodibility` (the flag),
`erodibility_contrast` (fluvial/weathering exponent, default 2.5),
`erodibility_diffusion_contrast` (creep, default 1.0 — softer, because creep is
governed by regolith mobility more than parent-rock competence), and
`erodibility_max` (the stability clamp, default 5×).
