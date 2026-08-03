# There are no deep cells

*2026-08-03 · the interfingering field report → the bench exhibit → the borehole
conversation → the stratigraphic-correlation design pass
(`docs/audits/2026-08-03-stratigraphic-correlation-design.md`) · assets
`0149-interfinger-contact-wall.png`, `0149-interfinger-contact-benchcut.png`*

> blogworthy: **reflexions in a deepsim codebase** — the day a mechanism shipped,
> was walked, was rejected, and was superseded-as-plan inside twenty-four hours,
> and why that is the process *working*; and **respect for earth processes** —
> the fix was not a better blur, it was asking how real geology continues across
> distance. Lens 1 rides along: the user's field report was sharper than the
> design audit's warning, and the design pass that followed executed a user
> sketch rather than an assistant invention.

## The report

P11 slice 3 dissolved the ~460 m straight member borders, exactly as its
acceptance criterion demanded — and the user, walking the world the same day,
rejected what replaced them: *"as predicted, i do not like this, visually. it
still creates sharp differences between regions, they just don't follow the
cardinal directions. what's worse, the difference is not only at the surface: it
extends all the way down the column."* The slice-3 design audit had flagged
precisely this risk (its F6: the near dither inherits the far tier's cell-wide
blend semantics the user had already rejected by eye), and its P-6 said the walk
would be the ratification. The walk arrived; the ratification is a rejection.

The mechanism reading confirmed at desk: the membership dither assigns each
column exactly ONE deep cell — forced by the mass coupling (record, regolith
depth, and ledger must name the same cell or loose material leaks at every
boundary) — so a "finger" is not a surface blend. It is a full-depth transplant
of a neighbouring cell's entire geological biography, sharp at column
granularity, merely no longer axis-aligned.

## The exhibit

A bench cut at the contact zone near the x ≈ 82,483 m cell edge (pose recorded:
feet (82490, 215.5, 13346) m, yaw 0, pitch +0.12) produced the day's second
finding, starker than the first. The wall shows banded sandstone/conglomerate
strata on the left — and on the right, at the same elevation, floor-to-rim:
`has_contents: false`. Not a different stratigraphy. **No record at all.** The
membership dither transplants record *extent*, not just identity — a column
boundary can be a vertical cliff between "the world has history here" and "the
world has none."

## The conversation that replaced the mechanism

The user, on the record-side question of neighbouring cells disagreeing about
thickness and stacking: *"a deep cell is a construct. in reality there are no
deep cells… barring any discontinuity feature… wouldn't we just join layer to
layer across boundaries? layers created at the same time in two different deep
cells should… blend? find a midpoint and smoothly grade their thickness, as
well as whatever the physical drivers etc does to facies."*

That is stratigraphic correlation — the geologist's cross-section between
boreholes, which is what our deep cells are: boreholes 460 m apart. And the
principle that came with it now governs the whole presentation stack:
*"ideally our default case, barring any physical drivers, is utterly smooth
interpolation between all deepcell boreholes. **non-smooth detail is refinement
content, and for the default deepsim plugin pack, it must model a process
honestly.**"* The 2026-07-19 ruling said sharp analytic edges are forbidden
("if i see a square boundary I'll scream"); this is its positive half —
smoothness is the unearned default, and sharpness must be *bought* by an
honestly modelled process.

## What the design pass found

Dispatched the same hour with the sketch verbatim as its anchor; three results
worth the record:

1. **The mass argument holds by linearity.** Blend the correlated beds and the
   regolith depth — a sum of thicknesses — blends identically: blend-of-sums
   equals sum-of-blends, exactly, in f64. The coupling that forced
   one-cell-per-column never applied to blending *the whole stack coherently*;
   it only ever forbade blending some of the reads.
2. **Correlation is a read-side change.** The recorder, the quantum and carry,
   the merge key — untouched. Every deep-time golden must stay bit-still
   through the build, which converts the scariest semantic change of the month
   into one with the strongest possible tripwire.
3. **The recommended rule needs no new state.** Partition each borehole's stack
   by chapter — the record's one globally shared clock, already funded in the
   packed unit — and correlate within a chapter by cumulative-thickness
   fraction. Pinch-outs and truncation wedges fall out of the arithmetic;
   nothing needs a rule for them. Correlation never *fails* on today's grid,
   because no honest discontinuity exists yet — so the failure door ships as a
   named seam whose heir is the future fault pass.

The membership dither retires as a plan; the `SubCell` stencil, the accessor
layer, and the packed record it shipped with survive as exactly the substrate
the correlation needs. A mechanism nobody voted *against* yesterday is gone
today because the user looked at it — and the parts of yesterday's work that
were built honestly turn out to be the foundation of its replacement. That is
what seam-first buys when it works.

The one question the pass refused to answer is the right one to leave open: is
the octaves' stochastic facies cut an "honestly modelled process" under the new
principle, or decoration? The geostatisticians say a truncated-Gaussian field
is *declared uncertainty about what boreholes cannot know* — their discipline's
own standard of honesty. The user's two rulings pull against each other there,
and only the user can weigh them.
