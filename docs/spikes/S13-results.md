# S13 results — where the terrain roughness goes

> **⚠ RE-SCOPED 2026-08-03 (doc-topology sweep F2, stamped by the integrator).** The
> option-C pricing below (§ "the 230 m deep-cell candidate", *"the only candidate that
> makes the summit plateau into a landform"*) was priced OUT against the pregen
> `<60 s` budget *"which is a ratified number"*. **That budget was renegotiated by the
> user 2026-08-02, 60 s → 1,200 s** (P11 slice-2 merge; `ROADMAP.md` § P11,
> `crates/dc-worldgen/tests/s7_measurements.rs`, `dependency-graph.md` P11 row —
> *"as long as it doesn't take 20min"*). Option C's cost objection is therefore
> expired; its merits were never re-examined under the new budget. The measurements
> below are unchanged and remain a dated record.

Status: measurement complete, 2026-07-21. **Measurement only — no production
behaviour changed.** The harness is
`crates/dc-worldgen/examples/roughness_probe.rs`; it reads the world through two
new read-only windows on `WorldGenerator`
(`surface_elev_m`, `lattice_point`) and `DeepField`'s public fields, and calls
nothing on a generation path. Numbers from the Windows dev box, release profile,
**seed 1337** (the client's `BENCH_SEED`, which every walk to date has been
driven through), `Extent::Medium` (17×17 pregen cells, ~251 km; deep grid 545²
at 460 m), `tectonic_history: true`, `thickening_scale` 80 and 160 — i.e. exactly
the two worlds journal/0040 walked.

Dispatched by journal/0040 + corrections #23: a walk across the world's highest
crest measured **7.2 m of relief over 1.75 km, identical to the decimetre** at
both amplitudes, while absolute elevation moved +714…+1079 m. The hypothesis to
test was that the collapse lattice's per-refinement amplitude decay attenuates
`provenance_roughness` (90–420 m) by one to two orders of magnitude, and that
`DeepField::surface_at_voxel`'s bilinear sample of a 460 m grid low-passes away
the rest.

**Headline: the decay hypothesis is CONFIRMED to the factor — 20×, exactly
`AMP_DECAY^L_DEEP`. The bilinear hypothesis is FALSIFIED — the 460.8 m resample
reproduces the raw deep grid's relief and mean step to within 1 %. And a third
mechanism nobody had named turns out to matter as much as either: at the summit
the simulated deep-time surface is itself a plateau, flat to 0.07 % grade over
10 km.**

---

## Method

- **Worlds.** `Pregen::run_with(WorldParams { seed: 1337, extent: Medium },
  DeepOverrides { tectonic_history: Some(true), thickening_scale: Some(80|160) })`
  — the exact path `dc-client`'s `--tectonics --amplitude n` boot takes
  (journal/0039). ~15.5 s per world.
- **Four sites, not one.** The world's highest deep cell is *by definition* a
  local maximum, so its neighbourhood is flat by construction; measuring only
  there would manufacture the conclusion. So every curve is reported at
  the **crest** (highest land cell), the **steepest** land cell, a
  **median**-elevation land cell, and **walk-0040** — the literal coordinates
  journal/0040 stood on, (10500, −15000) m.
- **Relief vs. window.** For each window `W` a 65×65 grid at stride `W/64` is
  laid over the site; the *union* of all nine windows' grids (37 629 distinct
  offsets) is sampled once, and relief at window `W` is `max−min` over the union
  points inside `W`. Nesting makes the curve monotone by construction and lets a
  50 km window still see 100 m detail.
- **Attribution.** Four surfaces at every point:
  - `final` — `surface_elev_m`, the shipped surface, rivers carved;
  - `lattice` — the same before river carving (`lattice_point(L_VOXEL, …)`);
  - `no-jitter` — bilinear over the level-5 lattice. Repeated midpoint
    *averaging* with the displacement suppressed **is** bilinear interpolation of
    the level-5 values, so this is the exact zero-jitter surface, not an
    approximation;
  - `deep-raw` — `DeepField::surface_at_voxel` sampled directly.

  `jitter = lattice − no-jitter` is then the refinement pyramid's own
  contribution, with rivers excluded from the attribution rather than smeared
  through it.
