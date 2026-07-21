# 0041 — Five percent of the roughness budget

> blogworthy: a fractal terrain schedule that spends 95 % of its amplitude on
> four octaves it then throws in the bin, and a bug in the measuring instrument
> that produced a beautiful, self-consistent, entirely wrong set of tables
> before a three-line round-trip check caught it.

journal/0040 walked the highest crest of the first `--tectonics` world and came
back with a number that felt like an accusation: **7.2 m of relief over 1.75 km,
identical to the decimetre at `--amplitude` 80 and 160**, while the continent
itself had risen a kilometre. It filed the mechanism as a hypothesis
(corrections #23) and refused to believe it — correctly, by its own rule: a
mechanism is a hypothesis, only numbers are evidence. This entry is the numbers.

Two of the three things we suspected turned out to be true. The third was
false, and there was a fourth nobody had named.

## Reproducing the accusation

The first job was to get the walk's own number out of a headless probe, because
without that anchor every subsequent table is a story about a different world.
`Pregen::run_with` (the door journal/0039 opened) takes the same seed the client
boots with — `BENCH_SEED = 1337` — the same Medium extent, the same overrides.
Sample eight surface heights at 250 m stride across (10500, −15000) m and print
them.

```
walk (0040):  1287.1 1288.0 1288.9 1286.2 1281.7 1281.7 1285.3 1286.2
probe:        1286.1 1287.0 1287.9 1285.2 1280.7 1280.7 1284.3 1285.2
```

Every sample **exactly one metre low**, in both worlds, at both amplitudes. The
client's pose sits a metre above the surface voxel it reports; the shape,
spacing, and 7.2 m relief are identical. The walk went east. Anchor holds.

That one-metre offset is worth more than it looks. It is the difference between
"my headless number resembles the walk's" and "my headless number *is* the
walk's, plus a known constant" — and only the second lets you argue from the
decomposition afterwards.

## The instrument lied first

Before any of the real tables, the probe produced a full set of beautiful ones
that were wrong.

The measurement needs to move between three coordinate systems: world voxels,
the 17×17 pregen cell grid, and the 545×545 deep-time grid. `DeepField` has a
private `deep_coords` that does it; the probe had to re-implement the inverse to
turn "the world's highest deep cell" back into a place to stand. And
`deep_coords` centres the pregen grid with **integer** division — `(wp / 2) as
f64`, which for a 17-cell grid is 8, not 8.5. The probe wrote `wp as f64 / 2.0`.

Half a pregen cell is **7.4 km**. Every derived site was measured 7.4 km from
where it was labelled. The tables were internally consistent, plausible, and
described the wrong ground — the "crest" row reported a place 85 m below the
crest, and the raw-grid statistics printed alongside it disagreed with the
sampled surface by a factor of 20 without ever contradicting themselves.

What caught it was not suspicion, it was a round-trip: print, for each site, the
deep cell's own stored value, `surface_at_voxel` at that voxel, the zero-jitter
surface, and the final walked surface. Four numbers that must agree. They
differed by 85 m, and there is no interpretation of that except a broken bridge.
The check now ships in the probe's output and the results doc tells the next
reader to look at it first.

> The general lesson, which is not new but keeps arriving in new clothes: a
> measurement harness needs its own falsifier. Corrections #18/#19 say pick the
> instrument that can see your question; this adds *and make the instrument
> assert it is pointed at the thing you named*.

## Where the roughness goes

The elevation lattice refines by midpoint displacement over 15 levels, from
16 384-voxel cell corners down to single voxels, and each level's jitter is
`rough · 0.55^level`. `rough` is `provenance_roughness` — 90 m for craton,
420 m for orogeny.

Measuring the jitter actually injected at each level (child minus its own parent
average, ~215 points per level, parity rules mirrored from the shipped code)
reproduces `rough · 0.55^L / √3` at every level to two decimals. The schedule
does exactly what it says. That was never the question.

The question is what happens at **level 5**. Level 5 is `L_DEEP` — the locale
scale, 460.8 m, matched deliberately to the deep-time tier's own 460 m cell. At
that level the analytic elevation is *discarded* and replaced by
`DeepField::surface_at_voxel`. The docstring is careful and correct about why:
the deep-time surface should drive macro-terrain, the finer levels keep their
jitter, "so no sub-locale detail is lost".

No sub-locale detail is lost. All the **supra**-locale detail is, and that is
where the amplitude lives:

| level | wavelength | jitter injected | fate |
|---:|---:|---:|---|
| 1 | 7.4 km | 66.9 m | discarded at level 5 |
| 2 | 3.7 km | 39.5 m | discarded at level 5 |
| 3 | 1.8 km | 22.7 m | discarded at level 5 |
| 4 | 921 m | 13.9 m | discarded at level 5 |
| 5 | 461 m | — | **replaced by the deep-time surface** |
| 6 | 230 m | 4.1 m | survives |
| … | … | … | survives |
| 14 | 0.9 m | 0.03 m | survives |

**5.0 % of the scheduled roughness budget reaches the ground.** Exactly
`0.55⁵ = 1/19.8`, at every site, for every provenance. An Orogeny column is
scheduled 513 m of roughness and can physically deliver at most 25.7 m of it —
about 8 m at one sigma.

The measured 7.2 m is not a defect. It is the design point of a schedule that
was written before `L_DEEP` existed and never re-derived after it arrived. The
decay constant is indexed by *absolute* level, so it is still paying off the
four octaves that the deep tier now owns, and it arrives at the first octave it
actually keeps having already spent 97 % of its amplitude.

## The bilinear was innocent

corrections #23's other suspect was `surface_at_voxel` itself: a bilinear sample
of a 460 m grid, read by the lattice at 460.8 m spacing. Near-Nyquist resampling
of an interpolated field *looks* exactly like a low-pass filter.

It isn't one, measurably. Over a 20.7 km box at four sites, the level-5 lattice's
relief and mean cell-to-cell step reproduce the raw deep grid's to within 0.4 %
and 0.3 % respectively:

| site | raw grid relief / mean step | level-5 lattice |
|---|---|---|
| crest | 258.79 m / 4.301 m | 258.38 m / 4.312 m |
| steepest | 1579.74 m / 18.051 m | 1581.34 m / 18.073 m |
| walk-0040 | 185.07 m / 3.852 m | 184.43 m / 3.859 m |

Bilinear is exact at cell centres and the two spacings differ by 0.17 %, so the
lattice sees essentially every value the grid holds. There is no deep-time detail
being smoothed away below 460 m **because there is none in the source to
smooth**. Falsified, cleanly, which is the good outcome.

## The fourth mechanism

The site-selection decision that mattered most was not measuring only at the
crest. The world's highest deep cell is *by definition* a local maximum: its
neighbourhood is flat by construction, and a probe that samples only there
manufactures its own conclusion. So the probe measures four sites — crest,
steepest land cell, median land cell, and journal/0040's literal coordinates.

That is what exposed the thing nobody had named. The world has **two regimes**:

| window | walk-0040: deep / lattice | steepest: deep / lattice |
|---:|---|---|
| 100 m | 0.06 m / **5.47 m** | **7.59 m** / 3.58 m |
| 1 km | 1.00 m / **19.84 m** | **78.24 m** / 11.96 m |
| 10 km | 42.24 m / 22.78 m | 792.84 m / 13.48 m |

At the steep and median sites the erosion sim supplies 51–78 m of relief per
kilometre and the fractal layer is a garnish. At the two summit sites it supplies
**0.7–1.0 m per kilometre** — adjacent 460 m deep cells differ by 0.29 m, a
**0.065 % grade**, sustained across a 10 km box that varies by under 60 m. The
sim has built a genuine high plateau, and it is the highest land in the world, so
it is precisely where a player who goes looking for mountains arrives.

journal/0040 did not walk a broken world. It walked the flattest interesting
place in a world that is not uniformly flat, and it walked it because that place
is the summit.

## What this changes

The standing sequence said "the amplitude call is the last live cause of dismal
mountains". corrections #23 already killed that. What S13 adds is that the
replacement — "it's the lattice's decay" — is *also* only two thirds right:

- fixing the decay fixes the **100 m – 500 m** band, which is what a walker's
  near field is made of, and it costs nothing (same draws, same lookahead);
- it cannot make a plateau into a range. That is an erosion-supply and deep-tier
  resolution question, and it is a different piece of work.

Three candidate calibrations are written up in `docs/spikes/S13-results.md` with
predicted relief at every window size and the tradeoff each one buys. None is
flipped on. The binding constraint on all of them is `s7_walk`'s
`SEAM_TOLERANCE_VOXELS = 6` — adjacent voxel columns may not differ by more than
6 voxels — and the walk test prints its own headroom: **max interior step 2**.
Because per-level slope contribution *grows* toward the fine end, a uniform 3.3×
boost lands at 6.6 and fails that assert, while a band-limited one (levels 6–10
only, tapering to unity by level 11) leaves the finest levels alone and passes.
That asymmetry is the whole design space, and it is only visible because the
decay table was measured level by level rather than argued about.

One candidate was rejected outright by a number, and it is the one that sounded
best: deriving jitter amplitude from the deep field's own local gradient, so
plains stay smooth and mountain flanks roughen, self-scaling with the sim. The
measured local gradient at the summit is 0.29 m per 460 m. That rule would drive
the summit's roughness to zero and make the exact photograph journal/0040 took
strictly worse. Recorded so nobody re-derives it.

Assets: none — this entry is arithmetic. The pictures come after the user picks
a calibration, which is the point.
