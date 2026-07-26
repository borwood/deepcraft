# 0115 — The census that could not count its own defect

*2026-07-26 · the walk owed since journal/0112 — colluvium, the alluvial contrast, and the
pit field that turned out not to be a pit field*

> blogworthy: **lens 1 (AI-native development)** and **lens 4 (respect for earth
> processes)**. Three probes and one gated assertion agreed with each other about the size of
> a defect. All four were wrong, for the same structural reason, and none of them could have
> discovered it. A person flying over the terrain was right in one sentence. This is the
> entry where the walk stops being the ceremony that follows the measurement and becomes the
> instrument that corrects it — and where the acceptance criterion that licensed a
> calibration turns out to have been mathematically incapable of failing for the reason it
> mattered.

## Why there was a walk at all

Three slices had shipped without anyone standing in front of them: material-aware creep
(0112), hybrid `p` (0113), and the erosional calibration that ships switched off (0114). The
close block named the stations and said, in as many words, *whoever walks it should look at
the pits first.*

Doctrine says tour-map before spending live time, so the first thing built was
`walk_tour_0115.rs`: rank the strongest exemplar of each signature on the world the client
actually boots, print poses, and report a null honestly if there is nothing to see. It found
stations, and it also found — before anyone had launched anything — that the pit count in
`stubs.md` was measured on `mfd_routing`'s **small fixture**. On production-Medium the same
census returned **530**, against a control of **0** on the shipped world.

That felt like a good catch. It was the first of three numbers this entry has to withdraw.

## Station B and C — the facies contrast, and what the user saw that the probe did not

Two cut benches, identical geometry, 50 m standoff, `--fullbright --edges`: a hillslope
column in drainage deciles 0–2 (**73.88 m** of material whose recorded species disagrees with
what its depositing environment implies — colluvium, carried down the slope) against a trunk
column in decile 9 (**88.22 m** — alluvium, far-travelled and sorted), 5.2 km apart.

Twenty contiguous voxels down each face read almost identically: **8/8 carbonaceous
mudstone**, with a **1/8 pore-fill accessory** every ten voxels or so. On the hillslope face
that accessory was **sandstone**; on the trunk face, **conglomerate** — the coarse,
far-travelled end. The right materials in the right places, at one eighth of one voxel in
twenty.

The integrator called it a null: *the facies distinction exists in the record and does not
reach the eye.* The user, looking at the same two frames:

> *"Clearly, the conglomerate speckle appear in layers while the sandstone speckle appears
> more diffuse, and that does mean something."*

Which is exactly right, and it is the signature rather than the magnitude. A flow dropping
its coarse load as its competence ceiling falls deposits a **bed**. Creep smearing an
unsorted mixture down a slope deposits **scatter**. Two facies out of one mechanism with one
knob absent (journal/0112) — and the difference survived all the way to a voxel face, not as
contrast but as **texture**. The probe measured *how much* and reported not-enough; the eye
measured *arrangement* and found it.

That is the same distinction that decides the rest of this entry, and it showed up first
here, in a station nobody thought was important.

The user's own verdict on legibility was to accept it: *"at this resolution of course it does
look much the same… we could eat it since we compromise on voxel resolution."* Recorded as
ratified — the facies work is not blocked on appearance.

## Station A — the pits, and the sentence that broke the measurement

Relaunched on `--calibrated-rates`, **lit** pass this time, because a hole is a shape
question and fullbright is blind to shape.

The deepest hollow on the world sits at (41870, 12433). At the voxel tier its floor is
**391.5 m** against four cardinal neighbours at 473.4 / 477.0 / 480.6 / 513.0 — **81.9 m
below the shallowest**, and 130.3 m below all eight on the deep grid. From the rim it does
not read as damage at all. It reads as an **amphitheatre**: terraced walls, a flat dark
floor, the shape of a dry basin. **300 of the 530** were flagged by the depression fill as
recorded lakes, which raised a real design question — endorheic basins exist, and a world
eroding at a realistic rate is *entitled* to some.

Then the user flew the region:

> *"Looking over the region, it looks absolutely pockmarked with similar pits — roughly every
> cell has a deep depression. Honeycombed landscape."*

