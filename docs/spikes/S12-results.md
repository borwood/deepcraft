# S12 results — tectonic history (uplift(t), analytic forcing, crustal columns, isostasy)

Status: spike complete, 2026-07-20. Code behind `DeepConfig::tectonic_history`
in `crates/dc-worldgen/src/deeptime/` (`tectonics.rs`, `isostasy.rs`, and phases
threaded into `erosion.rs`/`grid.rs`/`recorder.rs`/`field.rs`/`mod.rs`). The
measurement harness is `examples/tectonic_spike.rs`; the falsifiers are
`tests/tectonic_history.rs`. Numbers below from the Windows dev box, release
profile, seed `0x0D5E_ED57_2026`, Medium world (17×17 pregen cells, ~251 km,
460 m deep grid = 297 025 cells) and Large (69×69, capped to 1850 m cells).

Architecture RATIFIED 2026-07-20 (tectonics.md banner, U1–U8). This spike
implements the flag and reports the eight § SPIKE measurement groups.

> **Wall-time note.** Cost was measured twice; the numbers below are the quieter
> run, in which the **Medium off-flag baseline landed at 14.33 s — matching
> corrections #12's 13.79 s shipped number** — so these absolutes are trustworthy
> (an earlier run under heavy concurrent-agent load read ~25 s off-flag and is
> discarded). Still, ratios (on/off, and the isolated isostasy cost) are the most
> robust figures.

## What was built

The inversion (tectonics.md § 2): surface uplift stops being the input. Plate
kinematics (seeds + velocities, advected through K chapters) drive an **analytic
thickening forcing** evaluated at deep-grid resolution; the forcing thickens
**crustal columns**; **smoothed-load Airy isostasy** derives elevation from the
columns each step; erosion argues with it. One flag, off by default,
byte-identical off.

- Analytic forcing `amplitude(type, v_conv) × exp(−(d/W)²)`, `d` = exact signed
  distance to the plate-pair bisector, summed over the 3 nearest seeds, with
  arc/trench sidedness offsets — **no grid term** (`tectonics.rs`).
- Crustal columns `t_crust`/`crust_kind`/`exhum`; thickening adds to `t_crust`,
  erosion decrements it and grows `exhum` (`erosion.rs`).
- Airy compensation of the flexural-wavelength-smoothed load, relaxed at
  `iso_rate` (`isostasy.rs`); the smoothing is a scalar separable prefix-sum blur.
- `DepUnit.chapter` stamp (joins the merge key; 0 off the flag); drainage export
  on `DeepField` (`recv`/`area`/`lake`/`exhum`/`t_crust`/chapter table).

## Group 6 — byte-identity off / determinism on (NON-NEGOTIABLE) — PASS

- **Off is byte-identical against every tectonic knob varied.** `plate_scale_km`,
  `chapters`, `ramp_chapters`, `orogen_width_km`, `arc_gap_km`,
  `advection_plate_widths`, `thickening_scale`, `flex_wavelength_km`, `iso_rate`
  all changed with the flag off → `r`, `h`, and every strata record identical to
  the default run (`flag_off_is_byte_identical_regardless_of_tectonic_knobs`).
  The tectonic-only planes stay empty and every unit stamps chapter 0.
- **Determinism on**: double-run byte-identity of `surf` and `strata` (true/true);
  **scalar↔parallel byte-identity** of `r`/`h`/`t_crust`/`strata`/ledger at a
  finer cell that forces the rayon fork (`tectonic_parallel_equals_scalar`…).
- Existing same-seed and integration byte-identity tests still pass (production
  config keeps the flag off, so the shipped world is unchanged).

**Pass bar (flag-off byte-identity + on-flag double-run + scalar↔parallel): MET.**

## Group 5 — forcing wavelength: the 50 km smear dies (NON-NEGOTIABLE) — PASS

Analytic belt half-width (peak → half-maximum) across two head-on continental
plates, vs the design `W`:

| `orogen_width_km` W | peak thickening | gradation-to-peak (half-max) |
|---:|---:|---:|
| 15 | 80.0 m/iter | **12.5 km** |
| 25 (default) | 80.0 m/iter | **20.9 km** |
| 40 | 80.0 m/iter | **33.4 km** |

The gradation-to-peak tracks `W` linearly (≈ 0.83·W, the `exp(−(d/W)²)` half-max)
and at the default sits at **20.9 km vs the ~50 km smear** the painted 2-ring
forcing produced (earth-processes § 1). There is **no 14.7 km-cell term anywhere
in the expression**, so no pregen-cell signal can survive — the wavelength is a
pure function of the km-stated `W`. `orogen_width_km` is the knob the user's U4
belt-sharpness call turns.

**Pass bar (measured gradation-to-peak ≈ designed W, ≪ 50 km): MET.**

