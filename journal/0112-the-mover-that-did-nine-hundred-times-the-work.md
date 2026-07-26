# 0112 — The mover that did nine hundred times the work

*2026-07-26 · Movement 2b, continuation (b) — material-aware hillslope creep*

> blogworthy: **lens 2 (procgen against the backdrop of priors)** and **lens 3
> (reflexions in a deepsim codebase)**. The angle is not "we added creep to the
> transport family". It is that the *previous* slice was briefed as the big
> appearance-changer, built exactly as ratified, and returned a null — and the
> instrument it was obliged to ship pointed at the actual answer, which was sitting
> in the same file the whole time doing nine hundred times the work with no name on
> any of it. A measured null rerouted an arc in one day.

## The number that promoted this slice

journal/0110 made the *fluvial* load material-aware. Every invariant it was
supposed to satisfy, it satisfied; the identity it carried reached **0.000006 % of
the archive**. The diagnosis (corrections #55) was not about sorting:

| where this world's sediment goes | metres, over the run |
|---|---|
| picked up by the flow (entrained + incised) | **659.5** |
| weathered to regolith **in place** — never enters a load | **256,886** |
| moved by **hillslope creep** | **605,117** |

**Creep does 918× what the rivers do, and it carried no identity at all.** The
world's dominant sediment router was anonymous. That is the whole reason this slice
exists, and it is a reason nobody could have stated before the fluvial pass existed
to be measured against — a scalar load has no ledger to itemise, and *"how much did
the rivers move compared to creep"* was not a question the engine could be asked.

## The physics, and the one thing not to improve

`material-behavior.md` § 13.2 has said since 2026-07-24 that transport is a
**family**: *"an agent moves material along a driving field"* — water, wind, ice,
**gravity** — sharing the same load machinery and differing only in their field and
their competence curve.

Gravity's competence curve is the interesting one, because **there isn't one.**

Creep is diffusive and gravity-driven. It does not sort by grain size the way a
falling competence ceiling does; a slope sheds what is lying on it, in the
proportions it is lying there. So material-aware creep means **identity travels and
nothing is sorted** — and the temptation to give creep a ceiling so it looks like
the river would have destroyed the only thing the slice buys:

> **colluvium = locally derived + poorly sorted; alluvium = far-travelled +
> sorted.**

That is a real facies distinction. A debris apron at the foot of a scarp is made of
the scarp, unsorted; the point bar fifty metres away is made of whatever the water
still had left after twenty cells of dropping the coarse fraction. Two rocks, one
mechanism, and the difference between them is a knob that is *absent* in one member.
Building creep as its own system would have had to invent that contrast. Building it
as a regime of the family got it for free.

## What was reused, and the one thing extracted

Almost all of it was already there:

- the **species multiset** and the seven-`Litho` resolution;
- **`outcrop_shares`** — the near-surface composition seam. It was serving the
  erodibility blend, then Movement 2b's entrainment; creep is the third consumer of
  the same window walk, so the composition a slope sheds can never disagree with the
  rate that slope erodes at;
- the **diffusion fluxes themselves**, unchanged: the same frozen surface, the same
  per-cell limiter, the same `eff_diff`. The species pass is a *third* gather over
  the same four edges.

One thing was extracted rather than reused, and it is worth the paragraph.
Movement 2b split bulk into named species in **two** places — entrainment and
incision — each open-coding journal/0109's residual rule inline. Creep is a third.
Three copies of *"the last non-zero share takes `total − Σ earlier`"* is how a
project ends up with four, and journal/0110 is on record that the **tempting**
generalisation of this rule leaks silently. So it became one function,
`split_by_shares`, byte-identically, and all three movers call it. The anti-leak
tests now sit on the one function instead of on whichever copy someone remembered.

## Mass, per species, at a junction with no downstream

journal/0110's per-species proof does not transfer as written, and noticing that was
the first real problem of the slice.

The fluvial proof is a **residue at a junction**: a cell holds `q_s` of a species,
hands shares to its receivers, and the shares must sum to `q_s` with no residue. It
works because the chain has a *direction* — there is a moment at which a cell is
finished and its load is spent.

A diffusion junction has no such moment. Every cell is simultaneously donor and
receiver on different edges of the same gather; there is no downstream order to walk
and no point at which a cell "hands over" anything. So the invariant had to be
restated on the **edge** rather than on the cell, and it rests on two facts:

1. **The edge flux is antisymmetric to the bit.** Cell *i* computes
   `eff_diff(i)·(sᵢ − sⱼ)·scale[i]`; cell *j*, gathering the same edge, computes
   `eff_diff(i)·−(sⱼ − sᵢ)·scale[i]`. IEEE-754 subtraction is exactly antisymmetric
   — `fl(a − b) = −fl(b − a)` for every finite pair — so those are the same number.
   This is the property the *scalar* diffusion already depended on for `ΣH`
   conservation; it just had never been written down.
2. **Both endpoints split that flux by the same donor composition, through the same
   function.** So what leaves *i* of a species is bit for bit what arrives at *j*.

Per-species conservation is therefore exact by construction rather than by
tolerance, and journal/0110's own-budget rule survives intact: the residual is taken
on the **species** axis of a single edge, so no species is ever rounded against a
denominator it shares with another.

It is asserted three ways, none of which is an argument:

- `a_composition_split_closes_to_the_bit` — `assert_eq!` on the sum, not an epsilon.
- `no_species_is_created_or_destroyed_by_creep` — two running maxima over every
  (cell, species, epoch) of a whole world: `Σ_species` of a cell's creep must equal
  the scalar metres the terrain moved there, and `Σ_cells` of a species must be
  zero.
- `mass_is_conserved_with_material_creep_on` — the whole-world ledger
  `Δ(ΣR + ΣH) == uplift + biotic`, which a share attributed to a cell but never
  taken from its donor would fail.

**The denominator was the bug.** The itemisation audit first divided its residual by
`Σ_species |net|` — the cell's *net* per species — and it failed at 10⁻⁶ on the
first run. The cause is not a leak: a cell that sheds as much as it gains has a net
near zero with real material moving through it, so a normal rounding error over a
near-zero denominator reads as a catastrophe. The right denominator is the **gross
traffic** — every edge flux, in or out. A conservation audit needs to be scaled by
what moved, not by what was left over.

## There is no configuration in which identity is inert

The slice's design story is *"the terrain moves by the scalar; the identity is an
attribution"*, and the natural test is to find a configuration where identity is on
and the terrain does not move. Three attempts failed, and the failures are the
finding.

1. **Erodibility off.** The world still moved. `full_agents` puts `outcrop_shares`
   in the frost multiplier, the eolian deflation susceptibility *and* the wave
   attack rate.
2. **`full_agents` off too.** It still moved — because the wind and wave agents run
   **after** the recorder, deliberately, so their own facies reach the record. They
   were reading *this epoch's* freshly written species back into a rate before the
   epoch was out.
3. And the last route cannot be switched off at all: the record's rock is what
   `outcrop_shares` publishes, that composition is what the fluvial load
   **entrains**, and the competence ceiling rains a species out by its settling
   velocity. **Changing what a hillslope is made of changes how much of it the river
   can hold.** Turning that off means turning off material-aware transport, which is
   the thing creep is gated on.

So the claim is pinned where it is exactly true — the first epoch, roster off, where
both arms enter with the same empty record and the only thing that can differ is
what `record` writes at the end. Everywhere else, identity is physics. Five
consumers read the archive's rock back into a rate, and I found them by being wrong
about it three times rather than by grepping, which is the honest way to report it.

## The number this slice is accepted on

NUMBERS_GO_HERE

## What a player sees

PLAYER_GOES_HERE

## What it cost

COST_GOES_HERE

## The thing this entry is really about

TAIL_GOES_HERE