- **Per-level decay.** For each level, ~215 lattice points around the site
  (skipping the even/even points, which inherit their parent and are never
  jittered) are compared against **their own parent average**, recomputed in the
  probe with the same parity rules `collapse::lattice` uses. `rms |delta|` is
  therefore the jitter actually injected, not a restatement of the constant.

### A deviation, and the guard that caught it

The first run of this probe measured the wrong ground. `DeepField::deep_coords`
centres the pregen grid with **integer** division — `(self.wp / 2) as f64` = 8 for
a 17-cell grid — and the probe's inverse used `wp as f64 / 2.0` = 8.5, offsetting
every derived site by half a pregen cell (**7.4 km**). Every table it produced
was self-consistent and wrong. The fix was a three-line round-trip assertion
printed in § 5 — `surf[cell]` vs `surface_at_voxel` vs `no-jitter` vs `final` at
each site — which now agree to ≤0.1 m and are the guard that any future reader
should check first.

> **Noticed in passing, unmeasured, flagged for someone else.**
> `collapse::climate_at` uses `f64::from(self.pregen.grid.w) / 2.0` (= 8.5) where
> `deep_coords` and `CellGrid::cell_of_voxel` both use integer `w / 2` (= 8). If
> that is not deliberate, the climate bilinear is offset half a coarse cell
> (7.4 km) from the cell grid it interpolates. Not investigated here; not a
> roughness question.

**Resolved (journal/0043): it was a bug — the same `8.5`-for-`8` slip this probe
made, live in production.** `climate_at` now uses integer `w / 2`, matching
`deep_coords` / `cell_of_voxel` / `build_cells`. The mis-registration had shifted
every temp/precip consult 7372.8 m north-and-east of the terrain it keyed; the
frost line and desert band moved with it. Guarded by
`climate_at_reproduces_cells_own_climate_at_centre`.

---

## 1. The walk baseline — reproduced exactly

journal/0040's transect, headless. 8 samples at 250 m stride across the
walk-0040 site, quantized to the voxel surface the client's
`pose_set { surface: true }` reports:

| | 8 heights, m | relief |
|---|---|---|
| walk (0040), amp 80 | 1287.1 1288.0 1288.9 1286.2 1281.7 1281.7 1285.3 1286.2 | 7.2 m |
| **probe, amp 80, +x** | **1286.1 1287.0 1287.9 1285.2 1280.7 1280.7 1284.3 1285.2** | **7.2 m** |
| walk (0040), amp 160 | 2248.3 2248.3 2249.2 2247.4 2242.9 2242.0 2246.5 2247.4 | 7.2 m |
| **probe, amp 160, +x** | **2247.3 2247.3 2248.2 2246.4 2241.9 2241.0 2245.5 2246.4** | **7.2 m** |

Every sample is **exactly 1.0 m below** the walk's, in both worlds — the client's
pose height sits one metre above the surface voxel it reports. Shape, spacing and
relief are identical to the decimetre. The walk ran **east**. The 7.2 m /
1.75 km baseline is reproduced, in both worlds, and the anchor holds.

Transects at the other three sites, same stride, amp 80 (worst of four
directions):

| site | relief over 1.75 km |
|---|---|
| crest (highest land) | 8.1 – 12.6 m |
| walk-0040 | 7.2 – 9.0 m |
| median land cell | 8.1 – 90.9 m |
| steepest land cell | 43.2 – 138.6 m |

So the world is **not** uniformly flat. The two summit sites are; the median and
steep sites carry real grade. The walk landed on a plateau — and the plateau is
the highest ground in the world, which is why it read as damning.

---

## 2. The decay schedule, measured

At walk-0040 (lattice roughness 259.2 m — the 1-ring average of
`provenance_roughness` over the four cells sharing the corner). `analytic amp` is
`rough · AMP_DECAY^level` with the shipped `AMP_DECAY = 0.55`; `rms |delta|` is
the jitter actually injected, measured against each point's own parent average.