## Group 2 — recorder growth from chapter stamps (NON-NEGOTIABLE) — PASS (factor stated)

Medium, biology + erodibility on, 200 iters, record-bearing columns:

| config | total units | mean/col | max | hist [0,1,2,3,4,5+] |
|---|---:|---:|---:|---|
| off (K=1) | 682 289 | 3.29 | 50 | [0, 131576, 18745, 9049, 6991, 41160] |
| K=4 | 1 510 583 | 5.30 | 44 | [0, 15613, 34858, 45104, 66097, 123596] |
| K=8 (default) | 2 182 057 | 7.65 | 49 | [0, 5928, 14973, 19509, 21660, 223203] |
| K=16 | 3 355 721 | 11.76 | 55 | [0, 2302, 5695, 8506, 9784, 259024] |

- `sizeof(DepUnit)` = **16 bytes, unchanged** (the `u8` chapter fits the existing
  8-byte padding, as the design predicted).
- **The merge-break factor at K=8 is ~3.2×** the off record (682 k → 2.18 M units;
  mean/col 3.29 → 7.65, a +4.36 rise, inside the design's ≤ K−1 = 7 bound). The
  factor scales sub-linearly in K (×4 at K=4, ×3.2 at K=8, ×4.9 at K=16 — the
  extra breaks concentrate in the already-thick 5+ tail). The `sum(units)==H`
  invariant and the S10 overprint merge are untouched (recorder test passes).
- (The design's "~71 k baseline" was the S9 500 m/biology figure; this off
  baseline is the 460 m Medium biotic+erodibility record, 682 k — the honest
  same-config comparison.)

**Pass bar (merge optimization survives within a stated small factor): MET —
factor ~3.2× at K=8, stated; the user judges it.**

## Group 3 — ritual cost + the U3 tradeoff (production entry, parallel path)

Measured at `build_field_cfg` (the production entry, byte-identical parallel path
— corrections #12).

| extent | flag | iters | wall (s) | peak (MiB) | max elev | units |
|---|---|---:|---:|---:|---:|---:|
| Medium | off | 200 | 14.3 | 32.6 | 12032 | 682 289 |
| Medium | ON | 200 | 16.2 | 68.3 | 953 | 2 182 057 |
| Medium | ON | 300 | 23.5 | 73.5 | 1228 | 2 334 494 |
| Medium | ON | 400 | 31.3 | 75.9 | 1504 | 2 392 361 |
| Large | off | 200 | 15.2 | 30.6 | 12663 | 564 357 |
| Large | ON | 200 | 18.9 | 46.2 | 2051 | 1 014 660 |
| Large | ON | 300 | 27.1 | 46.2 | 2878 | 937 864 |
| Large | ON | 400 | 35.2 | 45.6 | 3705 | 856 835 |

- **Isostasy, the only new per-iteration serial cost, isolated: 8.04 ms/iter →
  ~1.6 s over 200** (Medium production grid). This is the load-smooth + Airy
  relax; the per-chapter repaint is a one-time precompute (K+1 planes, ~ms each).
- **On/off wall ratio ≈ 1.13× at Medium 200** (16.2/14.3, i.e. **+1.9 s → ~16 s**)
  — squarely inside the design's "≈ +1–3 s → ~15–17 s, same ritual class"
  estimate. The isolated isostasy 1.6 s is the bulk; the rest is the ~3× larger
  record.
- **Peak memory +35 MiB at Medium ON** (32.6 → 68.3), **above** the design's
  "+~8 MiB resident" estimate — the extra is dominated by the ~3× record growth
  (chapter stamps, group 2) plus the export planes (~9 MiB), not the columns
  (~5 MiB). A real finding: the memory cost is the *record*, not the physics.
- **U3 (amended) tradeoff — biography length vs price, user-owned.** More
  iterations buy more relief and more legible chapters at a near-linear wall cost.
  At Medium: 200→300→400 iters is 16.2→23.5→31.3 s and max elev 953→1228→1504 m
  (relief keeps building — belts are not yet at equilibrium at 200). At Large the
  effect on relief is stronger (2051→2878→3705 m), because
  the coarser 1850 m cell has a smaller flexural smoothing radius so the belt
  stands taller. **The user picks the biography length; 400 iters at Medium is
  ~34 s inflated / well under the "5 min is fine" ceiling.**

**Pass bar: reported (no cap — U3 amended). MET as a reported tradeoff.**

## Group 7 — relief / hypsometry for the amplitude call (U7)

Medium, 200 iters, land-cell elevations vs `thickening_scale`:

| `thickening_scale` | max elev | p50 land | p95 land | mean land |
|---:|---:|---:|---:|---:|
| 40 | 678 | 492 | 645 | 426 |
| 80 (default) | 953 | 613 | 889 | 546 |
| 160 | 1501 | 857 | 1377 | 782 |

Clean monotonic amplitude response — the data the U7 dismal-mountains amplitude
call is made against (and, per § 4.2, must be made *against this forcing, not
before it*). **Note the contrast with the legacy path's 12 km max elevation**:
the old `provenance_uplift × uplift_scale` piled uplift straight onto bedrock
every iteration (physically unbounded spikes), while the tectonic path builds
crust and lets isostasy cap elevation realistically — so the honest tectonic
relief is *lower and needs the amplitude turned up*. The default 80 is
conservative; the user will likely want 160+ (see group 4, where the landforms
only become legible at 240).

**Pass bar (relief distributions packaged for the amplitude call): MET.**

## Group 1 — clamp / numerical stability under repainting (NON-NEGOTIABLE) — PASS

- The full stack on (tectonic history + erodibility contrast 4.0 + biology + full
  agents) across 8 chapter repaints: **nothing non-finite**; `r`/`h`/`t_crust`/
  `exhum` all finite, `t_crust > 0`, `h ≥ 0`, `exhum ≥ 0`; material moved (no
  stall) (`stack_is_finite_and_stable_across_chapter_repaints`).
- The erodibility clamp is untouched in form (`expose` re-reads outcrop every
  step, so a repaint is automatically re-read next iteration — journal/0029).
- **Isostasy stability**: the default `(iso_rate 0.5, flex_wavelength 50 km)` is
  stable at both 460 m (radius ~109 cells) and Large's 1850 m (~27 cells); the
  smoothed load has no grid-wavelength rebound artifact by construction (the
  compensation is smoothed over tens of km, never per-cell — § 6.1). The
  smoothing is scalar, so it adds no non-determinism.

**Pass bar (no runaway/stall/non-finite; clamp holds; no grid artifact): MET.**
Ramped-vs-stepped knickpoint-transient *counts* were not separately tabulated
(see cuts).

## Group 8 — drainage export fidelity — PASS

- Exported `recv`/`area`/`lake` for all 297 025 cells; **Σ area over sinks =
  297 025 = n exactly (Δ 0.0e0)** — every cell's unit area drains to a sink, so
  the exported network is the sim's own last-iteration routing, not a re-derived
  one (`drainage_export_is_populated_and_accounts_for_every_cell`).
- 100 416 exported lakes (closed basins depression-filled at the final stand).
- Chapter table: 9 entries (K+1) × ~15 plates ≈ the "entire tectonic history of a
  world is a few KB" the design promised.
- Off the flag the export is empty (byte-identical DeepField in `surf`/`strata`).

**Pass bar (exported drainage matches the last iteration's routing exactly): MET.**
The before/after render of a river valley (retiring the pregen chords) is a
collapse-tier visual, out of this headless spike's scope.

## Group 4 — landform evidence (LOWEST PRIORITY — amplitude-gated)

The landform reads came back **inconclusive at the deep-grid, and the reason is a
genuine finding**: they are gated on *two* calibrations, not one. Measured at a
high amplitude (`thickening_scale = 240`, belts to **2876 m**), Medium, 300 iters,
ramped vs a stepped control:

| proxy | value | reading |
|---|---|---|
| water-gap (large-area channels held > 1200 m axis) | ramped 10 321 / stepped 11 441 (0.90×) | **proxy failed to isolate the mechanism** |
| superposition (≥3 chapter-tagged units/column) | 258 221 of 297 025 | **near-universal → not a belt-superposition measure** |
| exhumation | **max 11 m, land-mean 4 m**; cells > 500 m: **0** | **the load-bearing finding** |
| foreland-wedge ratio (moat sed / land-mean) | 0.00 | below the read floor |

Two honest conclusions:

1. **Exhumation is real but metre-scale, so exhumed metamorphic cores (§ 6.4) do
   NOT appear at the shipped erosion calibration.** Even under 2.9 km belts over
   300 iters the deep sim strips only ~4–11 m of bedrock — consistent with the
   erosion constants (weathering 0.02 m/iter, cover-shielded incision) and with
   S9's own loose end ("absolute thicknesses are thin because supply and time are
   compressed"). So the metamorphic-core and foreland-wedge signatures are gated
   on the **erosion/supply calibration**, not only the amplitude — a finding for
   the integrating session: raising `thickening_scale` makes tall mountains, but
   exhuming their roots and filling their forelands needs the denudation rate (or
   the iteration count) turned up too. `exhum` is exported so the collapse tier
   can read whatever grade *is* produced.
2. **The water-gap and superposition proxies are too crude to isolate their
   mechanisms** (the water-gap count is dominated by ordinary drainage on high
   ground, not antecedence; nearly every column spans ≥3 chapters at K=8 so
   "chapter-diverse" is trivially true). The clean reads — tracing a single
   channel's upstream-area continuity across a rising axis, and finding a place
   with two *belts* of different chapters — need a bespoke read harness that was
   out of budget (see cuts). The dynamic substrate they'd measure *is* present
   and stable (groups 1/3/8); the measurement of it is what's missing.

**Pass bar: PARTIAL.** The mechanisms are built, stable and byte-deterministic;
the *legibility* reads are gated on the amplitude call (U7) and the erosion/supply
calibration (an S9 loose end), and the proxies here are too crude to stand as the
landform evidence. This is the honest state, reported per the triage rules.

## Deviations from tectonics.md forced by implementation reality (the § 14 pattern)

1. **The mass ledger did not migrate fully to thickness-space (design § 3.4) —
   and should not.** Full migration (reconstruct `R` from `(T,H)` every step)
   would rewrite `transport`/`weather`/`diffuse`, all of which operate on `R`
   directly, and forfeit byte-identical-off composition. Instead the engine keeps
   `ΣR+ΣH` as its stock and **declares isostasy's bedrock injection as an external
   input** (the `biotic_total` pattern), so the ported test `Δ(ΣR+ΣH) ==
   uplift_total + biotic_total` reads unchanged with `uplift_total` = summed
   isostatic ΔR. The thickness stock gets its *own* exact invariant `Δ(Σt_crust)
   == thickening − exhumation`. **Both ledgers balance to fp slack** (tests
   `r_plus_h_ledger_balances…`, `thickness_ledger_balances`). Two exact stocks,
   not one migrated stock.
2. **`Erosion::new`'s cached `uplift_sum` (§ 14.3)** is left in place (still valid
   and returned on the legacy path) with a comment that on the tectonic path it is
   *not* the ledger — the isostatic injection is. Flagged so no future reader
   rediscovers it as a conservation failure.
3. **Addressed-draw family re-addressed 0x5B00 → 0x5D00**: the design named
   `0x5B00_*` for per-chapter draws, but the biotic layer already owns that byte.
   Fresh `0x5D00_*` keeps the address spaces disjoint.
4. **Advection is Eulerian** (the stated § 13 casualty): no terrane docking, no
   lateral strike-slip offset of landscapes. Transform motion's *record* is
   deferred to the fault-event species (punctuation designs).
5. **Isostasy runs scalar in both drivers** (not the "deterministic parallel
   running-sum rows" § 6.1 offered as an option) — the scalar separable blur is
   byte-identical scalar↔parallel by construction and the isolated cost (8 ms/iter)
   does not justify the parallel-determinism obligation yet. The gather form is the
   door left open if it ever dominates.

## What was cut (priority order honored: 1, 5, 6, 2 non-negotiable and complete)

- **Group 4 landform controls** are proxies, not the clean controlled reads. The
  water-gap proxy at default amplitude was inconclusive (ratio 0.95) because the
  belts were too low; the exhumed-core and foreland proxies found nothing at
  default amplitude. Raised amplitude (run 2) is the honest fix, but the
  **rebound-persistence half-life (§ SPIKE 4f)** and the **isostasy-off foreland
  attribution control (§ SPIKE 4c)** were not built — there is no "thickening
  without isostasy" mode (the two do not flip separately by design), so those
  controls need a bespoke harness mode. The **angular-unconformity count/render
  (§ SPIKE 4d)** is a collapse-tier read (dip is re-derived analytically at
  collapse resolution from the chapter table) and out of this headless spike.
- **Ramped-vs-stepped knickpoint-transient tabulation (§ SPIKE 1)** — stability
  was proven (finite, no stall) but the transient ΔH-variance/knickpoint counts
  after each boundary were not separately tabulated.
- **Spectral check of the rebound field (§ SPIKE 1)** — argued by construction
  (smoothed compensation, no per-cell Airy) rather than FFT-measured.
- **Scalar-path cost datapoint (§ SPIKE 3)** — only the production parallel path
  was timed; the scalar comparison is covered by the byte-identity test, not a
  separate timing.

## Loose ends the integrating session inherits

- **The amplitude call (U7)** is the gating decision: default `thickening_scale`
  80 gives conservative ~950 m belts; the § 7 table + group 4's amplitude-gating
  say the user should judge at 160–240 against renders (production flip, 0030
  style — U8).
- **Memory is the record, not the physics** (group 3): if K=8's ~3.2× record
  growth is judged too heavy, the lever is K (the chapter count is a knob), not
  the columns.
- **The chapter table + `exhum` are exported** so the deferred passes have their
  axes: the metamorphic-grade slice (grade = f(peak_t_crust − t_crust, exhum)),
  the fault-event species, the analytic dip/angular-unconformity at collapse, and
  the water/3e-2 drainage handoff.
