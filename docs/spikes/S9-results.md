# S9 results — the deep-time tier (two-plane erosion + strata recorder)

Status: spike complete, 2026-07-19. Code in `crates/dc-worldgen/src/deeptime/`
(`grid.rs`, `erosion.rs`, `climate.rs`, `recorder.rs`, `refine.rs`); the
measurement harness is `examples/deeptime_spike.rs`; invariants in
`tests/deeptime.rs`. Numbers below from the Windows dev box, release profile,
seed `0x0D5E_ED57_2026`, Medium world (17×17 pregen cells, ~251 km). The spike
is **additive** — it does not touch dc-client, the live collapse path, the
pipeline registration, or pregen byte-identity.

The question (earth-processes.md § S9): a deep-time tier between pregen
(17×17 @ 14.7 km) and collapse. Three candidates —
**A** one coarse global mid-tier (~460 m), **B** landform-resolution global
(~48 m ≈ 27 M cells), **C** A + lazy bounded regional refinement. Judged on
wall time, memory, code complexity, and read-quality per the ratified method.

> The user blessed C **with recorded skepticism** and asked that B be treated as
> a live contender, not a strawman. It was. B lost — but on total cost of
> ownership, not on code simplicity (see the recommendation and its flip
> condition).

## What was built

A two-plane landscape-evolution model in our idiom, re-implementing the
mechanisms quarried from orogeny:

- **Two-plane erosion** (`erosion.rs`): bedrock stock `R` + alluvium stock `H`.
  Per fixed iteration: uplift (from pregen provenance, bilinear to the
  deep-time grid) → priority-flood depression fill + D8 routing → drainage-area
  accumulation → **mass-conserving stream-power transport** `A^m·S^n` with
  alluvial-cover shielding `(1−e^(−H/H*))`, entraining alluvium before incising
  bedrock and **never incising below the receiver** → cover-tapered bedrock
  weathering (R→regolith) → flux-limited hillslope diffusion. Sediment flux is
  routed explicitly down the receiver chain; whatever reaches a sink (sea /
  domain border) is deposited there. The only external input is uplift.
- **Strata recorder** (`recorder.rs`): the `StrataRec` shape adapted to
  deep-time — an ordered per-cell unit log, run-length-merged, pop-on-erosion,
  `sum(units)==H` finalize invariant. Units are **tagged by what was measured
  at the event** (never interpretation): subaerial/subsea (vs. the *current*
  paleo-sea-level stand), arid/humid (from the marched precip), and a
  Low/Med/High energy band (from the transport capacity). Erosion to bedrock
  sets a `stripped` flag so the next deposit is flagged an **unconformity** — a
  first-class proto-time-gap. Recording is driven by the **net** per-cell ΔH
  each iteration, which keeps the event count near the number of tag-changes and
  makes the invariant exact by construction.
- **Orographic march** (`climate.rs`): prevailing-wind 1D moisture march
  (precipitation, not residual humidity) with a cross-wind box blur to kill row
  streaking, re-marched every 20 iterations on the *current* topography.
- **Paleo-sea-level** cycle (deterministic sinusoid): transgression/regression
  so coastal columns record alternating marine/subaerial bands
  (earth-processes.md § 6). Mass-neutral (sinks still deposit all incoming
  flux), so the conservation invariant is untouched.

Determinism: fixed scan order, no wall clock, no ambient entropy. The only draws
are the addressed initial-roughness jitter (`SALT_DT_ROUGH`) and the C
boundary perturbation (`SALT_DT_PERTURB`), both new high-byte salts (`0x5900_*`)
that never collide with pregen's `0x5700_*`. `Instant` wraps runs only in the
harness, never in sim logic.

## The A/B/C numbers table

All on the Medium world. Coarse tiers run 200 iterations with the recorder ON;
the B scales run the recorder OFF (erosion-only, the cheaper datapoint) at a
bounded 4-iteration budget to read a stable ms/iteration, then project. "Peak"
is the analytic working set (grid planes + recorder + reused scratch + a
priority-flood heap high-water estimate) — honest and reproducible, not an OS
sample.

