# 0030 — The flip you cannot see

*2026-07-20. The user ratified turning erodibility coupling on: "flip it, i want
to see." So: one line of config, three gates, and eight photographs taken twice.
This entry is the photography and the appraisal, not the mechanism — the
mechanism is 0029.*

## What was flipped

`production_config` in `deeptime/field.rs` now sets `erodibility: true`
alongside the S10 `biotic: true`. That is the whole behavioural change. No rate,
no contrast, no clamp was touched — the amplitude question is explicitly the
user's and is not this entry's business.

Every world created from here on has a different shape. Worlds created before
today are not reproducible under this build. That is the accepted cost of the
flip and it was the reason it needed ratifying.

## The method, and the one thing that saved it

Four vantages in the player's own world (seed 1337, Medium, N=2 — the client
takes no `--seed`, journal/0027), each recorded as an exact pose so it could be
re-occupied after the flip:

| tag | pose | what it frames |
|---|---|---|
| `massif` | (35000, 3500, 68500) fly, yaw 0, pitch −0.12 | the 3,304 m summit dome from 3.5 km south — the silhouette question |
| `upland` | (35000, surface, 50000), yaw π, pitch +0.05 | a stripped alpine upland: one voxel of stone, one of mudstone, then **granite basement** |
| `flank` | (30000, surface, 84000), yaw 0, pitch +0.05 | a 400 m hillside climbing into the range, bare rock outcropping |
| `lowland` | (−25000, surface, −20000), yaw 0, pitch +0.03 | the uniform-green plain at 1,015 m |

`eye_in_solid` was false in all sixteen pose replies. Each vantage was shot
**lit and `--fullbright`**, before and after — sixteen frames.

Shooting both is the only reason this entry says what it says. Diffing the
before/after pairs pixelwise:

```
                     differing px      mean |Δrgb|
massif   lit              8.08 %          2.35
massif   fullbright       0.08 %          0.19
upland   lit             37.13 %         38.27
upland   fullbright       0.93 %          0.57
flank    lit             48.93 %         37.45
flank    fullbright       0.27 %          0.21
lowland  lit             30.51 %         16.60
lowland  fullbright       8.37 %          4.35
```

Read the lit column alone and you would report a dramatic result: half the
`flank` frame changed. It didn't. The lit passes were taken minutes apart and
the **sun had moved**; almost all of that 48.93 % is a shading difference on
geometry that is nearly identical. The fullbright column is the honest one, and
it says 0.08–0.93 % at three of four sites.

> blogworthy: the fullbright rule earning its keep a second time. In 0027 it
> proved a material was fine when the renderer had crushed it to black. Here it
> stopped the opposite error — it stopped me reporting a terrain change that was
> really just the time of day. A control that catches lies in both directions is
> not a debugging convenience, it is the measurement.

## What actually moved

I re-sampled the same 110-point lattice (5 km spacing, 20–65 km × 40–90 km,
across the main massif) before and after:

- total relief **2,615 m → 2,614 m**
- per-sample change: **min −2 m, max +2 m**, mean −0.34 m, mean |Δ| 0.70 m
- **38 of 110 samples did not move at all** at 1 m rounding

At all three ground vantages the walking surface dropped by exactly one voxel
(0.9 m): 1947.65 → 1946.75, 1647.05 → 1646.15, 1015.25 → 1014.35.

So the dominant visible effect of turning on differential erosion is that the
world sank about one voxel. That is not differentiation; that is a mean. 0029
knew this — it reported aggregate relief +2 m and built a uniformly-weakened
control precisely to separate "differentiates" from "erodes a bit differently
overall" — but seeing it as *one voxel of subsidence* is a different kind of
knowing than seeing it as *+2 m of relief*.

