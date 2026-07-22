# 0056 — The walk that found three things, none of them the one we went looking for

*2026-07-21, live co-walk. The user in the world, the integrator driving
teleports. We went to ratify an appearance change. We came back with a
rendering bug, a quantization artifact, a missing material, and a correction
against the integrator.*

> blogworthy: every finding in this entry came from the person standing in the
> world, not from the agent driving. The agent had numbers, tests, and a green
> gate for all of it. What it did not have was eyes.

## The plan, and the instrument

journal/0053 and /0055 had changed the world twice in a few hours — soil depth
now from the recorded regolith plane, and the surface skinned by the deep-time
record instead of by a climate threshold. Both shipped with gates green and
neither had been seen. So: a live tour, `--fullbright --edges`, because the
questions were material ones (what is the ground made of, how deep does it dig)
and fullbright's flat albedo is the control that can see them.

The user's terms were simple: *"run the tour, i'll dig at every site."*

## Finding one: the ground had holes in it

The first station looked wrong in a way that took three screenshots to pin
down. Sky-blue bands ran across the terrain in a rectilinear lattice — not
missing chunks, not a streaming transient (two screenshots twenty seconds
apart are pixel-identical, `assets/0056-holes-after-settle.png`). You could see
*through the world*.

The integrator's hypotheses were a near/far material disagreement, then an LOD
ring boundary, then a streaming gap. All wrong. The user, who was flying around
in it, simply said:

> the bands you see are missing side faces. these partials mostly have no side
> faces - some of them do, following no apparent pattern.

That was the whole diagnosis, and it was correct. `meshing.rs` culled side
faces against a **block-tier boolean** — is the neighbour's block non-Air? A
5/8 partial beside a 3/8 partial therefore lost its entire side face, and the
exposed 2/8 band was emitted by nobody. Faces survived only where a partial met
air or something genuinely full: the "no apparent pattern".

The mesher's own module header had recorded the assumption that made this safe:
*"Worldgen does not yet emit sub-8 loose voxels, so partial heights are
exercised by tests until loose-material deposition lands."* True that morning.
False by lunchtime. journal/0057 has the fix and corrections #29 has the
lesson — a capability shipped dormant with an explicit "it will light up for
free" note, whose dependency lived in prose, and prose cannot fail a build.

## Finding two: the surface was quantized to chunks

Then, again from the user rather than from the tooling:

> on the top-most voxel, only one material wins, adding a per-chunk
> quantization of surface material (around here some chunk surfaces are salmon,
> some are tan). dig one-deep at any surface and the mix appears gradient and
> natural. so this has to do with something quantizing the surface mixture only.

`assets/0056-surface-quantized-per-chunk.png` shows it plainly once you know:
hard rectilinear patches on the 28.8 m chunk grid. The buried path re-picks its
host member **per voxel column** via the 3c-2 boundary dither — machinery this
project built specifically to wander family contacts off the chunk grid. The
surface path resolved **one member per chunk**, drawn at the footprint's centre.
Two members of one class share a block twin but not an albedo, so the patches
were the same *block* in different *materials*.

It was a regression of a fix we had already made once, in the one place nobody
had applied it. Fixed in journal/0058: chunk footprints expressing more than one
surface member went 0/169 → 147/169.

## Finding three: the world has no soil

The last one is the largest, and it came from the user noticing an absence:

> the world does not generate dirt anywhere apparently (was only a veneer
> placement previously)

Verified in the content set, and true. `MaterialId::LOAM` is defined and
registered in **no geology class member** — its only other appearance in the
tree is a unit test. The class that sounds like soil, `dc:stratum/organic-soil`,
has exactly one member: `dc:geo/carbonaceous-mudstone`, whose material is a
lithified **rock**. Every soil horizon the deep sim faithfully records is
expressed as mudstone, because mudstone is the only thing registered to express
it as.

This retired a question filed the same morning. journal/0055's world skins
91.4 % to one block, and the integrator had filed two competing explanations —
summarization hiding soil, or a record that never grew it — plus a measurement
to discriminate them. Both were downstream of something simpler: there is no
soil substance in the world to hide or to miss. `Block::Dirt` survives only in
fallback paths, which is exactly the user's read that it had only ever been
veneer paint.

It compounds with form. `fill::is_loose` covers clastic and placer only, so
organics land in the *structure* bucket and would render as solid cubes even
once a soil material exists. Two stacked gaps, and the fix order falls out:
substance first, then form-from-provenance.

## The correction: eight kilometres

The tour also produced a false alarm, and it was the integrator's.

Sampling the column at each station, the sections came back far thinner than
journal/0055 reported — 3 voxels where 89 were claimed. Two more sites also
under-expressed. It looked systematic, and it was escalated as such: possibly
the shipped world did not match the numbers the slice had been accepted on.

It did match. `soil_depth_probe::STATIONS` holds **world metres**, and divides
by 0.9 to reach voxels. The integrator multiplied, landing 8.6 km from every
station — a real place, correctly measured, 19 deep cells from the one on the
label. journal/0055 was right to the digit.

The instructive part is not the unit slip; it is the verification that failed to
catch it. Teleporting, the integrator checked that `pos_voxel` matched the
coordinates aimed at, and called the conversion confirmed. That test could only
ever answer *"did I land where I aimed"* — never *"is where I aimed the right
place."* A check that cannot distinguish the two hypotheses is not a check.
Corrections #28; a sibling of #25, where we walked the flattest place in the
world and concluded the world was flat.

## What the walk actually cost and bought

Cost: an afternoon at the wrong coordinates, and one escalation that briefly
questioned two shipped slices.

Bought: a rendering bug that every test passed over, an artifact that reversed a
fix we had already earned once, a missing material that reframes the world's
monotony from a presentation problem into a content gap, and a correction to a
verification habit. Also a null: the user's verdict on the deep section was
*"it reads quite mixed and comparatively deep compared to some other areas, all
in all good"* — the distribution-first work, at least, reads right underground.

Three of the four findings were the user's. The agent had numbers, tests, and a
green gate for all of it.
