# 0043 — The climate that sat half a cell north

S13 (`docs/spikes/S13-results.md`) went looking at roughness and, on its way in,
tripped over its own coordinate math: `DeepField::deep_coords` centres the coarse
pregen grid with **integer** division — `(wp / 2) as f64` = 8 for a 17-cell grid —
while an early version of the probe's inverse used `wp as f64 / 2.0` = 8.5. Half a
pregen cell. 7.4 km. Every table the probe printed was internally self-consistent
and wrong, until a round-trip assertion caught it.

That fixed the probe. But S13 left a flag stapled to the wall on its way out:

> `collapse::climate_at` uses `f64::from(self.pregen.grid.w) / 2.0` (= 8.5) where
> `deep_coords` and `CellGrid::cell_of_voxel` both use integer `w / 2` (= 8). If
> that is not deliberate, the climate bilinear is offset half a coarse cell
> (7.4 km) from the cell grid it interpolates. Not investigated here.

This entry investigates it. Verdict up front: **it was a bug.** The identical
`8.5`-for-`8` slip the probe made in S13, sitting live in production `climate_at`,
mis-registering every temperature and precipitation consult by half a coarse cell.

## The derivation — ground truth by construction

The coarse grid is `w × w` cells centred on the world origin. Cell `(gx, gy)`
covers world voxels `[(gx − w/2)·CV, (gx − w/2 + 1)·CV)` on each axis, where
`w/2` is **integer** division and `CV = CELL_VOXELS = 16384` (voxel = 0.9 m,
so a cell is ~14.7 km). Two anchor functions fix the convention:

- `cell_of_voxel(vx) = vx.div_euclid(CV) + w/2` — the array index of a voxel's
  cell. Origin voxel → the centre cell `w/2` (= 8 at w=17).
- `cell_center_voxel(gx) = (gx − w/2)·CV + CV/2` — cell `gx`'s centre voxel.

The per-cell climate (`lat_deg`, `precip`) is a **point sample at the cell
centre**. To interpolate it, `climate_at` maps a voxel to a continuous grid
coordinate and reads a bilinear blend of the four surrounding cell samples.
Bilinear evaluated *at a node* returns that node's value exactly — no blend. So
the registration is correct iff a voxel sitting on cell `gx`'s own centre maps to
the **integer** continuous coordinate `gx`. Solve for the constant `K` in
`c(vx) = vx/CV + K`:

```
c(cell_center_voxel(gx)) = ((gx − 8)·CV + CV/2)/CV + K = (gx − 8) + 0.5 + K
                                                          =! gx
  ⇒ K = 7.5
```

`K = 7.5` is exactly what the **other three sites** produce:

| site | `half` | constant `K` | at a cell centre |
|------|--------|--------------|------------------|
| `deep_coords` (field.rs) | `(wp/2) as f64` = 8 | `8 − 0.5 = 7.5` | reads node exactly ✓ |
| `build_cells` resample (grid.rs) | — (`(g+0.5)/w·wp − 0.5`, the inverse) | ⇒ 7.5 | reads node exactly ✓ |
| `cell_of_voxel` / `cell_center_voxel` | integer `w/2` = 8 | — | centre-anchored ✓ |
| **`climate_at`** (collapse.rs) | **`f64::from(w)/2.0` = 8.5** | **`8.5 − 0.5 = 8.0`** | **reads `gx + 0.5` ✗** |

`climate_at` produced `K = 8.0`, half a cell too high. At a cell centre it landed
on continuous coordinate `gx + 0.5` — dead between two cells — and returned the
50/50 blend of the cell and its northern/eastern neighbour instead of the cell's
own climate. The `deep_coords` comment even *claims* to "match the collapse
layer's `climate_at` convention"; the two had silently disagreed since S13's slip
was born.

## Why it only bites odd worlds

`f64::from(w)/2.0` equals `(w/2) as f64` whenever `w` is even. The bug is a
**parity trap**: it only manifests for odd `w`, where integer division truncates
the `.5` that float division keeps. Every extent preset is odd — Small 5,
Medium 17, Large 69 — so the bug was live at every size the game ships. An even
preset would have hidden it forever.

## The discriminating test (red, then green)