| level | spacing | analytic amp | rms &#124;Δ&#124; | max &#124;Δ&#124; | fate |
|---:|---:|---:|---:|---:|---|
| 1 | 7372.8 m | 142.57 | 66.88 | 160.14 | **discarded** |
| 2 | 3686.4 m | 78.41 | 39.48 | 77.11 | **discarded** |
| 3 | 1843.2 m | 43.13 | 22.74 | 42.57 | **discarded** |
| 4 | 921.6 m | 23.72 | 13.94 | 23.73 | **discarded** |
| **5** | **460.8 m** | 13.05 | *917.94* | *1022.66* | **`L_DEEP`: elevation REPLACED by the deep-time surface** |
| 6 | 230.4 m | 7.18 | 4.09 | 7.16 | survives |
| 7 | 115.2 m | 3.95 | 2.31 | 3.94 | survives |
| 8 | 57.6 m | 2.17 | 1.20 | 2.15 | survives |
| 9 | 28.8 m | 1.19 | 0.73 | 1.19 | survives |
| 10 | 14.4 m | 0.66 | 0.37 | 0.65 | survives |
| 11 | 7.2 m | 0.36 | 0.21 | 0.36 | survives |
| 12 | 3.6 m | 0.20 | 0.11 | 0.20 | survives |
| 13 | 1.8 m | 0.11 | 0.06 | 0.11 | survives |
| 14 | 0.9 m | 0.06 | 0.03 | 0.06 | survives |

`rms |Δ|` runs ~0.57 × the analytic amplitude at every surviving level, which is
exactly `1/√3` — the standard deviation of the uniform draw
`u ∈ [−1, 1]` the code makes. The schedule is doing precisely what it says.

**The mechanism is not the decay alone — it is the decay *composed with* the
`L_DEEP` override.** At level 5 the elevation is thrown away and replaced by
`DeepField::surface_at_voxel` (`rms |Δ| = 918 m`: that row is the deep field
overwriting the analytic value, not a jitter draw). Everything below level 5
descends from those replaced values, so **the jitter injected at levels 1–4 — 67,
39, 23 and 14 m RMS, at 7.4 km, 3.7 km, 1.8 km and 920 m wavelengths — is
computed and then discarded in its entirety.** The first surviving level is 6,
which by then is at `0.55⁶ = 2.8 %` of the roughness parameter.

Budget, at every one of the four sites, to the decimal:

| | scheduled (levels 1–14) | surviving (levels 6–14) | share |
|---|---:|---:|---:|
| any roughness value | `rough × 1.222` | `rough × 0.0611` | **5.0 %** |

Per provenance (independent of site):

| provenance | `rough` | scheduled | surviving, worst case | surviving, 1σ |
|---|---:|---:|---:|---:|
| Craton | 90 m | 110.0 m | 5.5 m | 1.72 m |
| Rift | 180 m | 219.9 m | 11.0 m | 3.44 m |
| Arc | 260 m | 317.7 m | 15.9 m | 4.98 m |
| **Orogeny** | **420 m** | **513.2 m** | **25.7 m** | **8.04 m** |
| OceanFloor | 25 m | 30.5 m | 1.5 m | 0.48 m |

**That is the order-of-magnitude gap corrections #23 asked for.** An Orogeny
column is scheduled 513 m of roughness and can physically deliver at most 25.7 m
of it to the walked surface — about 8 m at one sigma. The measured 7.2 m is not
mysterious; it is the design point of a schedule nobody re-derived after `L_DEEP`
was introduced.

The attenuation factor is exactly `AMP_DECAY^L_DEEP = 0.55⁵ = 1/19.8` — **1.3
orders of magnitude**, inside the hypothesis's stated 1–2.

---

## 3/4. Relief vs. window, and the attribution

Relief = max−min, metres, over nested windows centred on the site. `jitter` is
`lattice − no-jitter`; `final` includes river carving. amp 80 unless stated.

### walk-0040 (where journal/0040 stood)