530 of 44,090 is 1.2 %. "Roughly every cell" is not a rounding disagreement with 1.2 %; it is
a different claim about the world. One of the two observations had to be wrong.

## The census saturates

It was the census.

*"Is this cell more than a metre below **every one** of its eight neighbours"* is a
**winner-take-all** predicate. It scores a cell only when its neighbours are higher — so as
the defect spreads, **neighbouring cells sink too and stop qualifying each other**, and the
count falls back toward zero exactly as the damage becomes universal. It is maximised by
*isolated* pits. It is structurally blind to a pockmarked one.

The gated assertion in `mfd_routing` uses that predicate. `stubs.md` #29 was sized with it.
The tour map re-implemented it faithfully. **Three instruments agreed because they were the
same instrument**, and the property they share is that they cannot fail informatively for
this failure mode.

The non-saturating instrument was already in the tree — the router's own depression fill,
`filled[i] − routed[i]`, which measures the hollow at a cell regardless of what its
neighbours are doing:

| | hollows >1 m | >10 m | >50 m | deepest | fill volume |
|---|---|---|---|---|---|
| shipped | **0** (0.0 %) | 0 | 0 | 0.0 m | 0.0 km³ |
| calibrated | **1,377** (3.1 %) | 817 (1.9 %) | 97 | 112.8 m | **5.4 km³** |

2.6× the saturating count, and a clean zero on the control — so the whole thing is
attributable to the amplitude and nothing else.

And it *still* did not reach "roughly every cell", because it too only sees **closed**
hollows. A bowl with a spillway scores zero on both tests and looks identical from the air.

## Concavity, and the sentence the whole session turns on

The user, without seeing any of these numbers, proposed the mechanism:

> *"Erosional solve could be bounded by cell and — because it's never been visible due to
> magnitude — probably is."*

That is a testable claim and it does not require closure. If the solve carves each cell
relative to its own edges, the **discrete Laplacian** of the surface — `mean(8 neighbours) −
self` — goes systematically large, whether or not any given hollow drains. On a smooth
landscape it averages ~0 and stays tight.

| | mean | p10 | p50 | p90 | p99 | >1 m | >5 m | >20 m |
|---|---|---|---|---|---|---|---|---|
| shipped | −0.14 m | −0.3 | −0.1 | +0.1 | +0.3 | **0.0 %** | 0.0 % | **0.0 %** |
| calibrated | +0.73 m | **−50.8** | −0.1 | **+52.8** | **+106.3** | **35.8 %** | 30.0 % | **23.2 %** |

The shipped world's **entire** concavity distribution fits inside ±0.3 m. The calibrated
world's deciles are **−50.8 and +52.8**, its p99 is **+106.3**, and its **median has not
moved**. Nearly a quarter of all land cells sit more than 20 m off the mean of their own
neighbours, with symmetric tails.

Regionally, within 10 km of the walk station: **7.8 % closed hollows against 3.1 %
globally**. So they cluster 2.5× as well — both explanations were true, and neither was the
main event.

The main event is one line:

> **Relief grew 4.6 %. Cell-to-cell roughness grew about 170×.**

A landscape that becomes genuinely more rugged does so by growing its **relief**. This one
grew its **grid noise** while its shape stood still. Symmetric ±50 m tails with an untouched
median is not terrain; it is **adjacent cells oscillating against each other**.

## The criterion that could not have failed

journal/0114 chose `EROSION_CALIBRATION = 45` against four criteria and named the first as
load-bearing: *"a multiplier that moves the shape has stopped being a calibration and become
a redesign. Relief within 5 %: +4.6 % at 45×. **This is the binding criterion.**"*

The claim is true. It licenses nothing.

Relief is `max(surf) − min(surf)` — a **global extremal** statistic over 44,000 cells. It is
mathematically incapable of detecting anything about the spatial *arrangement* between those
extremes: **you can shuffle every interior cell of a heightfield and leave relief exactly
unchanged.** So criterion 1 could not have failed for this reason no matter how bad the grid
got. It was not a weak test of shape. It was not a test of shape at all.

