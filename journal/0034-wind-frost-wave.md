# 0034 — Wind, frost, and wave: the roster stops being one agent

For all its ambition the deep-time erosion engine had, until now, exactly **one**
agent. 0029 built the shape that admits many — resistance is a property of a
*(rock, agent)* pair, `Agent` is an exhaustively-matched enum, and a rock carries
a separate resistance axis for abrasion, dissolution, frost/ice, and wave attack.
But only the mechanical (abrasion) agent was ever wired to live erosion. The
other three axes were *populated and dormant*: computed, tested, and consulted by
nobody. Deserts had an arid *tag* but no arid *landform* — no dune fields, no
loess sheets, no deflation. Cold high ground weathered at exactly the same rate as
a warm lowland. Coastlines did nothing at all.

This entry turns the roster on. Wind becomes the **fifth** agent; frost and wave —
built in 0029 and left waiting — go live. All three ride behind one new flag,
`full_agents`, off by default and provably byte-identical when off, exactly the
way the S10 biotic layer and the 0029 erodibility coupling ship. The user flips
production themselves.

## The empty-roster problem, and why the enum made it cheap

The nice thing about 0029's structural choice is what it did to *this* milestone.
`Agent` is not `#[non_exhaustive]`, and every consumer matches it exhaustively.
Adding `Agent::Eolian` was therefore not a design decision I could forget to
propagate — it was a *compile error* at `LithoResistance::to`, at
`resistance_of_material`, at `Agent::ALL`, at every sweep. The compiler walked me
to each site that had to answer "what does wind read?" and refused to build until
I had. That is the guarantee 0029 paid for: a new agent cannot silently inherit
the mechanical answer. It cost about ten one-line edits, each forced and each
obvious.

Wind's axis is **cohesion, and only cohesion**. This is the same discipline 0029
learned the hard way (and recorded): reach for the property that governs *this*
agent, not a plausible-sounding blend. Wind cannot grind competent bedrock — it
lifts loose grain. So grain hardness (`smash`) is deliberately *absent* from the
eolian axis: a hard-but-loose sand deflates freely into a dune field, while a
sticky clay or a cemented crust resists however soft its grains. Keying on
cohesion makes sandstone-derived sand (cohesion 0.85) the dune source and
mudstone-derived clay (0.95) the resistant playa crust — which is the right way
round.

## Reusing the climate's wind, not inventing a second one

The temptation with an eolian agent is to invent a wind field. There already *is*
one: `climate::march` blows a moisture front along a prevailing zonal direction
(`wind_dx` — trade easterlies, mid-latitude westerlies, polar easterlies) and
rains it out over the orography. Inventing a second, differently-directed wind for
sediment would be a subtle lie — the dust would drift one way and the rain
another, and a lee desert would sit downwind of nothing.

So the eolian phase is a **1D march along the exact same `wind_dx`**. Dry,
unvegetated, subaerial cells hand loose cover to an airborne load; vegetated or
humid downwind cells trap it. In the hyper-arid source core the trapped sand
records as a **dune field**; on the semi-arid downwind margin the fine silt
records as a **loess** sheet. It is the moisture march's structure — per-row,
carry-a-scalar, settle-where-the-energy-drops — turned from rain onto sediment.
The vegetation gate reads the S10 biotic root-cohesion plane (`bio_resist`) when
biology is on, and treats bare ground as maximally deflatable when it is off.

The one thing the march *must* guarantee is that it never leaks mass: wind only
**redistributes** loose `H`, it does not create or destroy it. Each row conserves
its own airborne load, and whatever is still aloft at the downwind land edge
settles there. Measured over a seeded Small world: the mass ledger residual for a
wind-only run is `-0.000` — the redistribution is exact, and the global invariant
`Δ(ΣR+ΣH) == uplift + biotic` is untouched because wind contributes zero to the
right-hand side.

## The frost band: freeze–thaw is loudest at zero, not at cold

The instinct for a cold-weathering term is "colder → more," a monotonic ramp. That
is wrong, and the reason is the mechanism. Frost shattering is *freeze–thaw*: water
admitted into pores and joints, frozen, expanded, thawed, again. A permanently
frozen summit barely weathers — nothing ever melts to refreeze. A warm lowland
never freezes at all. The damage is maximal in the **band around 0 °C** where the
phase boundary is crossed and re-crossed. So the frost multiplier peaks at 0 °C and
tapers to nothing at ±`frost_band_width_c`, weighted by the rock's frost/ice axis
(competence discounted by permeability — a porous sandstone shatters where a tight
granite endures).