| window | final | no-jitter (deep) | jitter (lattice) | deep-raw |
|---:|---:|---:|---:|---:|
| 100 m | 5.48 | **0.06** | **5.47** | 0.05 |
| 250 m | 15.64 | 0.16 | 15.50 | 0.12 |
| 500 m | 18.30 | 0.38 | 18.11 | 0.37 |
| 1 km | 19.74 | 1.00 | 19.84 | 0.99 |
| 2 km | 22.41 | 2.61 | 22.58 | 2.58 |
| 5 km | 29.30 | 11.09 | 22.58 | 11.12 |
| 10 km | 54.68 | 42.24 | 22.78 | 42.27 |
| 25 km | 295.78 | 285.58 | 22.78 | 285.62 |
| 50 km | 1148.25 | 1133.88 | 23.13 | 1133.93 |

At amp 160 the same site: 6.13 / 16.19 / 18.90 / 28.46 / 41.89 / 82.85 / 159.65 /
503.08 / 1591.89 — with the **jitter column byte-identical** (5.47 / 15.50 /
18.11 / 19.84 / 22.58 / 22.58 / 22.78 / 22.78 / 23.13). That is journal/0040's
"identical to the decimetre" explained: the lattice's jitter is seeded from
`provenance_roughness`, which `thickening_scale` never touches, and at this site
it is the *entire* signal below 2 km.

### crest (highest land cell), amp 80 / amp 160

| window | final 80 | deep 80 | jitter | final 160 | deep 160 |
|---:|---:|---:|---:|---:|---:|
| 100 m | 10.02 | 0.07 | 9.98 | 11.24 | 1.42 |
| 250 m | 12.89 | 0.14 | 12.76 | 15.15 | 3.46 |
| 500 m | 16.78 | 0.33 | 16.72 | 19.68 | 6.90 |
| 1 km | 18.01 | 0.68 | 17.99 | 25.06 | 13.89 |
| 2 km | 19.56 | 2.29 | 18.60 | 39.53 | 27.19 |
| 5 km | 28.40 | 13.82 | 20.40 | 85.07 | 72.38 |
| 10 km | 71.87 | 56.88 | 20.87 | 171.83 | 162.27 |
| 25 km | 407.01 | 395.00 | 20.87 | 648.95 | 640.77 |
| 50 km | 1343.96 | 1336.53 | 21.90 | 1834.44 | 1830.84 |

### steepest land cell, amp 80

| window | final | deep | jitter |
|---:|---:|---:|---:|
| 100 m | 10.38 | 7.59 | 3.58 |
| 250 m | 22.66 | 19.32 | 6.40 |
| 500 m | 44.48 | 38.79 | 9.93 |
| 1 km | 80.27 | 78.24 | 11.96 |
| 2 km | 154.19 | 156.04 | 12.68 |
| 5 km | 390.52 | 394.11 | 13.14 |
| 10 km | 791.25 | 792.84 | 13.48 |
| 25 km | 1918.47 | 1918.34 | 17.44 |
| 50 km | 3304.22 | 3298.82 | 19.93 |

### median land cell, amp 80

| window | final | deep | jitter |
|---:|---:|---:|---:|
| 100 m | 7.96 | 5.08 | 3.75 |
| 250 m | 16.83 | 12.73 | 6.83 |
| 500 m | 31.21 | 25.60 | 8.68 |
| 1 km | 55.80 | 51.47 | 11.66 |
| 2 km | 106.08 | 101.67 | 12.13 |
| 5 km | 258.71 | 255.09 | 13.23 |
| 10 km | 515.13 | 512.00 | 13.82 |
| 25 km | 1264.24 | 1258.41 | 16.86 |
| 50 km | 3487.73 | 3482.05 | 18.95 |

### What the curve says

1. **The jitter column saturates at ~13–23 m and never grows again.** Beyond a
   500 m window every additional metre of relief comes from the deep field. That
   ceiling is the 5 % budget of § 2, and it is the same number at both
   amplitudes at every site.