> **A criterion over a global aggregate cannot license a claim about local structure.** "The
> shape is preserved" is a claim about arrangement; relief, mean, min and max are claims
> about magnitude. Pair every aggregate criterion with a **neighbour-relative** one — a
> Laplacian, a gradient distribution, a spatial autocorrelation — or the acceptance test is
> measuring the axis that did not break.

This is journal/0111's lesson arriving a third time, one level further in. That entry:
*a closed system cannot detect its own scale error*, because every internal instrument checks
the sim against itself. journal/0114: *a system that has stopped cannot detect its own logic
errors*, because nothing exercises them. And now: **a statistic that integrates over the
world cannot detect a defect in how the world is arranged.** Each time the blindness is in
the *shape of the measurement*, not in anyone's care.

## What this does to the blocker, and the correction the integrator owes itself

`stubs.md` #29 is renamed and re-scoped. It was *the clamp that was green because nothing
eroded*, sized at 148 pits, diagnosed as the never-incise-below-your-receiver clamp being
defeated by the four phases that run after incision. That mechanism is plausible and
probably contributes — but it **predicts isolated deep holes**, and isolated deep holes are
the *tail* of what the world actually shows. A slice briefed against it would clamp the tail,
go green, and leave 23 % of cells 20 m off their neighbours.

The standing hypothesis is now numerical, and it is **flagged as a hypothesis in the entry
that carries it**, because this session has already been burned twice by mechanisms that
sounded right: an explicit scheme run past its stability limit. It fits a number journal/0114
measured and did not connect — creep's flux limiter binds on **89–96 %** of cells, so the
diffusion has saturated into *"move everything one cell downslope this epoch"*, and a
saturated explicit operator **overshoots**. Two cheap discriminators go in the brief:
sign-alternation of the concavity field, and halving `myr_per_epoch` at doubled epoch count
and fixed total time. If it is stability, the roughness collapses and the landscape does not
move.

That second discriminator is the `cell_m / myr_per_epoch` register — **the same one stubs
#27's heir (b) turns**. The two top blockers may share a fix, which nobody had noticed while
they were named "a clamp" and "a conveyor".

Three corrections were filed, and the integrator's own recording is in two of them:
**#61** (relief-within-5 % licensed nothing) and **#62** (the pit census saturates), plus the
withdrawal of the assistant's mid-walk claim that the fix belonged in the refinement arc. It
does not: refinement adds shape *between* cell centres and would render a ±50 m checkerboard
faithfully. You cannot interpolate your way out of a heightfield whose adjacent samples
disagree by fifty metres.

## The other thing the walk found

Flying between stations the user reported the far tier rendering **granite and diorite** where
the near surface is **andesite and basalt**, with a negative control attached in the same
breath: the gentler sandstone/conglomerate slope beside it does *not* do it. A later aerial
caught it at scale — a hard rectangular seam across the frame with different rock on each
side, too straight to be fog.

`stubs.md` #15 is adjacent and does not cover it: that stub predicts a far mesa's flank
wearing its **cap** material, and this is the far tier showing the rock *underneath*. Filed
to Observed with the pose, the material pair, and the control. It matters more now than it
did this morning, because `coarse_surface` is held to a **statistical** agreement test
against `ColumnFill` — a test a systematic steep-slope failure passes comfortably — and the
calibration produces exactly the geometry that triggers it: more relief, thinner cover on
steeper ground.

## What this entry is really about

The walk was scheduled as ratification: look at the thing, bless the appearance, move on.
What it did instead was **falsify the sizing of the top item on the board**, twice, and
retire an acceptance criterion that had already licensed a shipped constant.

Neither correction came from a better probe. The first came from someone looking at a
landscape and saying it was honeycombed. The second came from someone guessing the artefact
was bounded by cell — and being right, before any of the arithmetic existed to check it. The
probes' job was to turn both into numbers, and they did that well and quickly. But **the
probes could not have generated either claim**, because each of them was built to answer a
question that had already been framed wrong.

> *The walk is not what you do after the measurement. Sometimes it is the only instrument in
> the room that can see the question.*
