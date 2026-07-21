# 0046 — The scarp and the plateau: paying the sampling debt

Corrections #25 left a debt: every landform impression on record — mine and
the user's — was formed at the summit plateau and along a low-gradient
transect, the two flattest kinds of place in the world. S13 measured that the
median site falls ~56 m/km and the steepest ~790 m per 10 km, and nobody had
ever *looked* at that ground. This walk pays the debt, and it is also the
first walk of a **stock production world**: no flags — U8 flipped this
morning, so `cargo run --release -p dc-client -- --horizon 6` boots a
tectonic world with the corrected climate registration. The flip, documented
rather than gated, exactly as ratified.

## Finding the ground

S13's "steepest land cell" is computed, not recorded, so the roughness probe
ran first and printed it: voxel (−18383, 11769) = (−16545, 10592) m, a
coastal cell at ~65 m elevation falling 138.6 m over 1.75 km on the +x−z
diagonal — ten times the crest's relief at the same window.

The recon was four compass frames from 50 m above the seat. Every direction
carried structure: dense terracing climbing toward −z and +x, a long
descending ramp toward +z — and, on the northern horizon, something this
project has never photographed: **a distant slope wearing its climate
zonation as visible bands** — green, then bare brown, then a pale stone cap.
The lapse-rate veneer, expressed as landscape at 6 km. The horizon knob
(journal/0042) is what made it visible; the climate fix (journal/0043) is
what makes it sit on the terrain it describes.

## An instrument lesson, again

The planned money shot — from the top of the scarp looking down the fall
line — produced a nothing frame: a flat green field to the horizon. Looking
*down* a 5 % grade is the blind angle; the ground falls away and compresses
into the skyline. Slopes read looking **up** or **across**. Corrections
#18/#19/#25 keep converging on one rule: the instrument (or the vantage, or
the sample) must be able to see the question. The frame is kept
(`0046-steep-fall-line-6km.png`) as the negative exhibit.

## The scarp

From the basin floor — which sits at **−77 m**, below the sea datum, dry
because nothing renders water yet — looking back up the face
(`0046-steep-scarp-from-basin.png`): terraced bare **stone** climbing out of
the basin, a brown **dirt** stripe at the old shoreline band, **green**
upland above. The veneer's elevation banding (Stone below −35 m, Dirt to
−1 m, then the temperate rule) drawn as a coastline scarp. It reads as a
place — the first frame from this project that does.

Along the scarp line (`0046-steep-along-scarp.png`): a terraced flank
marching to the skyline, the dirt band on the far horizon. Real hillside,
real aspect, real structure — all of it deep-field signal; S13's attribution
says jitter contributes ~12 m of the ~80 m/km here.

## The plateau, revisited

Same protocol at the crest (4154, −14705), now 1291.6 m (the flip's isostasy
shifted the summit +3 m from 0040's world; the plateau survives):
`0046-crest-comparison-6km.png` is still the featureless prairie of 0040,
now proven from a 6 km horizon rather than a 1.2 km one. **Both halves of
corrections #25 are confirmed by eye**: the plateau is real, and the world is
not that. The world has scarps, ramps, banded horizons, terraced flanks —
at the places where the deep field is steep.

## What this means for the recalibration pick

The three S13 candidates now have their context photographs. The steep
places are *already legible* — the deep field carries them. The flat places
are *simulated plateaus* — honest geography, rendered faithfully. So:

- A/B (louder sub-km garnish) would texture the plateau but cannot give it
  shape it does not have — and the scarp does not need them.
- C (finer deep tier) is the only candidate that changes what landforms
  *exist* between 460 m and 7.4 km.
- The real question the pictures pose is sharper than any candidate: is the
  460 m deep field's terrain, expressed honestly, *enough* — and where it is
  not, the fix is more simulation, not more noise. That is the Expression
  doctrine applied to relief.

Assets: `0046-steep-recon-{negz,yaw90,posz,yaw270}.png`,
`0046-steep-fall-line-6km.png` (the negative exhibit),
`0046-steep-scarp-from-basin.png`, `0046-steep-along-scarp.png`,
`0046-crest-comparison-6km.png`.

> blogworthy: the sampling-debt arc (#25 → this walk) — how a project
> convinced itself its world was flat by measuring only its summit, and what
> the world looked like when someone finally stood somewhere else.
