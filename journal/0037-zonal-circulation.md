# 0037 — The zonal circulation profile: one object for two defects

The climate had a three-way switch where it should have had a curve. `wind_dx`
returned `-1`, `+1`, `-1` — easterly trades below 30°, mid-latitude westerlies to
60°, polar easterlies above — and *nothing in between*. At the 30° and 60° band
boundaries the prevailing wind reversed direction in a single grid row, at full
strength, with no transition. The moisture march ran one way on row *k* and the
opposite way on row *k+1*; the eolian agent (journal/0034) marched dune fields
one way here and the mirror way one cell north. On flat ground that prints a
**dead-straight, grid-aligned line** across climate, vegetation, and dune
orientation — the exact "if I see a square boundary I'll scream" failure the
method doctrine (earth-processes.md § method rule 5) forbids, except this seam
isn't a facies contact you can dither away. It's the wind itself.

The second defect was quieter and took longer to see: **there was no subsidence
aridity at all.** Every dry place in the old model was dry for exactly one
reason — a mountain upwind had already wrung the air out (rain shadow). But
Earth's greatest deserts — the Sahara, the Arabian, the Kalahari, the Australian
interior — aren't in anyone's rain shadow. They ring the globe at ~30° because
that is the **descending limb of the Hadley cell**: air that rose and rained at
the equator, travelled poleward aloft, and sinks back down warm and dry around
30°, suppressing rain wherever it lands, mountain or no mountain. Our worlds had
no such belt. A player who knows Earth would look at 30° and find it as wet as
40°, which is wrong in a way that reading the landscape can't explain.

The insight that made this one slice instead of two: **these are the same
object.** The calm belt at 30° — the horse latitudes, where sailing ships stalled
for want of wind — is calm *because* the air there is sinking rather than
blowing along. The descending limb is the calm belt is the desert belt. So a
wind magnitude that eases through zero at 30° and a precipitation suppression
that peaks at 30° are two readings of one atmospheric fact. Build one profile and
both defects die together.

## The wind, as a curve

`zonal_wind(lat)` returns a signed magnitude in `[-1, 1]`: sign is direction
(negative easterly, positive westerly), magnitude is strength normalized so the
trade easterlies — Earth's strongest surface cell — read `1.0`. Each of the three
cells is a **squared-sine lobe** over its band: `sin(π·phase)²`, which is zero
*and* has zero slope at both edges. Squaring is the trick that matters. A plain
`sin` lobe hits zero at the boundary but arrives with a kink; the square arrives
flat, so the whole three-cell profile is C¹-continuous across 30° and 60°. The
sign can only change by passing through a near-zero magnitude — never as a step.

The band interiors keep the historical directions exactly (easterly at 15°,
westerly at 45°, easterly at 75°), so nothing that was working downwind of a band
interior moves. Only the hard flip is gone, replaced by a calm belt. With the
squared-sine lobe `|wind|` drops below 10% of peak within about ±3° of each
boundary, i.e. a ~6°-wide calm belt straddling 30° and 60° — which is about the
real breadth of the horse latitudes and the polar front. Amplitudes ride the
Earth ratio of surface means (trades ≳ westerlies ≫ polar easterlies, roughly
7:6:3 m/s → `1.0 : 0.9 : 0.45`), and the eolian agent reads `|zonal_wind|`
directly as its deflation strength, so dune fields *fade* into the calm belts
rather than reversing across them. The scream is gone by construction.

## The desert, as a suppression