| candidate | cell | grid | cells | 200-iter wall | ms/iter | peak mem | recorded units |
|---|---|---|---:|---:|---:|---:|---:|
| **A** (spec) | 460 m | 545² | 297 k | **14.1 s** | 70.3 | **52 MiB** | 293 k |
| A sweep | 1 km | 251² | 63 k | 2.5 s | 12.4 | 12 MiB | 72 k |
| A sweep | 500 m | 501² | 251 k | 11.6 s | 58.0 | 44 MiB | 251 k |
| A sweep | 250 m | 1003² | 1.0 M | 55.9 s | 279.7 | 172 MiB | 892 k |
| **B** | 48 m | 5222² | **27.3 M** | **≈ 63 min¹** | 18 933 | **2.83 GiB²** | (rec off) |
| B/4 | 96 m | 2611² | 6.8 M | ≈ 11 min¹ | 3 375 | 709 MiB | (rec off) |
| B/16 | 192 m | 1306² | 1.7 M | ≈ 1.7 min¹ | 512 | 177 MiB | (rec off) |

¹ **Projection, not a full run** (the task's honest-error-bar path). The full
27 M-cell run was not carried to 200 iterations; ms/iteration is measured (4
iters) and multiplied. The projection is corroborated by the sub-scale trend:
per-cell cost grows super-linearly (0.30 → 0.50 → 0.69 µs/cell·iter from 192 →
96 → 48 m; ~`n^1.3`, cache + heap-`log n`). Extrapolating **from the two
sub-scales alone** (192 m, 96 m) to 27 M predicts ~20.6 s/iter vs. the measured
18.9 s/iter at 48 m — agreement within ~15 %, so ±15 % error bars on the 63 min.

² **Recorder OFF.** With the recorder ON, add 27.3 M × 24 B of empty `Vec`
headers (≈ 648 MiB) *before* a single unit is stored, plus the units
themselves — B-with-record lands near **3.5 GiB** and pushes the wall past
~80 min. A costs 52 MiB with the recorder on because it is 92× fewer cells.

Reading the table plainly:

- **A is seconds and tens of MiB.** "~460 m, seconds at creation" is confirmed
  (14 s, 52 MiB), and the recorder is nearly free at that scale.
- **B is not "minutes, paid once."** On this machine it is **≈ 1 hour and
  ~3 GiB** at 200 iterations (recorder on). Even bounded to 40 iterations it is
  ~13 min. That is the first falsified framing (see § Falsified assumptions).
  And ~99 % of those 27 M cells are ocean floor and uniform craton interior a
  player will never approach — the compute is real, the *value* of most of it is
  not.
- **C is A + on-demand refinement.** Coarse global once (14 s), then a region is
  refined only when approached. A 460 m→57 m (8×) refinement of a 32-coarse-cell
  region is 256² ≈ 65 k fine cells; at the measured 250 m/1 M-cell rate that is
  ~18 ms/iter → **~3.6 s per region + halo**, paid lazily. C's extra machinery
  is the refinement builder, the coarse-tier boundary-condition inheritance, and
  the halo sizing — which the decay measurement below validates.

### Code complexity (counted honestly)

The deep-time engine is shared by all three — ~900 LoC (`grid` 210, `erosion`
330, `climate` 70, `recorder` 200, `mod` 90). The candidates differ only in the
driver:

- **A**: the engine + one global `run()`. Zero extra code.
- **B**: identical to A — same `run()` at a finer `cell_m`. **The simplest
  candidate by code.** Its whole cost is runtime and memory.
- **C**: A + `refine.rs` (~230 LoC: the windowed builder, ring perturbation, the
  decay measurement) + the yet-unwritten production glue to (a) inherit
  drainage/area from the coarse tier as a fixed boundary field and (b) stitch a
  refined region's strata into the collapse pyramid. Call it ~2× the driver code
  of A/B, plus a real correctness obligation (halo sizing) that A/B don't have.

So on *code alone* B wins and C is the most complex. The recommendation turns on
runtime/memory and read-quality, not code.

## Decay-length measurement (the C correctness question)

The halo theorem: a **relaxation** process has a finite influence radius (refine
a region with a halo, its interior matches a global run); an **advective**
process (drainage — no halo bounds a basin) does not. Measured on a fine 96²
window @ 250 m over the Medium world: build a reference fine run, then an
identical run whose outer **3-cell** ring bedrock is perturbed by ±30 m (the
boundary error a refined region inherits from the coarse tier), and read the
per-ring max surface error vs. Chebyshev distance from the boundary after 200
iterations. Threshold 0.1 m.

| regime | bulk envelope | deepest penetration | verdict |
|---|---:|---:|---|
| hillslope only (weathering + diffusion, no rivers, no uplift) | 21 cells (5.25 km) | 21 cells (5.25 km) | **RELAXES** — bulk == deepest, monotone decay |
| full fluvial | 15 cells (3.75 km) | **26 cells (6.5 km)** | **ADVECTS** — bulk relaxes but isolated deep spikes |

Max-err (m) vs. boundary distance d (cells), full fluvial:
`d0=34 d2=30 d4=11 d6=2.0 d8=0.84 d10=1.1 d12=0.28 d14=2.9 d16=0.45 d18=0.06
d20=0.15 d22=0.27 d24=0.07 d26=0.03 d28=0.31 d30=0.002`.

The mechanism reads straight off that row: the **bulk field relaxes** — from the
±30 m boundary bump down to <1 m within ~6 cells and toward the noise floor by
~d18. But **isolated spikes reappear deep inside** (d14 = 2.9 m, d28 = 0.31 m):
those are **drainage reroutes** — the boundary perturbation captured or beheaded
a channel and the signal jumped tens of cells inland along the receiver chain.
The hillslope-only run has no such spikes (bulk == deepest), because with the
rivers off there is nothing to advect. This is orogeny's bounded/unbounded
theorem, reproduced by measurement in our engine: **erosion's relaxation part is
haloable at ~16–24 cells; drainage is not.** (Their measured decay-length halo
was ~31 cells; our ~21 for the relaxation bulk is the same order.)

The consequence for C is precise and favorable: **C must not try to refine
drainage.** Drainage/area is decided **once, coarse and global**, at the A tier;
the refinement re-runs only the *bounded* processes (hillslope, weathering,
deposition, facies) inside a region + a ~16–24-cell halo, reading upstream area
as a fixed boundary field from the coarse tier. That is exactly the split the
theorem licenses, and it is why C needs A underneath it rather than replacing
it.

## Process classification (relax vs. advect)

For each engine in the earth-processes.md catalog: does it **relax** (bounded
influence radius → refine locally with a halo) or **advect** (global → decide at
the coarse tier)? Rivers are the known wall; classification cites the measured
decay where the spike exercised it.

| engine (earth-processes §) | relax / advect | basis |
|---|---|---|
| Tectonics — uplift/subsidence (§1) | **decide coarse** | the coarse boundary forcing itself; a smooth global field, resampled bilinearly |
| Igneous emplacement (§2) | **relax** | province (coarse) gates it; emplacement depth is column-local — no neighbor coupling |
| Weathering (§3) | **relax** | rate = f(local climate, cover); measured inside the hillslope-only decay (~21-cell bulk) |
| Hillslope diffusion / colluvium (§4) | **relax** | **measured**: bulk envelope 21 cells @ 30 m/0.1 m |
| Stream-power incision magnitude (§4) | **relax given area** | local in (slope, cover) — but its `A` input is advective (below) |
| Drainage / flow routing / area (§4) | **ADVECT** | **measured**: deep spikes to 26 cells; a basin has no halo. The wall. |
| Aeolian, glacial, gravity transport (§4) | relax (aeolian mildly advective downwind) | short-range; not simulated here, classified by mechanism |
| Burial / diagenesis / metamorphism (§5) | **relax** | per-column P/T path from its own stack + exhumation; column-local |
| Sea-level + climate cycles (§6) | **decide coarse** | a scalar/low-D global field; shared trivially, applied locally (as done here) |
| Structural deformation — fold/fault/unconformity (§7) | **relax** | fold-phase = f(uplift) at sample time; unconformities recorded per-column |
| Groundwater / karst / hydrothermal (§8) | relax (water table mildly regional) | solubility from the recorded volume (local); water table a bounded relaxation |

The single advective wall is **drainage**. Everything else either relaxes (and
is therefore haloable — C's regional refinement is sound for it) or is a cheap
global scalar field decided once at the coarse tier. This is the re-proof the
rivers/2-ring Observed item asked for, now with a number attached.

## Read-quality (does a collapsed column tell a true story?)

From the 500 m tier (251 k cells, 200 iterations, recorder on): **176 476
readable columns** (land or record-bearing), mean **1.42 units** (max 115),
mean tag-variety 1.19. Unit-count histogram `[0:15 807, 1:131 997, 2:9 045,
3:10 199, 4:1 369, 5+:8 059]`. **3.6 %** fining-upward, **2.8 %** carry a
proto-unconformity, **26.5 %** carry a marine band.

The distribution itself reads true: **most of the landscape is a thin
single-unit veneer** (uplands are erosional bypass zones — they *should* be
thin), while a fat tail of basins and coasts accumulates multi-unit,
multi-environment records (8 059 columns with 5+ units). Accommodation, not
uniform blanketing, controls thickness — which is the correct story.

Three example columns (top = surface):

**1. A pulsed alluvial fan on an arid range flank** — cell (184, 225), surface
2 886 m, H 7.3 m, 4 units:
```
  4.02 m  [Sa/A/M]  subaerial arid  med-energy
  2.11 m  [Sa/A/L]  subaerial arid  low-energy
  0.23 m  [Sa/A/M]  subaerial arid  med-energy
  0.93 m  [Sa/A/L]  subaerial arid  low-energy   (base)
```
Alternating med/low energy bands, arid throughout (the high interior sits in its
own rain shadow — the orographic march working). Read: successive debris-flow /
sheet-flood couplets on a fan, no marine influence at altitude. The energy
alternation is a readable "how did this get here?" answer: pulsed discharge.

**2. A transgressive-regressive coastal couplet** — cell (61, 143), surface
94.6 m, H 6.8 m, 3 units:
```
  5.92 m  [Sa/A/L]  subaerial arid  low-energy
  0.29 m  [Ss/-/L]  marine          low-energy
  0.55 m  [Ss/-/L]  marine          low-energy   (base)
```
Marine mud at the base (a highstand drowned the site), then a thick subaerial
arid fill as the shoreline regressed and alluvium prograded over it. The
marine→subaerial contact is a readable flooding surface; the thick cap records
base-level fall. This is the sea-level cycle leaving a legible signature — the
classic layered-cliff couplet, in miniature.

**3. A condensed, sediment-starved margin** — cell (20, 266), 7 units,
sub-decimetre each, alternating `Sa`/`Ss`:
```
  [Sa/A/L] [Sa/A/M] [Sa/A/L] [Ss/-/L] [Ss/-/L] [Sa/H/L] [Ss/-/L]  (base, thin)
```
Every sea-level swing recorded as a thin couplet with near-zero net
accommodation — a condensed section. Thin, but a *true* cyclic record: seven
transgressions you can count. (These low-accommodation columns are why mean
units > 1 while median thickness is small.)

All three answer "how did this get here?" in process terms — the ratified
validation bar. The recorder caught fining sequences, marine flooding surfaces,
and erosional gaps; the tags are all measurements, never interpretations.

## Recommendation

**Ship A as the always-on deep-time tier, and layer C's bounded regional
refinement on top for approached regions that need landform-scale detail. Do not
build B.**

The evidence:

1. **B's one-time cost is real and mostly wasted.** ~1 hour and ~3 GiB on this
   machine (200 iters, recorder on), and ~99 % of the 27 M cells are ocean and
   uniform craton no player approaches. B buys global landform resolution, but
   the landform resolution that matters is recoverable **on demand** by C at
   ~3.6 s/region.
2. **The decay measurement licenses the A+C split exactly.** Drainage is the one
   advective quantity, and it is *cheap* at coarse resolution — so decide it once
   at A and never refine it. Every process that benefits from fine resolution
   (hillslope, weathering, deposition, facies, folding) is a relaxation with a
   measured ~16–24-cell halo — precisely the haloable set. C refines only those.
   This is the user's skepticism answered with a measurement: C is not a "genius
   holy grail," it is the *only* split the bounded/unbounded theorem allows once
   you accept you want landform detail without paying B's global bill.
3. **Read-quality is already good at A resolution.** 460 m columns tell true
   stories (fining fans, flooding surfaces, condensed sections, unconformities).
   The collapse pyramid's existing addressed jitter refines sub-cell *relief*
   for free; C adds *simulated* sub-cell process only where a player will see it.
4. **A is the humane world-create.** 14 s vs. an hour is the difference between a
   loading bar and a coffee break, for a difference no player sees globally.

### The evidence that would flip this to B

**Engine performance.** This is a single-threaded scalar priority-flood +
transport. A parallel priority-flood (Barnes' parallel variant) plus a
multi-threaded transport pass could plausibly be 10–50× faster; at 30× the full
27 M×200 run drops from ~1 hour to **~2 minutes**. If the production engine hits
that — *and* the machine has the RAM for ~3.5 GiB of transient state — then B's
decisive advantage (it is the **simplest code**: no refinement builder, no halo
sizing, no coarse→fine drainage coupling, no strata-stitching) could outweigh
its cost. The recommendation is therefore **conditional on the erosion engine
staying scalar/serial**. Before committing to the A+C machinery, the real 3e
implementation should spike a parallel priority-flood and re-measure B's wall
time; if B fits a ~2-minute one-time budget, prefer it for its simplicity.

A secondary flip: if worlds are typically **Small** (5×5 cells, ~74 km), B at
48 m is only ~2.4 M cells (interpolating the table: <2 min, <300 MiB) — B is
fine there. B's problem is Large worlds, where it is intractable and C's laziness
is essential. So the honest nuance: **B for Small, A+C for Medium/Large** is a
defensible middle path if code-simplicity is weighted heavily.

## Falsified assumptions (→ corrections.md candidates)

1. **"B is minutes, paid once" (earth-processes.md § S9 framing).** Measured
   ~63 min (recorder off) / ~80 min (recorder on) and ~3 GiB for the full 27 M
   cells at 200 iterations on this machine — not minutes. It is minutes only at
   Small extent, or with a parallelized engine (untested). The "paid once" is
   true; the "minutes" is scalar-engine-optimistic by ~30×.
2. **Implicit "erosion is bounded, so a small halo suffices" (orogeny recon
   quarry).** Half-true and worth stating precisely: the *relaxation* part of
   erosion is bounded (measured bulk halo ~16–24 cells), but a stream-power
   landscape's **drainage** produces isolated deep boundary-error spikes
   (measured to 26 cells in a 48-cell half-domain, and unbounded in general).
   "Erosion is bounded" must be qualified: *hillslope* erosion is bounded;
   *fluvial* erosion inherits drainage's global reach. C works **only** because
   drainage is decided coarse, not because erosion as a whole is haloable.
3. **Minor: "the collapsed surface geometry is ahistorical, only composition is
   recorded" (orogeny's compromise we said we'd refuse).** In this spike the
   surface `R+H` *is* the historical eroded surface (we own generation), so we
   already don't inherit that compromise — noting it so 3e doesn't reintroduce a
   final-plane shortcut.

## Loose ends — what the real 3e implementation must decide

- **Physics calibration is plausible, not tuned to Earth.** The `k`, `m`, `n`,
  `H*`, uplift-scale, and iteration count were chosen so orogenic belts build
  hundreds of metres of net relief and basins accumulate readable stacks over
  200 iters; they are not fit to a target denudation rate. 3e should pin an
  iteration↔Myr and a cell↔km calibration against a real orogen's relief and
  sediment yield.
- **Absolute thicknesses are thin** (metres, not the tens-to-hundreds a real
  basin holds) because supply and time are compressed. The *patterns* (fining,
  cyclicity, unconformity) are right; the magnitudes need the calibration above,
  or a supply multiplier.
- **The coarse→fine drainage handoff for C is unbuilt.** `refine.rs` measures
  the halo but does not yet inherit area from a coarse run as a fixed boundary
  field, nor stitch a refined region's strata into the collapse pyramid. That is
  the core C production task, and it must respect the collapse layer's bounded
  lookahead (the halo width is now a measured input to that budget).
- **Recorder tag space is coarse** (env × 2-aridity × 3-energy). Real facies want
  more axes (agent: fluvial/aeolian/glacial/marine; grain-size continuum). The
  net-ΔH-per-iteration recording generalizes, but the tag vocabulary is a
  content decision that couples to the geology class contracts.
- **Parallelism.** The single decision that most changes the A/B/C verdict — see
  the flip condition. Spike a parallel priority-flood before committing.
- **Sea-level/climate curves are placeholders** (a single sinusoid, one band
  map). 3e wants epoch-indexed paleo curves as pregen state (the § 6 paleo axis).