2. **`final` and `lattice` are equal to 0.01 m at every window and every site.**
   River carving contributes *nothing* measurable at these four sites — worth
   knowing, because it means the attribution is clean and rivers are not a hidden
   term in the 7.2 m.
3. **`no-jitter` tracks `deep-raw` to ~0.1 %** everywhere. See § 5.
4. **The world has two regimes.** At the steep and median sites the deep field
   supplies 78 m and 51 m per kilometre and the lattice is a 12 m garnish. At the
   two summit sites the deep field supplies **0.7–1.0 m per kilometre** and the
   lattice is 100 % of what a walker feels. Whether a place is dismal depends
   entirely on which regime it is in — and the highest ground in the world is in
   the flat one.
5. **The missing band is 250 m – 5 km.** Below 250 m the lattice is at least
   present (5–10 m). Above 5 km the deep field takes over and the numbers are
   respectable. In between, on a summit, there is 1–11 m of relief where a real
   range carries 100–400 m.

For scale: a real mountain front at a 1 km window carries 200–500 m of relief.
The steepest site here reaches 80 m; the crest reaches 18 m.

---

## 5. The 460 m bilinear resample destroys nothing — hypothesis FALSIFIED

The collapse layer samples the 460 m deep grid at the level-5 lattice's 460.8 m
spacing. That is a near-Nyquist resample of a bilinear field and it *looks* like
it should be a low-pass filter. It is not, measurably:

| site | raw deep grid, 20.7 km box | level-5 lattice, same box |
|---|---|---|
| crest | relief 258.79 m, mean step 4.301 m, max 16.99 m | relief **258.38** m, mean **4.312** m, max **16.99** m |
| steepest | relief 1579.74 m, mean 18.051 m, max 24.41 m | relief **1581.34** m, mean **18.073** m, max **24.35** m |
| median | relief 1031.66 m, mean 11.706 m, max 24.50 m | relief **1032.80** m, mean **11.717** m, max **24.49** m |
| walk-0040 | relief 185.07 m, mean 3.852 m, max 16.13 m | relief **184.43** m, mean **3.859** m, max **16.12** m |

Agreement is within **0.4 % on relief and 0.3 % on mean step** at every site. The
`no-jitter` and `deep-raw` columns of § 3 agree just as closely at every window.
Bilinear interpolation is exact at cell centres and the resample spacing differs
from the cell spacing by 0.17 %, so the level-5 lattice sees essentially every
cell value the grid holds.

**There is no deep-time detail being smoothed away below 460 m, because there is
none in the source to smooth.** The corrections #23 bilinear hypothesis is
falsified.

## 5b. What the deep grid actually holds at a summit

The same neighbourhoods, raw:

| radius (cells) | span | crest relief | crest mean &#124;ΔS&#124; | walk relief | walk mean &#124;ΔS&#124; | steepest relief | steepest mean &#124;ΔS&#124; |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 1.38 km | **0.60 m** | 0.289 | **1.23 m** | 0.304 | 73.00 m | 18.211 |
| 2 | 2.30 km | 1.95 m | 0.493 | 2.76 m | 0.459 | 146.57 m | 18.254 |
| 5 | 5.06 km | 11.90 m | 1.072 | 10.70 m | 0.918 | 365.73 m | 18.241 |
| 11 | 10.58 km | 58.88 m | 2.097 | 45.87 m | 1.848 | 804.90 m | 18.257 |
| 22 | 20.70 km | 258.79 m | 4.301 | 185.07 m | 3.852 | 1579.74 m | 18.051 |

At the summit, adjacent 460 m deep cells differ by **0.29–0.30 m** — a
**0.065 % grade**, sustained over a 10 km box that varies by under 60 m. The
erosion sim has produced a genuine high plateau, and it is the highest land in
the world, so it is exactly where a player who goes looking for mountains ends
up. At the steepest cell the same sim sustains 18.2 m per 460 m (**4 %**) over a
20 km box — an order of magnitude better, and evidence the sim *can* make grade.

---

## Verdict