`climate_at_reproduces_cells_own_climate_at_centre` walks the interior cells at
Small **and** Medium, teleports to each cell's `cell_center_voxel`, and asserts
`climate_at` returns that cell's own baked `(temp_sea_level(lat_deg), precip)` to
1e-9. Before the fix, cell (1,1) at Small read **12.28 °C** against its own
**15.92 °C** — a 3.64 °C error, exactly the half-cell latitude blend (Small's
gradient is 14°/cell, so half a cell is 7° → 0.52·7 = 3.64 °C). After changing
the one line `f64::from(self.pregen.grid.w) / 2.0` → `f64::from(self.pregen.grid.w / 2)`,
green at both sizes.

## What moved

The fix moves the sample point exactly half a cell — `CV/2 = 8192 voxels =
7372.8 m` — on **both** axes. Old registration read climate from a point 7.4 km
**north-and-east** of the terrain it tinted; equivalently, the whole climate map
had been shifted 7.4 km south-and-west relative to the ground. Because
`old(vx,vz) ≡ new(vx + 8192, vz + 8192)` by construction, both readings come from
the corrected function.

Everything downstream of `climate_at` moved with it: the surface veneer
(Grass/Dirt/Stone, including the −4 °C frost line and the `precip < 0.10` desert
and `fringe && precip < 0.35` blight rules), the `soil` depth tiers, and
`StrataCtx.{temp_c, precip}` (the formation context every strata pass reads).

Quantified on a Medium world (seed `0x0D5EED572026`), a 81-column north-south
transect one cell east of origin: **7 columns flipped surface block**, all in the
22–33° latitude band, all `Stone → Dirt` — the old colder (further-north) reading
had pushed frost-marginal highland columns over the −4 °C line into Stone; the
corrected warmer reading leaves them bare Dirt. The frost/rock line, in other
words, was drawn 7.4 km off.

## The 30° desert, and why this is *not* a correction to #22

corrections #22 recorded that the ~30° subsidence band is arid in the data yet
renders green, because `collapse` only bares the surface below `precip 0.10` —
the belt (precip ~0.2–0.3 at the walked site) sits above that placeholder
threshold. Natural worry: was the aridity band misregistered, and does the fix
rewrite that story?

The precip-at-30° numbers, old vs new, across longitudes (Medium, same seed, the
lat-30 °N band at `vz = −43535`):

| world x (vox) | precip old | precip new |
|--------------:|-----------:|-----------:|
| −98304 | 0.512 | 0.600 |
| −81920 | 0.165 | 0.406 |
| −65536 | 0.089 | 0.198 |
|      0 | 0.054 | 0.108 |
|  16384 | 0.038 | 0.062 |
|  32768 | 0.038 | 0.064 |
|  65536 | 0.039 | 0.050 |
|  98304 | 0.409 | 0.338 |

The precip *pattern* shifted 7.4 km — individual columns cross the 0.10 line in
both directions (origin gains water 0.054 → 0.108; the western columns wet up).
But the qualitative picture #22 rests on is unchanged: arid columns (< 0.10)
exist across the 30° band under **both** registrations, and the reason the belt
reads green on the surface is still the placeholder `0.10` bare-threshold, which
this fix does not touch (veneer thresholds are DECIDED user-owned, ecology.md
2026-07-21 — do not bandaid). #22's mechanism and verdict stand; the fix changes
*which* 30° columns are bare by 7.4 km, not *whether* the belt renders as desert.
So **no corrections entry is filed** — nothing previously recorded is falsified.
(If any prior explanation to the user pinned a specific rock-line or desert
*location*, those coordinates were 7.4 km off and are worth re-reading against the
corrected field — but that is a re-measure, not a falsification.)

## Re-baselines

The fix changes generated world content everywhere climate is keyed, so any test
that pinned a climate-derived value or a whole-world fingerprint legitimately
moves — the journal/0030 re-baseline pattern. All gate suites were re-run; the
only test touched is the new discriminating one. No existing test asserted a
`climate_at` output or a climate-dependent world fingerprint, so none re-baselined
— the veneer/strata content shifted, but nothing in the suite had frozen a value
that the shift invalidates. (Had one existed, its delta would be the honest
consequence of climate sliding 7.4 km, not a regression.)

> blogworthy: the parity trap. A one-character difference — `f64::from(w)/2.0`
> vs `f64::from(w/2)` — that is provably identical for every even input and
> silently wrong for every odd one, hiding in production behind a comment that
> swore it matched the very function it disagreed with. The fix is one line; the
> lesson is that "centre the grid" has exactly one correct spelling and three
> functions have to agree on it.