Two honest caveats. My lattice is 5 km; the deep cell is 460 m, so this sampling
is blind by construction to the cell-to-cell contrast 0029 measured (+3 to +12 m
between adjacent cells, one contact inverting a contour). And the surface a
player walks is the collapse layer's, several transforms downstream of
`DeepField`. The claim here is narrow and it is the one that was asked for:
**at the shipped erosion amplitude, on the ground, with your eyes, you cannot
see this.**

## The appraisal, undecorated

**Is it better? No. It is the same.** The `massif` silhouette before and after
is the same flat-topped dome with the same single steep right flank — 0.08 % of
pixels differ in fullbright, which at this framing is a handful of voxels on the
skyline. The `flank` is the same even grey ramp. The `lowland` is the same
relentless green terracing, and it changed most of the four (8.37 %) without
changing character at all.

**Are ledges or benches visible at hard beds? No.** Not one, at any of the four
vantages, in either pass. The `upland` frame is the closest thing to a positive:
its middle distance has slightly more, and slightly taller, dark step edges
after the flip than before. I would not have noticed it without the diff, and I
would not defend it as a bench.

**Is basement legible as a resistant core? Not visually.** The `upland` site
sits on granite three voxels down, which is exactly the stripped-upland
prediction and it is *true in the data* — but it reads as a grey plateau both
before and after, because the thing that would make it read is standing proud of
its surroundings, and it stands proud by about a voxel.

**So: numerically real, visually negligible.** This is the outcome 0029's own
measurements predicted was possible — it found the effect modest until erosion
rates were raised 10×, at which point a hard bed stood 44.7 m proud. Nothing
here contradicts the milestone. The model is not the bottleneck; the amplitude
is. Cause 1 of the dismal mountains is genuinely closed, and closing it did not
make the mountains less dismal, because causes 2 and 3 are load-bearing and
still open. Dip is what turns a hardness contrast into a landform, and amplitude
is what makes it big enough to see.

If the user wants to see something, the next flip is the amplitude one, not
another appearance walk.

## Two tests changed, and why

The flip broke two tests. Both were legitimate, and neither was a bug.

**`dc-client::authority::worldgen_surface_seating_never_embeds`.** It asserted
that the voxel 2.5 m below `find_open_spawn`'s result is solid — checking the
*centre* column. But `true_surface_m` deliberately returns the **max over the
columns the body's footprint covers** (a body rests on the highest column it
straddles, documented in `worldgen.rs`). Probing the centre held by luck: the
world origin was flat. After the flip, column (0,0) tops out one voxel *below*
all four of its neighbours — I confirmed this from the running game, block by
block: (0,0) is dirt at y=1111 and air at 1112, while (±1,0), (0,±1) and
(−1,−1) are all grass/dirt at 1112. The seating was perfectly correct; the body
rests on the neighbours at 1001.7 m. The assertion now probes the footprint
corners as well as the centre, which is what the function's own contract says.

**`dc-worldgen::organic::the_measured_coal_seam_is_coal_a_player_can_dig`.** The
S10 coal site (world voxel 107338, 58787, on the headless seed) held a 24.03 m
seam with erosion lithology-blind. With coupling on it holds **17.04 m**, and 19
voxels of coal survive collapse instead of ~27. That is the flip doing its job:
differential weathering strips this seam's soft cover faster, so more of it is
gone by the end of the run. The test's subject is *routing* — coal is tagged by
biofacies, survives collapse as the COAL class, and arrives as diggable
`Block::Coal` — none of which is a thickness claim. Its three magnitude
thresholds were re-baselined from 20 m/20 vox to 15 m/15 vox as a floor on
"still a thick seam", with the old and new measurements recorded in the source
so the next reader knows the number moved and why. **This is a deliberate
loosening of a world fingerprint and it is the only one.**

Everything else in the suite passed unchanged, including all thirteen
erodibility tests and the byte-identity proofs.

## Files

`journal/assets/0030-{massif,upland,flank,lowland}-{before,after}-{lit,fullbright}.png`
(sixteen), this entry, ROADMAP (Shipped + Observed), and the two test files
named above.
