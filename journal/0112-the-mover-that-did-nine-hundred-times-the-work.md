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

`examples/colluvium_probe.rs` builds the production world (seed 1337,
`Extent::Medium`) twice — material-aware transport on in both arms, and **only** the
gravity member moving — so the delta is attributable to creep and not credited to the
family. The claim it answers is the one journal/0110 was accepted against and failed:
*what fraction of the recorded archive carries a material its own environment would
not have implied?*

| | anonymous creep (journal/0110's world) | creep carries identity |
|---|---|---|
| recorded mass whose material disagrees with its environment | **0.0259 m** of 440,578 m | **275,626.9 m** of 422,703 m |
| as a fraction of the archive | **0.000006 %** | **65.206 %** |

**A factor of eleven million.** Two thirds of the archive now says what actually
arrived instead of what the neighbourhood implied. The fluvial slice's mechanism was
correct all along; it was attached to the wrong 0.1 % of the sediment.

Where it sits is the second half of the claim, and it lands where colluvium belongs:
**71.3 % of the provenance mass is in the lower five drainage deciles** — the
hillslopes — and decile 0 alone holds 228,269 m of the 422,703 m archive at **69.03 %**
provenance, the highest of any decile.

### Are colluvium and alluvium distinguishable in the record?

**Not by a label, and that is `stubs.md` #25.** `DepUnit` is sixteen bytes with no
padding left, so a `mover` byte would cost roughly +42 MiB across 5.5 M units; the free
version is a packed `(species, mover)` byte and it wants to land with § 13.8's lineage
history. Today `arriving_species` sums the two mixtures and takes one argmax, which is
the right answer to *"what is this made of"* and no answer at all to *"who brought
it"*.

**By signature, yes, and it is measured.** Sortedness is the discriminator, and the
probe reports it as distinct species per recorded column, split hillslope/valley:

| distinct species per recorded column | anonymous | identity |
|---|---|---|
| hillslope columns (deciles 0–4) | 1.256 | **2.061** (+64 %) |
| valley + trunk columns (deciles 5–9) | 3.167 | **3.504** (+11 %) |
| hillslope/valley sortedness ratio | 0.396 | **0.588** |

The hillslopes are where the change lands, and by a factor of six against the valleys.
Decile 0 alone goes **1.091 → 1.952**: those columns were nearly single-species — a
thin record of whatever the local environment implied — and are now genuinely mixed,
which is exactly what *poorly sorted* means. The valleys were already receiving several
things, so they barely move. **That differential is the colluvium/alluvium contrast,
and it is the thing the slice bought.**

And the composition of the whole archive moved, which is the same finding at grid
scale:

| species | anonymous | identity | delta |
|---|---|---|---|
| fine clastic | 401,892 m | 109,357 m | **−72.8 %** |
| coarse clastic | 9,070 m | 151,423 m | **+1569.5 %** |
| organic soil | 24,058 m | 158,723 m | **+559.7 %** |
| peat | 5,353 m | 3,003 m | −43.9 % |
| charcoal | 204 m | 197 m | −3.4 % |

Under anonymous creep the record was 91 % fine clastic, because `litho_of_tag` reads a
low-energy environment and answers "mud". It is a quiet place, so it must be mud. But
a hillslope is not quiet *because nothing is happening* — it is quiet because what is
happening is **creep**, and what creep delivers is whatever is upslope: basement
detritus off a stripped ridge (coarse), reworked soil, and the mixture of everything
that was already lying there. The old record was answering a question about **energy**
with a claim about **material**, everywhere, for 91 % of the archive.

### Mass, per species

| audit | measured, over every cell/species/epoch of the run |
|---|---|
| creep itemisation vs the metres the terrain actually moved | **3.186 × 10⁻¹⁵** |
| any species created or destroyed by creep | **7.066 × 10⁻¹⁶** |

Both are at f64 summation-order noise, three orders under the 10⁻¹² gate, and the
second one is the interesting one: it is not a tolerance the arithmetic happens to
meet, it is the residue of *re-adding the same numbers in a different order*. The
per-edge quantities cancel exactly.

## The defect the magnitude surfaced

The first production run with creep identity on failed
`charcoal_reaches_the_voxel_as_an_inclusion_never_as_a_stratum` with a voxel that was
**8/8 charcoal**. A fire bed is capped at 0.04 m — 0.356 of an eighth — so the
expression path had produced a stratum of a thing that only exists as a lamina.

The mechanism, once found, is embarrassing in the good way. Thin charcoal beds sit in
the near-surface window, so `outcrop_shares` reports a charcoal share, so creep carries
charcoal downslope — correctly; colluvium really does contain reworked charcoal. At a
cell with little other deposition, that charcoal **wins the mixture argmax**, and the
unit is recorded as `OrganicCharcoal`. Then the merge key (tag, chapter, species) is
identical the next epoch, and the next, and thin laminae accrete into a three-metre
seam of charcoal.

`Litho::as_deposited` already existed to prevent exactly this class of thing, and its
doc stated the rule in the singular: *"the one lithology that cannot be [a deposit] is
`Litho::Basement`"*. It is not the one. **Peat is made where it lies, coal is peat
cooked in place, and charcoal is a fire event** — a mover that picks any of them up is
carrying *detrital organic matter*, and what it sets down is carbonaceous mud with
plant fragments in it. That is `Litho::OrganicSoil`, which is what the slot is for.

This is `material-behavior.md` § 12's four-way test caught in the wild, and it is its
first live instance. Transported material answers **category 3, "that material,
*moved*"**; an in-place organic is a **category 1/2** formation-or-transformation
product. Filing a moved peat as a peat asserts that the peat *formed at the receiving
cell*, which is one row from § 12's named pathology — a unit filed under the wrong
category loses the edge, and with it the identity, mass and provenance chains the edge
would have carried.

**And the part worth carrying forward: it was surfaced by a magnitude, not by a test.**
The rule was equally wrong the day Movement 2b shipped. Nothing caught it, because the
fluvial pass moves 0.109 % of this world's sediment and a transported organic could
never win an argmax at that scale. Creep moves 918× more and the false claim became
reachable inside one run. So:

> **A rule can be wrong and unreachable at the same time, and "unreachable" is a
> property of the current magnitudes, not of the rule.** When a slice multiplies a
> path's throughput by three orders of magnitude, every latent rule on that path goes
> live at once — and the suite that was green yesterday only ever tested the reachable
> half.

That is corrections #57, and it is the third instance in three days of the same
family: #51's guard that could not see the case, #55's claim nobody stated so nobody
checked, and now an enumeration that was complete for the traffic it had.

## What a player sees

**This one is worth a walk, and it is the first slice in three that is.** journal/0109
and journal/0110 both moved goldens and changed nothing visible; this changes the rock
in most of the world.

- **Strata — the big one.** A hillslope cliff face was 91 % mudstone, because the rule
  was "quiet place, therefore mud". It is now a mixture dominated by coarse clastic and
  carbonaceous soil: sandy and conglomeratic bands off the ridges above, dark
  organic-rich bands where soil crept in, interbedded rather than uniform. Distinct
  species per hillslope column nearly doubled. A road cut should read as *bedded* where
  it used to read as *massive*.
- **Landforms — essentially unmoved.** Mean surface elevation −1895.08 → −1895.09 m,
  maximum 1286.8 → 1286.7 m. This is the same landscape made of different rock, which
  is the correct signature for an identity change riding a mass-conserving pass.
- **Resources — moved, and this needs eyes.** Peat mass in the archive fell 43.9 %, and
  the coal-seam census test had to be re-baselined because the strongest diggable
  low-energy seam is at a different site. Coal is *made* by the biotic layer and is not
  directly touched, but where it survives depends on what is deposited over and around
  it, and that changed everywhere.
- **Vegetation — indirect and untested.** The biotic layer reads parent material
  through the record; a hillslope whose parent is now coarse clastic rather than mud
  has different phosphorus and different waterlogging. Nobody has looked.

**Recommended walk stations:** a hillslope road cut (the interbedding), a scarp foot
(a colluvial apron beside a channel deposit — the facies contrast this slice bought),
and the re-baselined coal site. A tour map should run first; the strongest exemplar of
the apron/channel contrast is a headless search, not a guess.

## What it cost

| | anonymous | identity | delta |
|---|---|---|---|
| deep run (production world) | 30.48 s / 33.12 s | 31.86 s / 30.48 s | **+1.38 s, then −2.65 s** |
| resident | 169.05 MiB | 187.21 MiB | **+18.16 MiB (+10.7 %)** |
| recorded units | 5,542,653 | 6,732,988 | **+21.5 %** |

**The gen-time delta is under this machine's noise floor and is reported as two runs
rather than one**, because the second run put it on the *other side of zero*. That is
the honest answer — a third agent was building on the same box — and it is also the
useful one: whatever the species gather costs, it is smaller than the variance of
measuring it, on a ~31 s deep run.

The residency is the honest price and it is **all merge key**. `DepUnit` did not grow
by a byte; there are simply more units, because the species now genuinely varies from
epoch to epoch on a hillslope where it used to be a constant function of the tag.
journal/0110 paid +0.04 MiB for the same axis because the fluvial identity almost never
differed from the default — the axis was free precisely to the extent that it was
saying nothing. Now it says something, and 18 MiB is what saying it costs. Interbedded
colluvium is a real stratigraphic feature and the record is holding it.

Gen time is not a constraint here, and the point is that the pass is cheap enough to
disappear into the noise: the species gather is a third pass over the same four edges
with a seven-element split, fully parallel, and it is not measurable against the
run-to-run variance of the deep loop.

**Not paid, and measured rather than assumed:** recording creep as a gravity-caused
mover in the flow record — which would discharge `stubs.md` #18's constant `cause` —
would cost **535,363 donor faces × 8 chapters = 4.28 M entries × 16 B = 65.4 MiB**, on
a 187 MiB world. That is a third of the world's residency to inhabit a second enum
value, and it is not this slice's trade to make. `FlowCause::Gravity` has been
enumerated since FLOW slice 1 and nothing here forecloses it; the number is now on
record so the decision has one.

## The thing this entry is really about

Two slices ago the brief said *"this is the big appearance-changer"* about the fluvial
transport pass, and it was wrong by a factor of nine hundred. The pass that moves this
world's sediment was sitting in the same file the whole time, thirty lines further
down, called `diffuse`, moving 605,117 m of rock a run with no name on any of it.

Nobody was careless. **`diffuse` had never been asked what it was moving, because until
Movement 2b there was no vocabulary in which the question could be posed.** A scalar
has no composition to report. The instrument that could ask it was built by the slice
that failed, and the failing slice's real output was the number that pointed here.

So the shape of the last three days is: build the mechanism, measure it honestly, watch
the measurement reroute the arc, and find that the reroute was cheap because the second
member of a family costs almost nothing once the first one exists. Creep reused the
species multiset, the composition seam, the diffusion fluxes and the mass budget. The
only genuinely new thing it contributed is a **knob that is absent** — no competence
ceiling — and that absence is the entire reason a colluvial apron looks different from
a point bar.

That is the argument for § 13.2's family framing, cashed out. Building creep as its own
system would have had to *invent* the colluvium/alluvium contrast, and would probably
have invented it as a sorting rule with a constant in it. Building it as a regime of
one quantity got the contrast for free, from a term left out.