The honest question was temperature data. Deep time carries **no pregen `temp_c`
plane** — the only climate field marched onto the deep grid is precipitation. But
latitude and the eroding surface *are* present every epoch, and the S10 biotic
layer already derived an air temperature from them (`30 − 0.55·|lat|` at sea
level, lapsed `6.5 °C/km` up the surface) to gate its species tolerances. Rather
than invent a second climate, I lifted that derivation into
`climate::air_temp_c` and pointed both the biotic layer and the frost agent at it,
so "where does it freeze" has exactly one answer. That refactor is byte-identical
(the biotic suite confirms it), and it means the periglacial band rides *up* the
mountains and *down* the latitudes precisely where biology already agrees it is
cold.

Frost folds into the existing `weather` phase as a third multiplier —
`weathering × (wmult × litho_sus × frost)`. This is the payoff of 0029's other
observation: in-place weathering is not one process, it is the *sum of every
agent's attack on rock that hasn't moved yet*. It had one term (mechanical); frost
is the second, temperature-gated; and the day dissolution lands it will be the
third, with a limestone weathering fast through the chemical term while resisting
the other two — no rewrite, just another factor. Measured: over 1442 cells inside
the periglacial band, frost stripped **+668.5 m** of extra bedrock into regolith;
a warm control band well outside 0 °C moved **−0.00 m**. The signal is
periglacially localized, not a blanket speed-up — which is the whole claim.

## Wave: the sea does the cutting, at whatever level it stands

Wave attack is a shoreline process, and the shoreline already *moves* — the
sea-level curve cycles (§ 6), so a coastal column should record raised and drowned
wave-cut features over successive stands. The wave phase leans entirely on that.
Each epoch, a cell is attacked if it stands in the freeboard band just above the
**current** sea stand *and* touches open water; waves cut it down toward sea level
(clamped so it never drops below the stand — a platform forms *at* sea level, it
does not dig a hole), scaled by the rock's wave axis and a taper strongest right
at the waterline. The quarried volume goes offshore into the deepest adjacent
subsea cell as marine sediment, so the term is mass-neutral: loose cover entrained
first, then bedrock, the sum deposited into the sink. Because the phase writes a
*neighbour's* cell it stays scalar (like the transport flux chain), which keeps it
deterministic and byte-identical scalar↔parallel.

Measured, the wave signature is real but modest at this resolution: 327 coastal
cells lowered a total of 1.3 m over 40 epochs (~0.0001 m/epoch/cell). Which is the
honest place to say what 460 m (here 1 km, for a fast test) **cannot** express.

## What 460 m can and cannot say

The ratified deliverable is explicit: at deep-tier resolution the readable unit is
the *region*. This milestone produces dune-**field** and loess-**sheet** regions
in the record and surface, periglacial **bands**, and wave-cut **coasts** — not
individual dunes, scree cones, or sea stacks. A dune field is a cell (or a run of
cells) the record tags `Eolian::Dune`; the actual bedforms inside it are
collapse-tier detail for the sub-460 m band. The wave term cuts a coast *cell*
toward sea level; the notch-and-visor geometry of a real sea cliff is likewise
below the grid. This is not a shortcoming to apologize for — it is the "coarsen the
cause, refine on approach" architecture doing its job. The agents write the
regional cause honestly (and mass-conservingly); the appearance detail is dressed
on approach.

The wave magnitude in particular is deliberately left where it fell. Whether a
coast should retreat a metre or fifty per world is an appearance-class number, and
by standing doctrine those are the user's to set — the mechanism rides as built.

## How it stays off

`full_agents` defaults false, and `production_config` inherits it (it changes no
line there). With it off, the wind and wave phases are skipped entirely and the
frost plane is left empty and read as the identity `1.0`, so every world made
today is bit-for-bit what it was — the existing world-fingerprint and integration
suites pass unchanged. The off-path proof is the stronger 0029 kind: a run with the
flag **on** but all three rate knobs at zero executes every new phase and still
lands on the flag-off result to the bit (bedrock, alluvium, and strata planes all
`assert_eq!`), with biology and erodibility also on. The added terms are genuinely
identity/zero-preserving, not merely gated.

> blogworthy: the periglacial band — why the intuitive "colder rock weathers
> faster" is exactly backwards, and how a weathering rate that peaks at 0 °C falls
> out of the freeze–thaw *mechanism* rather than being imposed on it.

## Knobs

Per the standing "where there's a cell range, there's a knob" doctrine: the master
`full_agents` flag, plus `eolian_deflation` / `eolian_arid_precip` /
`eolian_deposit_frac` for wind, `frost_weathering_gain` / `frost_band_width_c` for
frost, and `wave_erosion` / `wave_band_m` for wave. The three agents reuse the
erodibility `contrast`/`max` knobs for their susceptibility tables — the same
"turn compressed property-sheet numbers into landform contrast" lever, and the
same stability clamp bounds all of them. The clamp is what keeps frost's additive
enhancement bounded (`≤ 1 + gain·cap`); wind and wave are further bounded by the
loose cover actually available and by the freeboard to sea level, so no term can
run away.
