# 0071 — Four stations and two honest nulls

*2026-07-22. The participative walk ratifying journal/0068's
thickness-dominance rule — the user in the world, the integrator driving
teleports and station briefings, verdicts recorded per station. Tour map:
`examples/tour_map_0071.rs`. The lean variant, at the user's direction: one
production run, no pre/post sim diff.*

> blogworthy (lenses: respect-for-earth, AI-native development): **how do you
> ratify a change that moves 56.5 % of a world?** Not by staring at goldens —
> by standing on the most-changed ground with the person who owns the look,
> predicting the nulls out loud before arriving, and treating "I see nothing"
> as a recordable verdict. Two of five planned readings were nulls, both
> predicted, both informative — and the walk's field reports (a rendering
> artifact, a quantization worry, a perf cliff) were worth as much as its
> verdicts.

## The map, and a number worth pausing on

The walk world (client seed 1337, Medium) flips **56.5 %** of recorded cells
under the dominance rule — more than the probe seed's 41.5 %. The rule's
blast radius is seed-dependent; the transition table's shape (thin records
going basement) is not.

Station 1 — the armored coast — **did not exist on this world**: no interior
coastal cell flipped to basement. A probe-level null, briefed as such before
the game even launched. Seed 1337's coasts simply are not thin-record coasts.

## Station 2 — the flipped plain (soil → basement, 98 % deficit)

The purest case on the map: 0.02 m of recorded soil, in two units, over
granite. The column scan under the user's feet read `granite` at the surface
with `basalt` beneath — the basement was always what you *saw* here, because
2 cm of soil never expressed a voxel. What changed is what erosion believes:
until 0068 this cell eroded as *soil* (the top unit's label — the exact
carve-out-shaped error, unnamed); now it erodes as the platform it visibly
is. Predicted eye-null, confirmed eye-null: a plain that stops eroding looks
like a plain. The verdict is the mechanism's, not the vista's.

The user then stepped in a hole and found a rendering artifact (below).

## Station 3 — the coal seam (PASS, trench cut by hand)

World (−11.9 km, 6.0 km), a red mudstone upland at 419 m. Column, top down:
14 voxels of mudstone, then **3 voxels of coal at y 449–451**, resting
directly on basalt over granite. The seam exists because this column's peat
took 11.8 m of section on top — the burial axis (0066), read through the
degenerate geotherm (0067). Outcrop here flipped `coarse → fine`: the upland
now erodes as the mud its window actually holds — the mirror image of
station 2's armoring, at the same walk.

The user cut the trench themselves: **"looks great."**

## Station 4 — the charcoal face (PASS, with a design observation)

World (−39.1 km, −23.4 km): the densest fire history on the map — 9 beds,
0.32 m total, mean 3.6 cm. The block palette of the column is pure
mudstone-and-air, which *is* the passing grade: a fire lamina must never be
a block. The specks in the cut face read as they did in 0066 — **"looks
reasonable"** — with an honest limitation attached: it is hard to tell
*which* dark flecks are charcoal because "everything is honestly so
blended." The user's sketch — give material heightmaps **characteristic
shapes** instead of random scramble, so heightmap splat blending becomes
decodable — is filed in ROADMAP Observed as a visuals/materials thread.

And the user's parting read of the station, worth keeping verbatim:
*"there's clearly a lot of history below my feet! much strata. so different
from the bedrock field."* That contrast — floodplain over fire history vs
armored granite plain — is the differentiated world the whole arc has been
building toward, read correctly from the ground by eye.

## The field reports (each filed in ROADMAP Observed, same day)

1. **Anisotropic texel-edge dither** on close-pressed walls: vertical texel
   boundaries dissolve into noise, horizontal stay razor — the signature of
   f32 precision exhaustion in world-position-derived UVs at |x| ≈ 99 km,
   with the user's competing splat-mixing hypothesis recorded alongside and
   one discriminating test (the same wall-press near world origin).
2. **The dominance flip quantizes smooth gradients**: a thinning deposit
   crosses the plurality threshold along a *coherent contour* — the S-4
   class of edge, unlike the old rule's noise-like flips. Blend-by-window-
   share filed as the continuous variant. No call made; watch future walks.
3. **Chunk gen time is noticeable in vertical streaming**: dropping from
   height makes time "slow to a crawl, sometimes" — a tick-contention
   symptom filed as observation, coupling forward to octree coarse-below.

## The verdict

The thickness-dominance rework **rides as walked**: two passes, two honest
nulls, zero defects, no tuning requested. The appearance-class ratification
the merge report owed the user is discharged — the world-wide outcrop change
is ratified from the ground, with the amplitude question (whether any of
this ever reads as *relief*) still parked exactly where journal/0030 left
it, now with a filed slice to make it walkable too.