| claim | status |
|---|---|
| "the lattice's amplitude decay attenuates roughness by 1–2 orders" | **CONFIRMED** — exactly `0.55⁵` = 19.8×, 5.0 % of budget survives |
| "and that is why the mountains are dismal" | **PARTLY** — true below 500 m; false above it |
| "the 460 m bilinear sample low-passes deep-time detail away" | **FALSIFIED** — reproduces the raw grid to 0.4 % |
| "thickening_scale never reaches sub-460 m relief" | **CONFIRMED** — jitter column byte-identical at 80 and 160 |
| unnamed third mechanism | the deep-time surface is itself a 0.07 %-grade plateau at the summit |

Confidence: **high** on the decay factor and the bilinear falsification (both are
arithmetic over a reproduced baseline). **Medium** on "fixing the decay fixes
dismal mountains" — it fixes the 100 m–500 m band, which is what a walker's near
field is made of; it cannot make a plateau into a range.

---

## Candidate calibrations — NOT flipped on

The user picks from pictures, not from rate constants. Each candidate is stated
with predicted relief, and predictions are **extrapolations, not measurements**:
jitter relief at window `W` is scaled by the RMS ratio of the amplitude bands
that can contribute inside `W` (levels whose spacing ≤ `W`). Re-running
`roughness_probe` after a flip replaces every predicted number with a measured
one, which is the point of shipping the probe.

The binding constraint on all of them is `tests/s7_walk.rs`'s
**`SEAM_TOLERANCE_VOXELS = 6`**: no two adjacent voxel columns may differ by more
than 6 voxels (5.4 m), asserted over 10 000 chunks. Adjacent-column steepness is
dominated by the *finest* levels (each level's slope contribution is
`amp_L / (spacing_L/2)`, and that ratio *grows* by 1.1× per level), so any
uniform boost lands squarely on that test.

**Measured headroom** (`cargo test --release -p dc-worldgen --test s7_walk --
--nocapture`, this run): `walk: max interior step 2, max border step 2 voxels`.
So the budget is **3.0×** on the finest levels before the assert fires, and
that number is the whole reason candidates A and B differ.

### A. Uniform re-anchor — `amp(L) = rough · AMP_DECAY^(L − 2)`

Move the schedule's anchor two levels finer, so level 6 starts where level 4 used
to. One constant, ×3.31 on every surviving level.

| window | walk-0040 today | predicted | crest today | predicted |
|---:|---:|---:|---:|---:|
| 100 m | 5.5 | **18** | 10.0 | **33** |
| 250 m | 15.6 | **51** | 12.9 | **42** |
| 500 m | 18.3 | **60** | 16.8 | **55** |
| 1 km | 19.7 | **66** | 18.0 | **60** |
| 5 km | 29.3 | **86** | 28.4 | **81** |
| 10 km | 54.7 | **118** | 71.9 | **126** |

- **Cost:** zero. Same draws, same memo, same lookahead bounds; no gen-time or
  memory change at all.
- **Risk — the serious one, and it is measured: this candidate probably fails
  the seam test.** Adjacent-voxel steps scale by 3.31 too, and the measured
  headroom is 3.0× (max interior step 2 of 6 voxels). 2 × 3.31 = **6.6 > 6**.
  A ×2.5 variant (`AMP_DECAY^(L − 1.68)`) would fit with nothing to spare;
  raising `SEAM_TOLERANCE_VOXELS` instead would be re-tuning the invariant to
  fit the change, which is the wrong order.
- **Risk:** ocean floor and craton get the same 3.31× (craton surviving 1σ
  1.72 → 5.7 m), so plains get visibly bumpy at the same time as mountains get
  legible. That may be desirable; it is not selective.
- **Risk:** every existing world changes. Not a compatibility concern today
  (nothing is persisted), but it is a *reproducibility* break of the same class
  as the erodibility flip (journal/0030).

### B. Band-limited re-anchor — boost only the levels the player walks

Boost levels 6–10 and taper back to 1 by level 11:
`amp(L) = rough · AMP_DECAY^L · boost(L)`, `boost = [4.0, 3.4, 2.8, 2.2, 1.6]`
for levels 6…10, `1.0` for 11–14. RMS band ratios: ×3.77 at ≥250 m windows,
×2.58 at the 100 m window.

