# 0030 — The flip you cannot see

*2026-07-20. The user ratified turning erodibility coupling on: "flip it, i want
to see." So: one line of config, three gates, and eight photographs taken twice.
This entry is the photography and the appraisal, not the mechanism — the
mechanism is 0029.*

*The slug is what I believed when I started writing and I am leaving it, because
the entry is now partly about how I got there. The corrected finding is narrower
and more useful: **the surface changed everywhere and improved nowhere.** What
you cannot see is a landform. Corrections #18.*

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

Shooting both is the only reason this entry says what it says — though not for
the reason I first thought, and the wrong turn is the most useful thing here.
Diffing the before/after pairs pixelwise:

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

My first reading of that table was: the lit passes were taken minutes apart, the
**sun had moved**, and the fullbright column is therefore the honest one — the
terrain barely changed. I wrote it up that way and committed it.

The user read it and said: *"the sun does not move whatsoever today."* There is
no day/night cycle. My mechanism did not exist.

So I measured instead of guessing, twice:

- The lit difference **survives 16×16 block averaging** — mean |Δ| per block 21.2
  (flank), 32.5 (upland), 11.7 (lowland). It is not per-voxel texture noise.
- Its **signed** mean is ≈ 0 — −0.3, −0.9, +0.1 of 255. It is not a brightness
  shift. Broad regions got brighter and other broad regions got darker, in
  balance.

Bright-here-dark-there with no net change is the signature of **face orientation
changing**: surfaces that were top faces became side faces, and the reverse. Not
lighting. Geometry.

Then I looked at `0030-flank-before-fullbright.png` properly, which I should have
done before trusting it. It is a **featureless grey field.** An entire terraced
hillside — every step of which is plainly visible in the lit frame of the
identical geometry — renders as one uniform mass. Of course it does: fullbright
is unlit pure vertex colour, so every face of a block is the same colour, and on
terrain made of *one material* there is nothing left to see. The 0.27 % never
meant "the shape didn't change." It meant **"this control is blind to shape."**

I had inverted the reliability of my two controls and then reported the blind
one.

> blogworthy: **the control that is blind to your question will report "no
> change" forever.** `--fullbright` is mandatory here because in 0027 it proved
> a material was correct when the renderer had crushed it to black — a *data*
> question, which it answers perfectly. I carried that trust to a *geometry*
> question, where the directional shading it strips is the only thing
> distinguishing one face of a voxel from another. Same flag, same rule, right
> answer and then dead wrong, and the tell was sitting in a committed PNG that I
> had diffed but not looked at. Recorded as corrections #18.

## What actually moved

I re-sampled the same 110-point lattice (5 km spacing, 20–65 km × 40–90 km,
across the main massif) before and after:

- total relief **2,615 m → 2,614 m**
- per-sample change: **min −2 m, max +2 m**, mean −0.34 m, mean |Δ| 0.70 m
- **38 of 110 samples did not move at all** at 1 m rounding

At all three ground vantages the walking surface dropped by exactly one voxel
(0.9 m): 1947.65 → 1946.75, 1647.05 → 1646.15, 1015.25 → 1014.35.

So *at landform scale* the world barely moved, and what it did was sink about one
voxel — a mean, not a differentiation. 0029 knew this: it reported aggregate
relief +2 m and built a uniformly-weakened control precisely to separate
"differentiates" from "erodes a bit differently overall."

But the lit frames say the **ground-level surface changed a great deal**, and
those two facts are not in tension once you see the mechanism. Elevation is
continuous; the voxel surface is quantized at 0.9 m. A sub-voxel elevation change
does not move a slope — it moves the *rounding*, and the rounding is what decides
whether a given 460 m cell reads as a flat bench or a step, and which of its
faces point up. Shift a whole hillside by less than one voxel and you re-cut
every terrace on it while moving the landform by nothing. That is what the
measurements describe together: ±2 m at 5 km spacing, unchanged silhouette, and
21–32 units of broad-region shading change on the slopes.

Two honest caveats. My lattice is 5 km; the deep cell is 460 m, so this sampling
is blind by construction to the cell-to-cell contrast 0029 measured (+3 to +12 m
between adjacent cells, one contact inverting a contour). And the surface a
player walks is the collapse layer's, several transforms downstream of
`DeepField`.

## The appraisal, undecorated

**Did it change? Yes — more than I first reported.** At the three ground
vantages the lit view is substantially rearranged: terraces sit in different
places, different faces catch the light, the near ground reads differently. This
is real and a player standing there would be standing somewhere subtly different.

**Is it better? No.** Different is not better. The change is a *re-cutting of the
same voxel terracing*, not the appearance of anything new. The `massif`
silhouette is unchanged in both passes (0.08 % fullbright, 1.07 block-mean lit) —
the same flat-topped dome with the same single steep right flank. The `flank` is
still an even grey ramp, just a differently-stepped one. The `lowland` is still
relentless green terracing.

**Are ledges or benches visible at hard beds? No.** Not one I can attribute to
lithology, at any of the four vantages. The terraces that moved moved everywhere,
including across ground that is a single rock type top to bottom — that is the
0.9 m quantization re-rounding, not a hard bed outlasting a soft one. The
`upland` middle distance has slightly more and taller dark step edges after the
flip, which is the closest thing to a positive, and I would not defend it as a
bench.

**Is basement legible as a resistant core? No.** The `upland` site sits on
granite three voxels down — exactly the stripped-upland prediction, and *true in
the data*. It reads as a grey plateau before and after, because what would make
it read is standing proud of its surroundings, and it stands proud by about a
voxel.

**So: the world moved, the landscape didn't.** This is the outcome 0029's own
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