`subsidence(lat)` is a precipitation-suppression factor in `[0, 1]`: two Gaussian
descending limbs, a strong one at 30° (σ = 7°, so the desert belt spans ~22–38°,
matching Earth's ~15–35° subtropical deserts) and a weak, broad one at the pole
(σ = 12°, reaching the grid's 78° north edge as a mild polar-desert hint). It is
deliberately **near-zero at the equator** — the ITCZ is the *ascending* wet belt,
and it must stay wet — and **near-zero at 60°**, which is the crucial asymmetry:
60° is *also* a wind calm belt, but there the air is *rising* (the polar front, a
storm track), so it is a *wet* calm belt, not a desert. Of the two calm belts,
only 30° is dry. Encoding that difference is the whole point: the model now knows
*why* each calm belt is calm.

The moisture march applies it. The land-cell rainout — shared verbatim between
the pregen march and the deep-time re-march, so paleoclimate and present climate
can never drift apart in mechanism — became:

```
precip = max(orographic, floor) · (1 − subsidence(lat))
```

The subsidence gate multiplies through everything, so the 30° limb and the poles
fall below the pipeline's 0.32 arid threshold *with no mountain in front of
them*, while the equator and 60° keep their wet baseline.

### The calm-belt march, and the floor that cost the most thought

The march is a 1D advection: moisture blows in off the ocean and rains out along
the row. But where the wind eases to zero, *advection itself weakens* — a calm
belt has no persistent front carrying moisture across it. Left alone, the march
reads a calm belt as bone-dry (nothing blew in), which is wrong for the wrong
reason. The old model made this concrete and damning: flat interior land, marched
to steady state, settled at **0.06** precipitation — the equator, the temperate
belt, everything flat and ocean-far was a desert. That is not what "no mountain
here" should mean.

So the rainout carries a **local background floor**: a baseline that local
convection sustains regardless of what the wind delivers, applied as a *floor*
(`max(orographic, floor)`, not an addition, so a wet windward slope keeps its
orographic value). The question that cost the most thought was *how much* floor,
and *where*.

The first two answers were both wrong, and instructively so. A flat floor of
`0.38` (the minimum that keeps the equator wet) lifted **every** flat interior on
Earth to ~0.36 — a mean precip change of 0.19 over land, humidifying the world's
steppes and continental interiors that are *genuinely* semi-arid. Worse, that
extra interior water became extra river discharge (`width ∝ √discharge` in the
collapse layer), and a mid-latitude interior river carved a bank steep enough
(an 11-voxel step, over the 6-voxel seam tolerance) to trip the walk continuity
test at 43° latitude. Fixing bone-dry interiors is right; making the Gobi a
rainforest is not.

The honest floor is **latitude-shaped**, because background rain is not uniform —
it exists where air *rises*. Two ascending sources give it: the equatorial ITCZ
(deep tropical convection, a strong bump at the equator) and the ~60° polar front
(the mid-latitude storm track, a modest bump). *Between* them — the mid-latitude
continental interior at 40–55° — the floor is small, so those flats stay honestly
semi-arid. This keeps the equator wet and the 60° calm belt from going bone-dry
while leaving mid-latitude interior runoff (and the rivers carved from it) right
where they were: the walk's 43° line changed by −0.009 and the seam vanished. The
floor is still gated by subsidence, so the descending limb stays a desert; only
the orographic term draws the advected reservoir down (the floor is locally
sourced), so the rain-shadow property — lee dry *because* the range is there —
survives untouched.

## What changed in every world, measured

This profile is **upstream of everything** — biotic growth, erodibility, the
eolian agent all read precipitation — so it changes the climate of every new
world. That is expected and was ratified directly (no flip-flag: this is a
correction of the climate model, not an optional agent). Byte-identity was not
worth preserving here, so it wasn't. But the magnitude is the user's to see, so
it was measured: on a seeded Medium world, the mean absolute precip change over
land cells is **0.082** (of the 0..1 scale), and the latitude profile of the
change is exactly the mechanism's signature:

```
lat 18–28°: Δ −0.09      (equatorward shoulder of the desert belt)
lat 28–38°: Δ −0.18      ← the new Hadley desert: sharply drier
lat 38–48°: Δ −0.01      ← mid-latitude interior barely moves (the shaped floor)
lat 48–58°: Δ +0.04
lat 58–68°: Δ +0.06      ← poleward flats lifted a little by the storm-track floor
```

The signature is dominated by the 30° desert; the mid-latitudes barely move,
which is both the honest outcome (their interiors were already the wettest thing
the old orographic march produced, so the floor rarely bites) and the reason the
river-seam regression went away. The defect-death tests assert it directly: the
profile never
reverses sign without passing through near-zero magnitude (defect a); a ~30°
band on *flat* terrain reads below the arid threshold while its neighbours don't
(defect b, subsidence with no orography); the equatorial band stays wet (c); and
the existing rain-shadow property tests still hold (d, the lee is still dry
because the range is there).

## The coal seam that moved (and the half-cell that hid it)

The one downstream test that broke was instructive twice over. The organic suite
pins a coal-seam proof to a fixed voxel — the site S10 measured a 24 m seam,
later 17 m after the erodibility flip. After the climate change that exact cell
records **0 m** of coal. The reflex is alarm: did the correction destroy coal? It
did not. Coal is an emergent product of forty epochs of biotic feedback on top of
the climate, and shifting precipitation upstream moves *where* the thickest swamp
lands. A hard-coded location is a golden that any upstream change invalidates, so
the test now scans the world for its thickest coal seam and proves the pipeline
(biofacies-tagged → survives collapse as the COAL class → diggable `Block::Coal`)
wherever the climate now puts it. The world still grows a **28 m** diggable seam;
coal was never in danger.

The instructive part was the *second* failure, because it nearly sent me down a
false trail. The relocated test still failed — no seam appeared to survive
collapse — and every plausible story pointed at the climate change: the subtropics
dried, coal must have moved somewhere that buries it, the change is too
aggressive. All wrong. The diagnostic that settled it printed, for each candidate,
the coal thickness read two ways: straight from the deep grid (`strata[i] = 28 m`)
and through the voxel↔cell round-trip the collapse layer actually uses
(`record_at_voxel = 0 m`). The two disagreed, which can only mean the voxel I
computed for a deep cell landed on the *wrong* cell. The deep-field coordinate
map halves the pregen width with **integer** division (`wp / 2` = 8 for the
17-wide medium grid); my inverse used `wp / 2.0` = 8.5. Half a cell — 8192 voxels
— of error, enough to sample the neighbour. The bug had been latent: under the
earlier, wetter floor coal was so abundant that the neighbour cell usually had a
seam too, so the test passed for the wrong reason. The gentler shaped floor thinned
that luck out and exposed a mapping error that was mine, not the model's. The
lesson is the corrections.md discipline in miniature: when a confident causal
story ("the climate change buried the coal") and a two-line cross-check disagree,
believe the cross-check.

> blogworthy: the horse latitudes are the Hadley descending limb are the desert
> belt — three names for one column of sinking air — and how noticing that turned
> "smooth the wind" and "add subtropical deserts" from two features into one
> squared-sine profile plus one Gaussian.

## What a walk should photograph

The change is invisible to tests-as-numbers and loud to the eye, so it wants a
walk once the user is ready: fly the 30° latitude line and confirm a desert belt
that *isn't* behind a mountain, then cross a former hard seam at 30° or 60° and
confirm the climate/vegetation transition is now a gradient, not a straight
grid-aligned line. Both are lit-pass questions (shape and vegetation extent), not
fullbright. The one appearance call left open is the small poleward-flat
wettening (the ~60° storm-track floor, Δ ≈ +0.05) — whether the high-latitude
interior should read that green — and it rides as built pending that look.