| window | walk-0040 today | predicted | crest today | predicted |
|---:|---:|---:|---:|---:|
| 100 m | 5.5 | **14** | 10.0 | **26** |
| 250 m | 15.6 | **58** | 12.9 | **48** |
| 500 m | 18.3 | **68** | 16.8 | **63** |
| 1 km | 19.7 | **75** | 18.0 | **68** |
| 5 km | 29.3 | **96** | 28.4 | **90** |
| 10 km | 54.7 | **128** | 71.9 | **136** |

- **Cost:** zero, same as A.
- **Why it exists:** levels 11–14 (7.2 m down to 0.9 m spacing) are untouched, so
  the adjacent-voxel step barely moves. Estimated added slope across one voxel is
  ~0.3 voxels on top of the measured 2 — comfortably inside the 6 allowed.
  **Seam-safe by construction**, which A measurably is not.
- **What it buys:** relief concentrated in the 30 m–230 m band, i.e. hillslopes
  and ridges at the scale the camera actually frames, without per-voxel noise.
- **Risk:** 58 m of relief across a 250 m window is a ~23 % grade — a real
  mountain front, and near the edge of comfortable walking. If the character
  controller's step/slope limits are tighter than that, this is the candidate
  that finds out.
- **Risk:** shaped, not principled. It is five hand-chosen numbers, which is
  exactly the "bandaid tuning" the standing orders warn about — it rides
  as-built or not at all, and it is a *fractal* answer to a *simulation*
  question.

### C. Refine the deep tier instead — 230 m cells

The measurement says the fractal layer is not the only thing missing: the deep
surface is a plateau at the summit. Halving `DEEP_CELL_M` to 230 m puts the
erosion sim's own valleys into the 250 m–2 km band the curve says is empty.

- **Cost, measured-adjacent:** Medium is 545² = 297 k cells at 15.5 s and ~52 MiB
  today. Halving the cell is 1090² = 1.19 M cells → ~**62 s** and ~**210 MiB**,
  and `DEEP_MAX_WIDTH = 550` would have to rise. That **breaks the
  `pregen_time_vs_extent` <60 s budget**, which is a ratified number.
  "Worldgen time is not a constraint" is the standing user position, so this is a
  budget renegotiation rather than a blocker — but it is a renegotiation.
- **Predicted relief: none given.** This measurement *cannot* predict it. A finer
  grid does not create relief by itself; it lets the sim cut at a finer scale,
  and whether it does is an erosion-supply question (the S12 metre-scale
  exhumation finding, already Sequenced). Saying otherwise would be guessing, and
  the whole point of this document is that mechanisms are hypotheses until
  measured.
- **Why it is on the list anyway:** it is the only candidate that makes the
  summit plateau into a landform rather than draping noise over it. A and B make
  the *ground* legible; C makes the *mountain* legible. They are not exclusive.

### Rejected, with the number that killed it

**"Derive the jitter amplitude from the deep field's own local gradient"** — the
physically principled option: replace the constant `provenance_roughness` with
`c × max|ΔS|` over the deep 3×3 neighbourhood, so plains stay smooth and mountain
flanks get rough, self-scaling with the sim. Measured local mean `|ΔS|`:
**0.289 m at the crest and 0.304 m at walk-0040**, against 18.2 m at the steepest
cell. On this world that rule would drive summit roughness to **near zero** — it
would make the walked 7.2 m *worse*, not better, precisely where journal/0040
photographed the problem. Recorded so nobody re-derives it.

---

## Reproducing

```
cargo run --release -p dc-worldgen --example roughness_probe
```

~2 min: two 15.5 s Medium pregens with tectonic history, then ~300 k surface
evaluations. Check the `round-trip:` lines in § 5 first — if `surf[cell]`,
`surface_at_voxel`, `no-jitter` and `final` do not agree to ~1 m, the coordinate
bridge is broken and nothing below it means anything.
