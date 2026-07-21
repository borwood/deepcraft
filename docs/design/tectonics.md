# Tectonics — the moving world

> **RATIFIED 2026-07-20 (user)** — architecture and all eight user
> decisions accepted in session 4, same day as the draft. U1 (plate_scale_km
> = 65 knob), U2 (K = 8, knob), U4 (W ≈ 25 km, arc gap ≈ 50 km, judged from
> spike renders), U5 (~1 plate-width advection/run), U6 (≤1 impact, ~1–2
> other punctuation rolls/world), U7 (amplitude call re-sequenced to after
> analytic forcing), U8 (production flip after spike screenshots,
> 0030-style): accepted as recommended. **U3 amended by the user beyond the
> recommendation**: the ritual-length ceiling is not ~28 s — "when we're
> done with the game i can accept 5 min if that's what it takes." The spike
> should therefore report time/quality tradeoffs rather than optimizing to
> a cap; worldgen time is not a constraint (standing rule).
> Next step: the SPIKE, per § SPIKE below. Produced 2026-07-20 by the
> dispatched design agent (design pass → spike → milestone).

> **U8 DECIDED 2026-07-21 (user)** — production flip ON: `production_config`
> in `deeptime/field.rs` now sets `tectonic_history: true`. Ratified
> **without** gating on the appearance walk ("the flip is pomp"; "I want us
> to make progress. We can correct mistakes later"); the walk documents the
> flip rather than gating it (journal/0044). U7 amplitude rides at the
> default 80 per corrections #23 — the knob buys no sub-km relief (it lifts
> the continent, it does not make mountains), so its value is deferrable and
> not part of this flip. `full_agents` stays OFF (not ratified). Every world
> created after this flip is unreproducible under earlier builds — the same
> world-fingerprint cost as the 0030 erodibility flip.

**The one-sentence thesis.** Tectonics stops being a one-shot painting of
uplift onto coarse cells and becomes a *chaptered kinematic history* —
plates advect, boundaries re-classify, convergence thickens crustal
columns, isostasy turns thickness into elevation, and erosion argues with
it for five hundred million years — so that dip, unconformity, water gaps,
foreland basins and exhumed metamorphic cores are **recorded consequences,
not authored features**.

---

## 0. Priors in the corpus (swept before writing — do not re-derive)

- **The ratified scope** is earth-processes.md § 1's DECIDED entry
  (2026-07-20), items 1–7, plus the plate-scale knob flagged there as a
  user decision. This document is that design pass.
- **The two diagnosed diseases** (same entry): *one-shot upheaval* (the
  uplift plane never changes during the deep run) and the *50 km
  gradation-to-peak* (all tectonic forcing painted on 14.7 km cells with a
  2-ring falloff, then bilineared into the 460 m grid — nothing narrower
  than ~15 km wavelength can exist, and belt flanks smear to ~50 km).
- **S9 (measured)**: A tier 460 m / 297 k cells / 14.1 s / 52 MiB; C
  refinement ~3.6 s/region; hillslope decay 21 cells; **drainage is the
  sole advective process** — decided coarse and global, never refined
  (corrections #8). S9b: the flood has no byte-identical parallel form;
  the parallel path is the per-cell phases only (corrections #9).
- **The ritual budget as shipped**: `field.rs` caps the deep grid at
  `DEEP_MAX_WIDTH = 550` (~300 k cells at every extent; Large gets
  ~1.85 km cells), 200 iterations, ~13.8–14.4 s measured on the production
  parallel path with biology + erodibility on (corrections #12,
  journal/0029/0030).
- **Calibration (RATIFIED 2026-07-19)**: the Phanerozoic register — the
  recorded span is ~500 Myr, so at 200 iterations one iteration ≈ 2.5 Myr.
  Ages must be labeled before persistence/knowledge commits them as facts.
- **The stability machinery that must survive** (erosion.rs): mass ledger
  `Δ(ΣR+ΣH) == uplift_total + biotic_total`; *never incise below the
  receiver*; the erodibility feedback clamp `[1/erodibility_max,
  erodibility_max]` (journal/0029 — the erode-soft→expose-hard loop is
  self-reinforcing and the clamp is what bounds it); the S10 lagged
  biotic coupling; scalar↔parallel byte-identity per phase.
- **The recorder that must extend** (recorder.rs): per-cell
  deposition-ordered `DepUnit` stacks, run-length merged on tag equality,
  pop-on-erosion, `sum(units) == H` exact; the S10 pedogenic overprint is
  what holds the record at ~71 k units instead of 665 k — any extension
  that breaks that merge re-explodes the record.
- **The layer-cake Observed entry** (ROADMAP, user 2026-07-20): the only
  structural feature in the entire world is `DepUnit::unconformity: bool`.
  Every bed is horizontal regardless of history. "The seams have to match
  orientation properly if this was pushed up in a tectonic event."
- **Dismal mountains** (ROADMAP, diagnosed): cause 1 (lithology-blind
  erosion) closed and flipped; causes 2 (no dip/fold — this document) and
  3 (conservative amplitude — a user call that should now WAIT for this
  design, see § 4 and § 14) are the load-bearing remainder.
- **Orogeny quarry steals** (recon 2026-07-19): uplift-value-as-fold-phase
  (coherent folding without a 3D solver); exhumation-exposes-cooked-rock
  (grade = burial + stripped overburden); jitter-renormalized-to-period.
  Cost philosophy: "coarsen the cause, never delete it and fake the
  appearance."
- **The sibling design** (earth-processes § 4, RATIFIED 2026-07-20): wind
  agent + frost/wave activation is being built against the existing
  `Agent`/lithology interfaces. Nothing here touches those interfaces;
  chapters *feed* them (an arid chapter is when a dune field forms) —
  compatibility is by leaving the agent contract alone.
- **Water/hydrology**: water.md (one quantity, two regimes; S11 GO; the
  water table is pinned by the drainage network — corrections #15);
  3e-2 decision 1 (drainage decided ONCE at the coarse tier, C inherits).

## 1. The machine as built, and precisely what is wrong with it

Reading `pregen/tectonics.rs` and `deeptime/grid.rs`:

1. Plates are Voronoi seeds with static velocities, drawn once
   (`SALT_PLATE`). `n_plates = clamp(w²/20, 3, 24)` — Small 3, Medium 14,
   **Large 24 (clamped)**, so characteristic plate diameter is ~43 km /
   ~67 km / ~208 km respectively. Nobody chose any of those numbers as a
   province-density statement (§ 10).
2. Every plate-crossing cell edge is classified once from the *static*
   relative velocities; effects (±1400 m·m orogeny, arc/trench, rift,
   ridge, transform) are painted onto cells over a 2-ring falloff and
   summed; the dominant contribution stamps the cell's `Provenance`.
3. The deep-time grid then bilinears `provenance_uplift(provenance) ×
   uplift_scale` from those 14.7 km cells into 460 m cells — **once** —
   and that constant plane is added to bedrock every iteration for 200
   iterations. `Erosion::new` even precomputes `uplift_sum` on the
   assumption it never changes.
4. The velocities never move anything. They are consumed entirely by the
   one-shot classification and discarded. There is no time axis anywhere
   in tectonics; the deep run's only dynamic external is the sea-level
   sinusoid and the re-marched climate.

Consequences, all observed in the field: one orogeny per world per place,
all of the same age; no superimposed belts, no migrating arcs, no
unconformity *machinery* (only strip-to-bedrock events); flat-lying beds
everywhere (the layer cake); belt flanks that ramp over ~50 km because the
forcing cannot express anything sharper than the cell smear; and rivers
whose exported network was computed by pregen hydrology **on the
pre-erosion coarse surface** and then carved into the *post*-erosion
deep-time terrain (§ 7 — a real inconsistency found during this pass).

## 2. The architecture in one paragraph

The seven scope items are one machine, and the load-bearing move is an
**inversion**: surface uplift stops being the input. The input becomes
**plate kinematics** — seeds + velocities, now actually advected through
N **chapters** spanning the run (§ 3). Each chapter's boundary geometry is
evaluated **analytically at deep-grid resolution** (exact
distance-to-bisector × convergence, § 4) — but what that forcing drives is
not elevation, it is **crustal thickening** of per-cell columns (§ 5).
**Isostasy** — Airy compensation applied at the flexural wavelength — then
*derives* elevation from column state each step (§ 6), which is what makes
erosion-unloading rebound, persistent relief, foreland moats and exhumed
metamorphic cores fall out of one mechanism instead of four patches.
Drainage already re-routes every iteration; chapters make it face a
*changing* tectonic landscape, and the final drainage is exported instead
of dropped (§ 7). The record gains a time stamp per unit and a compact
**world-level chapter table** from which per-chapter deformation is
*re-derivable analytically at any resolution* — the event-entry species
(§ 8) is therefore mostly free, with an explicit sparse event list for
the things plate geometry cannot re-derive (punctuation § 9, and the
igneous engine's dikes when that design lands). Everything sits behind
one `DeepConfig` flag, off by default, byte-identical when off (§ 12).

What follows takes the items in dependency order, not brief order:
forcing and columns underlie history; history underlies the record.

## 3. Tectonic history — uplift(t) as chapters

### 3.1 The chapter structure

The 200-iteration run divides into **K chapters** of equal iteration
count. Per chapter boundary:

1. **Advect** every plate seed: `pos += velocity × chapter_dt`, in world
   metres. Continental flags ride the plate. Velocities may themselves
   rotate slowly per chapter via an addressed draw (a Wilson-cycle
   half-turn over the run), or stay fixed — start fixed; rotation is a
   later knob.
2. **Re-classify**: recompute the analytic boundary set (§ 4) from the
   advected geometry. This replaces the per-cell-edge classification
   entirely; there is no painted-effects step anymore.
3. **Repaint** the thickening-rate forcing at deep-grid resolution
   (§ 4/§ 5) — cost is ~milliseconds (300 k cells × nearest-2-of-n
   plates).
4. **Ramp, don't step** (load-bearing, see § 7): the active forcing plane
   is a linear blend from the previous chapter's plane to the new one over
   the first `ramp_iters` of the chapter (default: the full chapter — the
   forcing is always in transit, like real plate reorganization). An
   instantaneous step would defeat antecedent rivers by out-running
   incision in a single 2.5 Myr tick; a ramp is what lets a river saw
   through a rising ridge, which is the entire water-gap mechanism.
5. **Fire punctuation hooks** (§ 9), then resume the ordinary iteration
   loop (flood → route → transport → weather → diffuse → record — all
   untouched).

**Chapter count K — options** (2.5 Myr/iteration under the Phanerozoic
register):

| K | iters/chapter | Myr/chapter | reads as |
|---|---|---|---|
| 4 | 50 | 125 | few, long eras; belts well-developed but at most ~2 superpositions per place |
| **8 (recommended)** | 25 | 62.5 | Earth-orogeny-length chapters; 2–4 legible superpositions; advection per chapter stays sub-plate-width |
| 16 | 12–13 | ~31 | dense biography; risk: no chapter lasts long enough for erosion to differentiate its belt before the next repaint (measure) |

Recommendation: **K = 8** default, `chapters` a config knob. Whether 25
iterations of erosion produce a *readable* belt against the new
thickening rates is a spike measurement, not a guess — if it does not,
the honest fixes are more iterations (§ 11 cost fork, user-owned) or
faster per-chapter rates, never fewer chapters faked louder.

### 3.2 Advection scale — an honest compression, stated

Earth plates move 2–10 cm/yr ≈ 20–100 km/Myr; over 500 Myr that is
10 000–50 000 km — dozens of crossings of a 251 km world. **Earth-true
advection speed is impossible at our extent and undesirable at any**: the
map would churn to noise. The design parameter is therefore **total
advection over the run, in plate widths**: default **~0.5–1.5 plate
diameters** (≈ 30–100 km at Medium), i.e. boundaries genuinely migrate
across provinces — an arc sweeps, a belt's locus moves, a margin that was
passive becomes convergent — without erasing the map's identity.
Mechanism fidelity over resolution fidelity: what the record needs is
*boundaries moving relative to columns*, not centimetres per year.

This compression has one real casualty, stated in § 13
(considered-and-rejected, "Lagrangian columns"): our columns are Eulerian
— crust does not carry its accumulated record sideways, so terranes do
not dock and strike-slip does not laterally offset landscapes. Transform
motion expresses as recorded fault events (§ 8) and collapse-tier
geometry, not as displaced terrain.

### 3.3 Recorder implications — how deformation is tagged per epoch

Two additions, both cheap:

1. **`DepUnit.chapter: u8`** — the chapter index at deposition. A
   *measurement* (when), like every other tag. It joins the tag-equality
   merge key, so units no longer merge across chapter boundaries — this
   is deliberate (a chapter boundary is a real time surface) and its cost
   is bounded: at most K−1 extra unit breaks per continuously-depositing
   column (≤ 7 units at K=8; the S9 mean is 1.42 units, so the record
   grows by a small integer factor at worst — measured in § SPIKE). The
   stamp is expected to fit `DepUnit`'s existing padding (tag 4 B + f64
   8 B + bool 1 B currently rounds to 16 B); the spike verifies `sizeof`.
   The stamp is also the **age label** the Phanerozoic register requires:
   chapter c spans `[500 − c·(500/K), 500 − (c+1)·(500/K)]` Myr before
   present, and this labeling must land **before** persistence/knowledge
   commits ages (3e-2 decision 5's deadline).
2. **The chapter table** (§ 8) — the world-level record of plate state
   per chapter, from which the deformation each unit has experienced
   since deposition is *re-derived*, never stored per cell.

### 3.4 Repainting vs the stability machinery — what must be measured

Stated as obligations, not predictions:

- **The mass ledger changes shape twice.** First, uplift is no longer
  constant: `Erosion::new`'s cached `uplift_sum` becomes per-chapter
  (`Σ_c iters_c × uplift_sum_c`), including ramp integration. Second and
  deeper: under the § 5/§ 6 inversion, *elevation is a derived quantity*
  and the conserved stock is **column thickness**: the invariant migrates
  to `Δ(ΣT + ΣH) == Σ thickening + biotic_total`, with `R` reconstructed
  from `(T, H)` by isostasy. The conservation test must be ported, not
  weakened — it stays exact, on the new stock.
- **The erodibility clamp** (journal/0029) is untouched in form —
  `expose` already runs every step, so a repaint that changes what
  outcrops is automatically re-read next iteration. The open question is
  transient behavior: a repaint moves the landscape's attractor, and the
  approach to the new equilibrium passes through knickpoint storms and
  possibly through the clamp's rails. The 0029 stress-test method (no
  runaway, no stall, nothing non-finite, N iterations at high contrast)
  re-runs **per chapter boundary** in the spike.
- **Never-incise-below-receiver** operates on per-step geometry and is
  indifferent to why the surface moved; no change, asserted by the
  existing tests under the flag.
- **Recorder churn**: a repaint redistributes erosion/deposition, and
  units-per-cell could spike at chapter boundaries. Measured (§ SPIKE 2);
  the merge machinery and the S10 overprint are not modified by this
  design, so the 665 k → 71 k optimization survives *unless* the
  chapter-stamp merge-break costs more than the bounded analysis above
  says — which is exactly what the measurement is for.

### 3.5 Determinism

All entropy from caller-owned seeds, no wall clock, unchanged:

- Plate seeds/velocities: the existing `SALT_PLATE` draws at t=0.
- Everything per-chapter (velocity rotation if enabled, punctuation
  rolls, any jitter on boundary kernels) uses **addressed draws keyed by
  (seed, new salt family `0x5B00_*`, chapter, entity)** — never sequence-
  dependent, per the addressed-draw discipline. New high-byte family so
  pregen (`0x5700_*`) and deep-time (`0x5900_*`) address spaces never
  collide.
- Advection and re-classification are pure arithmetic on drawn state.
- The iteration schedule, chapter boundaries and ramps are fixed by
  config — no convergence checks in sim logic (a harness may wrap
  `Instant` around, never inside).

## 4. Analytic boundary forcing

### 4.1 The function

Kill the painted 2-ring effects. For a deep cell at world position `x`
(and equally for any *finer* sample — see § 8.3, this is the point):

1. Find the two nearest plate seeds `(p1, p2)` (current chapter's
   advected positions). `x`'s plate is `p1`; the governing boundary is
   the `p1–p2` bisector; **signed distance** `d` = distance from `x` to
   that bisector (positive on `x`'s side), computed exactly.
2. **Convergence** `v_conv` = relative velocity of `p2` w.r.t. `p1`
   projected on the bisector normal (closing positive) — the same
   quantity the current classifier thresholds, now continuous.
3. **Boundary type** from `(v_conv, continental(p1), continental(p2))` —
   the existing taxonomy (continent–continent orogeny; ocean–continent
   arc/trench; ocean–ocean arc; rift; ridge; transform) with the
   existing thresholds.
4. **Forcing** = `amplitude(type, v_conv) × kernel(d; W, side)`:
   - `kernel` is a smooth compact bump (recommended `exp(−(d/W)²)` per
     side) — **no grid term anywhere in the expression**, so the only
     wavelengths in the forcing are the designed ones (method rule 5 is
     satisfied at the source instead of dressed after).
   - **`W` — orogen half-width — is a first-class design parameter in
     km**, per side. Default proposal: continent–continent `W ≈ 25 km`
     scaled by `1/√v_conv` (fast convergence → narrow sharp belt — the
     ratified intent stated verbatim in earth-processes § 1 item 2).
   - **Sidedness is where arc/trench asymmetry lives**: for subduction,
     the trench is a narrow negative kernel offset `D_t ≈ 10 km` onto the
     subducting side, the arc a positive kernel offset `D_a ≈ 40–60 km`
     onto the overriding side (real arc–trench gaps are ~100–250 km; ours
     compress with the world, knobs in km). These are parameters a
     designer can read, not ring counts.
5. **Junctions**: sum contributions from the boundaries to the `k = 3`
   nearest seeds (i.e. also the `p1–p3` bisector when within kernel
   reach). Bounded, smooth, and triple junctions stop being painted
   blobs.

Cost: 300 k cells × n plates distance evaluations per chapter repaint —
milliseconds at Medium's 14 plates; still trivial at a few hundred plates
(§ 10 option B) with a coarse spatial bucket over seeds if ever needed.

### 4.2 What the forcing drives

Under the § 5 inversion the function's output is a **thickening rate**
(m/iter of crustal thickness), not surface metres: orogeny thickens,
rift/trench thin, ridge is young-thin-crust creation, transform is
near-zero net with a narrow damage stripe (roughness, § 4.3; fault
events, § 8). Ocean floor carries slow thermal subsidence as a small
negative background. Surface elevation is then § 6's business.
`provenance_uplift()` — the per-provenance rate table — survives only on
the legacy path behind the flag (§ 12).

**Amplitude re-tune is mandatory and sequenced**: every amplitude in the
current system was tuned against the 50 km smeared template
(earth-processes § 1 item 2's prerequisite note). The dismal-mountains
**amplitude call (cause 3) should be made against this forcing, not
before it** — tuning against the smear and then landing this design
wastes the tuning round. Flagged for the integrating session; the spike
delivers relief distributions to tune against (§ SPIKE 7).

### 4.3 Provenance under the new scheme

Provenance today is per-pregen-cell, stamped by the dominant painted
effect, and it drives (a) `provenance_roughness` → the collapse jitter
amplitude, (b) igneous formation context (province-gated fitness),
(c) `provenance_uplift` (dies, above).

Migration:

- **Provenance becomes a classification query on the analytic forcing**:
  `provenance(x, chapter)` = the § 4.1 boundary type whose kernel
  dominates at `x`, else Craton/OceanFloor/Shelf by the base state. It is
  evaluable at any resolution and any chapter — per-pregen-cell
  provenance is retained as this function *sampled at cell centres, final
  chapter* so every existing consumer keeps its contract.
- **Roughness gains a time axis honestly**: a cell's roughness reads the
  **maximum orogenic intensity over chapters, decayed by age** (an old
  eroded belt is smoother than a young one — the superposition signature
  reaching the collapse jitter). One small per-deep-cell plane
  (`orogenic_age`/intensity, f32, ~1.2 MB) or re-derived from the chapter
  table at sample time; recommend re-derivation (it is a k-nearest-seeds
  query per sample against ≤ K chapter states — cheap, and no plane to
  store).
- **Igneous formation context** (geology.md; blocked § 2 flag): fitness
  gates on `provenance(x, chapter_of_emplacement)` — the chapter table
  finally gives intrusions a *time* as well as a place. That is this
  design handing the igneous pass what it needs, not doing its work.

## 5. Crustal columns (2.5D)

### 5.1 State

Per deep cell, three new planes beside `r`, `h`, `uplift`(→ thickening):

| plane | type | meaning | bytes @ 550² |
|---|---|---|---|
| `t_crust` | f64 | crustal thickness, m (continental ~35 000 seed, oceanic ~7 000) | 2.42 MB |
| `crust_kind` | u8 | Continental / Oceanic / Transitional (shelf) — sets density ρc ∈ {2800, 2950, 2870} | 0.30 MB |
| `exhum` | f64 | cumulative bedrock exhumed (Σ of R-lowering by incision + weathering), m | 2.42 MB |

Total **≈ 5.1 MB** against the 52 MiB run — noise. (Density as a per-kind
constant, not a per-cell plane: no consumer needs per-cell density
variation that composition kind doesn't already carry. Rejected per-cell
composition vectors in § 13.)

### 5.2 Dynamics

- **Pregen seeds it**: `crust_kind` from the plate's continental flag
  (+ Shelf where pregen pins shelves), `t_crust` from kind + addressed
  jitter, `exhum = 0`.
- **Collision is thickening**: § 4's forcing adds/removes `t_crust`.
  Physics runs on column integrals; **no voxel is ever consulted** —
  the ratified 2.5D constraint verbatim.
- **Erosion couples both ways**: every metre of bedrock converted or
  incised (`weather_cell`, the incision term) decrements `t_crust` and
  increments `exhum`; deposition grows `H` (sediment load, ρs ≈ 2400),
  not `t_crust`. Both are per-cell additions inside already-existing
  phases — parallel byte-identical by the same per-cell-kernel argument
  as today.
- `exhum` is the § 6.4 metamorphism input and is kept in `DeepField`
  (+2.4 MB resident) for the collapse tier.

## 6. Isostasy and flexure

### 6.1 Per-cell Airy at 460 m would be wrong — the honest minimum is Airy at the flexural wavelength

The brief says "Airy-style local compensation at minimum." Taken
literally per-cell, that is **physically wrong and numerically
dangerous**: real lithosphere supports loads elastically below the
flexural wavelength (tens of km); a 460 m column does not float
independently. Per-cell Airy would make every incised valley floor
rebound locally against its own ridges — an erosion↔rebound feedback at
exactly the grid scale, i.e. a stability hazard aimed straight at the
clamp machinery, and a grid-wavelength artifact of the kind method rule 5
exists to forbid.

So the v1 mechanism is **Airy compensation of the *smoothed* load**:

1. Per cell, column load `L = t_crust·ρ(kind) + H·ρs` (sea water on
   submerged cells if we care to be tidy — spike decides if it matters).
2. Smooth `L` with a separable kernel of half-width **Λ_flex** — the
   flexural parameter, a knob, default **≈ 50 km** (~110 cells at 460 m;
   ~27 cells at Large's 1.85 km cells — resolution-independent because it
   is stated in km).
3. Equilibrium surface `e_eq` from the smoothed root
   (`e_eq = (T̄·(ρm−ρc) + H̄·(ρm−ρs))/ρm − C_ref`, ρm = 3300, `C_ref`
   calibrated so a 35 km quiet continental column sits ~+400 m and a
   7 km oceanic column ~−4000 m).
4. Relax bedrock toward equilibrium each iteration:
   `r += λ_iso · (e_eq − surf)`, with **λ_iso** a rate knob (1.0 =
   instant; default < 1 for numerical gentleness — mantle response at
   2.5 Myr/iter is effectively instant, so λ_iso is a stability choice,
   measured not guessed).

This *is* a flexure approximation, not a placeholder for one: smoothing
the compensation is what plate rigidity does. It buys the two signatures
that matter at our resolution:

- **Erosion-unloading rebound**: erode a belt, the (smoothed) root
  rebounds ~ρc/ρm ≈ **0.85** of the removed thickness; net surface
  lowering per metre eroded ≈ 0.15 m, so total rock processed per metre
  of peak lowering ≈ **6.7 m** — which is the exhumation engine, and the
  dynamic answer to one-shot upheaval: uplift *responds* to erosion for
  hundreds of Myr after convergence stops.
- **The foreland moat**: beside a belt, the smoothed root exists where
  the surface load does not → the flank is pulled *below* its local Airy
  height → a depression hugging every belt → the sediment trap. Loading
  by the fill itself deepens it (H is in the load), which is real
  foreland-basin behavior. Whether the moat actually traps a readable
  sediment wedge is a spike read criterion (§ SPIKE 4c), not asserted.

**Determinism/parallelism**: the smoothing is a fixed-order gather
(running-sum per row/column, rows independent → deterministic parallel
per the established per-cell-kernel argument; or a naive fixed-tap gather
if the running sum's serial row cost matters — measured). The relax step
is pure per-cell. No new serial floor.

### 6.2 What full flexure would add, and why it does not earn v1

The full thin-plate biharmonic (`D∇⁴w = load`) adds, beyond the smoothed
response: the **forebulge** (a ~10 m-scale peripheral swell) and
Te-variation effects (craton vs young lithosphere responding
differently). Against its costs — an implicit solve or wide stencil
iteration with its own convergence/determinism obligations — a
tens-of-metres bulge at 460 m cells under hundreds of metres of local
relief jitter is **below the read floor**. Rejected for v1 (§ 13);
`Λ_flex` per-crust-kind (craton stiffer) is the cheap door left open, and
the spike's foreland measurements will say whether the approximation's
moat is deep and wide enough or whether flexure gets promoted.

### 6.3 Loop placement

Isostasy runs as a phase **after** transport/weathering/diffusion and
**before** the recorder, so the recorded ΔH per iteration reflects real
deposition/erosion, not isostatic motion of bedrock (isostasy moves `R`
only — the record mirrors `H` and is untouched by rebound, which is
correct: uplift does not deposit anything).

### 6.4 Exhumation reaching the strata record — readable metamorphic cores

Connecting to earth-processes § 5 (burial/metamorphism), with the orogeny
steal "grade = burial + stripped overburden":

- The **basement exposed at a stripped column** stops being generic
  "igneous, hardest" (the current single fallback). Its grade reads
  `f(peak_t_crust − t_crust_now, exhum)` — i.e. how deep this rock once
  sat and how much section has passed through the surface above it.
  Concretely at collapse: `exhum` and thickening history select among
  basement classes (schist → gneiss → migmatite-grade toward deeply
  exhumed belt cores, unmetamorphosed basement in cratons) via the
  ordinary class-fitness machinery — a **content roster addition for the
  metamorphic slice geology.md already sequences**, keyed by axes this
  design provides. This document provides the axes, not the roster.
- Grade **zonation** falls out spatially for free: `exhum` is largest in
  belt interiors and tapers outward, so a traverse reads core → flank
  grade descent — the § 5 signature, discoverable, and the
  prospecting-payoff hook (ore roster's "porphyry at arcs, veins near
  intrusions" wants exactly these axes).
- The erodibility coupling benefits automatically: harder high-grade
  basement outcropping in old cores widens the shield effect journal/0029
  already measured — no new coupling code, `expose` just meets more
  classes when the lithology table gains them.

## 7. Drainage re-march per chapter

### 7.1 What is actually one-shot today (a correction to the brief's framing)

Deep-time drainage is **not** one-shot — `flood`/`route`/
`accumulate_area` re-run every iteration. What is one-shot is (a) the
**forcing** it answers (uplift never changes, so drainage converges to
one attractor and sits there — no tectonic event ever re-poses the
question), and (b) — found reading `collapse.rs` during this pass —
**the exported river network is not deep-time drainage at all**: the
rivers carved into the world come from pregen `Cell.river`/`discharge`
(14.7 km chord segments), computed by pregen hydrology **on the
pre-erosion coarse elevation**, while the terrain they are carved into is
the deep-time eroded surface. Two authorities, one of them stale. This
design's § 7 deliverable fixes both.

### 7.2 What persists across chapters, and what the landforms cost

Nothing new persists: **the terrain is the memory.** A river's incised
valley is its own record — being locally lowest, it keeps winning D8
routing next iteration. Antecedence is therefore *emergent*, with one
condition this design must supply: chapter forcing **ramps** (§ 3.1.4).
Under a ramp, a transverse river crossing a rising axis keeps pace by
incision and saws a gorge — a **water gap** — while a step function
would dam and divert it. The specific landforms and where they come from:

- **Water gaps**: emergent from ramped uplift × per-iteration re-routing,
  visible at 460 m (a gorge is many cells long). Read criterion in
  § SPIKE 4a.
- **Basin captures / divide migration**: emergent from differential
  chapter uplift; the record shows a discharge/energy jump in the
  captured valley's units (a *measured* tag change — no new machinery).
- **River terraces from uplift pulses**: sub-460 m landforms; deep time
  supplies the *cause* (incision-rate history in the record), the
  C-refinement/collapse tier supplies the *shape*. Explicitly not a deep-
  grid deliverable — coarsen the cause, refine on approach.
- **Superimposed valleys, underfit streams**: emergent from chapter
  sequence; no code, just read-quality checks.

### 7.3 Named coupling points with the slated deep-hydrology work

The two designs must know about each other (ratified). The couplings, by
name, so water.md's next pass can cite them:

1. **`DeepField` exports drainage instead of dropping it.** Today the
   run keeps `surf + strata` only. Add: final-chapter `recv` (D8
   receiver), `area`/discharge, and the lake mask (depression-filled
   cells at final sea stand). Consumers: (a) the **3e-2 frozen macro
   drainage topology** (decision 1: drainage decided ONCE, coarse; the
   river-conditioning corridor mechanism inherits it) — replacing the
   stale pregen chord network of § 7.1(b) as the collapse carving
   source, one authority; (b) the **water-table pinning lattice**
   (corrections #15: a water table is pinned by the drainage network —
   S11's production form needs exactly this lattice); (c) the **body
   graph's initial bodies** — lakes and the sea seeded as S11
   pinned/finite bodies with levels from the deep run.
2. **Per-chapter channel masks** — one bit per cell per chapter
   (~300 k × K bits ≈ 0.3 MB at K=8): the **paleo-channel record**.
   Consumer: the erosional-cave family (water.md § "what survived the
   sweep": an abandoned conduit is a former channel the water table later
   dropped below — the cheapest cave family precisely because this
   design was going to compute its substrate anyway). Cheap enough to
   keep speculatively; drop if the cave design doesn't ratify.
3. **One sea-level datum.** The deep run's paleo-sea curve ends at some
   final stand; the water design's ocean body is level-pinned (DECIDED).
   These must be the same number from the same config, or shorelines and
   the marine record disagree with the sea the player meets.
4. **Timescale ownership boundary**: deep time owns drainage *history*
   (chapters); the water design owns drainage *presence* (bodies, flow,
   saturation) from year zero on. The handoff artifact is item 1's
   exported planes — nothing else crosses.

## 8. Recorder event-entries — the second species

### 8.1 The data model

The record must express things that *modify previous entries* — tilt,
fault offset, intrusion, truncation — which a deposition-ordered stack
cannot (the § 2 igneous flag is blocked on exactly this). The design
principle: **events key on TIME, not on unit indices, and store causes,
not consequences.**

Three parts:

1. **`DepUnit.chapter: u8`** (§ 3.3) — every unit knows *when*.
2. **The chapter table** (on `DeepField`): per chapter, the plate state
   — seed positions, velocities, continental flags — plus that chapter's
   punctuation events. ~n_plates × 5 f64 ≈ 560 B/chapter at Medium;
   **the entire tectonic history of a world is ~5 KB.** From it, the
   deformation any unit has experienced is *re-derivable analytically*:
   the § 4 forcing function evaluated at any `(x, chapter)`.
3. **A sparse explicit event list** for what plate kinematics cannot
   re-derive: `StructEvent { kind, chapter, support, params }` with
   `kind ∈ { FaultSlip, Intrusion, Truncation, Punctuation(...) }` and
   `support` a compact spatial primitive (segment + width for a fault
   trace; disc for an impact; region ref for a flood basalt). World-level
   list, **not per-cell** — dozens of entries per world, bytes to
   kilobytes. Per-cell participation is a geometry query at read time.

An event at chapter `c` applies to every unit with `unit.chapter < c`
(and to basement). Erosion popping units never invalidates events
(an event whose units are gone is vacuous); the merge machinery is
untouched because **units never store deformation** — so the
665 k → 71 k overprint optimization survives *by construction*, not by
care.

### 8.2 Why tilt is (mostly) not an event entry at all

Dip and fold are continuous consequences of differential uplift, and S9
already classified fold-phase as a cheap **relax** term (`fold-phase =
f(uplift)` at sample time — the orogeny steal). Under this design the
per-chapter uplift field is an analytic function of the chapter table —
so **the collapse tier computes a unit's attitude directly**: at column
`x`, a unit deposited in chapter `u` has been carried through chapters
`u+1..K`; its dip is `g(Σ_{c>u} ∇F_c(x))` — the accumulated tilt of the
differential uplift it survived, evaluated from ~5 KB of state at
**collapse resolution, not 460 m resolution**. Younger beds accumulate
fewer terms → they dip less → **angular unconformities emerge** wherever
an unconformity flag separates units whose accumulated-tilt sums differ.
The layer cake dies without a single stored dip vector, and the beds
"match orientation properly with the tectonic event" because the
orientation *is* the event, replayed.

Fault offsets are the discrete exception (a throw is not a gradient):
`FaultSlip` events derive candidate traces from chapter boundary
geometry (transform/convergent bisector segments + addressed jitter,
renormalized-to-period per the orogeny phantom-fault lesson), each with a
throw; the collapse tier displaces unit datums across the trace for
units older than the event. This is also precisely the marker-bed puzzle
substrate (§ 2's ash stripe offset by the same throw — already a
things-that-will-happen line).

### 8.3 Collapse-tier reading, and the resolution win

The collapse column materializer's inputs grow from `(units)` to
`(units + chapter table + event list)`. All three are read-only, tiny,
and available wherever `DeepField` is — including inside C-refinement
regions and the per-voxel member dither. Because deformation is
re-derived analytically, **structure does not step at the 460 m record
grid**: dip varies continuously at voxel resolution while the *facies*
story still steps at deep cells (the ratified, dressed contact). No
square boundaries; nobody screams.

### 8.4 Wire and growth discipline

- **Postcard is positional** (corrections #3): no `skip_serializing_if`,
  anywhere, ever. New fields on persisted/wire-adjacent types
  (`DepUnit.chapter`, the chapter table, the event list) are appended,
  never inserted; enums (`StructEventKind`, `Provenance` if it ever goes
  to wire) gain variants at the end only.
- Growth bounds, restated: units +0 B expected (padding; spike verifies),
  count growth bounded by K−1 breaks per column (measured); chapter
  table ~5 KB/world; events O(dozens). The record's memory story remains
  the S9/S10 one.

## 9. Punctuation hooks — the interface only

Each event type is its own later design (ratified). This section fixes
only the contract so the chapter machinery admits them:

- **When**: at chapter boundaries, after repaint, before the loop
  resumes (§ 3.1.5).
- **Whether**: addressed draws — `draw(seed, SALT_PUNCT, chapter, kind)`
  against per-kind per-world budgets. The **≤ 1 impact per world** rule
  is structural: one draw at world creation selects the impact's chapter
  (or none), rather than per-chapter rolls that need coordination.
- **What an event may touch** (the whole interface):
  1. the forcing plane for its chapter (flood basalt: an extrusive
     thickening + a resistant lithology entering the record as a regional
     unit — also the natural **marker bed** mechanism the igneous design
     owns; mega-landslide: an instantaneous mass redistribution respecting
     the ledger; impact: crater topography + a `StructEvent` truncation +
     a shocked/melt unit),
  2. the recorder, via ordinary units and/or a `StructEvent`,
  3. the mass ledger, as a **declared external input** (like
     `biotic_total` — every event kind reports its Δmass so conservation
     stays exact).
- **What it may not touch**: iteration order, the clamp rails, any
  undeclared plane. Determinism: everything from the addressed draw and
  the chapter state.

## 10. The plate-scale knob — USER DECISION (options + recommendation)

Today `n_plates = clamp(cells²/20, 3, 24)` is an emergent constant nobody
chose. It directly sets **provinces per km of travel** — how often a
walking player crosses into a new tectonic story. Characteristic plate
diameter `d ≈ extent/√n`; a straight 100 km walk crosses roughly
`100/d` boundaries (each now dressed with the § 4 orogen/arc structure
rather than a smear).

**A found defect the knob should fix regardless of choice**: the clamp at
24 makes Large worlds *emptier per km* than Medium (d ≈ 208 km vs 67 km)
— province density silently depends on extent, backwards if anything.

| option | Medium (251 km) | Large (1017 km) | provinces / 100 km walk | reads as |
|---|---|---|---|---|
| **A — status quo, documented** | 14 plates, d≈67 km | 24 plates, d≈208 km | ~1.5 (M), ~0.5 (L) | today's look at Medium; Large is sparse and inconsistent with it |
| **B — plate diameter in km (recommended)**, default 65 km | ~14, d≈65 km | ~230, d≈65 km | ~1.5 everywhere | today's ratified Medium look made *deliberate* and extent-uniform; knob in a unit the user can feel |
| **C — continental realism**, d≈130–150 km | 3–4 plates | ~50 | ~0.7 | one or two great belts per Medium world; majestic, sparse; a world is "a region", not "a continent" |
| **D — province mosaic**, d≈40 km | ~40 plates | ~650 | ~2.5 | drama every valley; **risk**: plate spacing approaches orogen width (2W ≈ 50 km § 4), the map becomes all-boundary and provinces lose identity |

**Recommendation: B** — express the knob as *characteristic plate
diameter in km* (`plate_scale_km`, default 65, n derived from area and
clamped to ≥3), because it preserves the Medium look the user has been
judging all along, fixes the Large inversion, and is the only formulation
in the unit the decision is actually about. Options C and D remain
reachable *as values of B's knob* — which is the strongest argument for
B: it makes the others settings, not architectures. Engineering note
either way: `Cell.plate` is already u16, so counts beyond 24 cost
nothing structural; classification cost stays trivial (§ 4.1).

Interaction to state: total advection (§ 3.2) is expressed in plate
widths, so this knob and the advection scale compose predictably —
smaller plates + same advection fraction = more boundary migration per
km², i.e. the knob also scales *history* density, not just spatial
density. The spike's read-quality pass runs at B-default and one
extreme (§ SPIKE 4).

## 11. Cost against the ritual budget

The shipped ritual is ~14 s at every extent (DEEP_MAX_WIDTH holds cell
count flat). Additions, by cost class (**estimates to be measured — the
spike's § SPIKE 3 replaces this table; per corrections #12 the spike must
measure the production entry point and name the driver**):

| addition | class | basis |
|---|---|---|
| chapter repaint + reclassify (×K) | ~ms total | 300 k × n-plate distance evals, K times |
| crustal-column bookkeeping | +0.2–0.5 s | per-cell adds inside existing parallel phases |
| isostasy: load smooth + relax (×200 iters) | +0.5–2 s | two O(n) separable passes + one per-cell pass per iteration; the only new per-iteration cost of note |
| recorder stamps / chapter table / events | noise | bytes |
| drainage export planes | noise | already computed; +~5 MB resident |
| **total, same iteration count** | **≈ +1–3 s → ~15–17 s** | inside the ~15 s ritual class |

**The honest fork**: if the spike finds K=8 chapters × 25 iterations too
short for readable per-chapter belts (§ 3.1), the fix is **more
iterations** — 300 iters ≈ +50 % wall (~21 s), 400 ≈ ~28 s. Per the
standing rule, worldgen time is not a constraint and ready-made/longer
rituals are sanctioned — but this is a real tradeoff and it is
**user-owned**: the spike reports relief-per-chapter at 200/300/400 so
the user chooses a biography length with the price tag visible. Memory:
+~10 MB working, +~8 MB resident (`exhum` + drainage export + masks) —
noise against 52 MiB.

## 12. Byte-identity and the flip strategy

One flag: **`DeepConfig::tectonic_history: bool`** (name bikesheddable),
default `false`, same class as `erodibility`/`biotic`:

- **Off**: no chapters (K=1, no advection), `provenance_uplift` bilinear
  plane exactly as today, no column planes allocated, isostasy phase
  no-ops (empty planes read as identity — the established pattern), no
  chapter stamps written (recorder path byte-identical), no exported
  drainage. Every existing world reproduces **byte-identically**; the
  determinism suite asserts it.
- **On**: the whole bundle. Items 1–4 do not flip separately — thickening
  without isostasy is meaningless, chapters without analytic forcing
  re-smears them — and a combinatorial flag matrix would multiply the
  byte-identity test surface for no consumer. (The brief's per-item
  granularity is honored in the *design*, not the flag.)
- **Production flip is the user's appearance call**, from spike
  screenshots/numbers, exactly like 0030's "flip it, i want to see" —
  and it CHANGES TERRAIN SHAPE for every world created after; worlds made
  before are not reproducible under it. The amplitude call rides
  *with or after* this flip, never before (§ 4.2).

## 13. Considered and rejected (and why — kept, per the brief, as a section of record)

- **Closed spherical/wrapped plate topology** — re-litigates S7's
  ratified continent-disc; the ocean ring remains the closure; every
  benefit this design needs (moving boundaries, superposition) exists on
  the disc.
- **Mantle-convection–driven plate motion** (self-organizing velocities)
  — enormous machinery to *generate* what a 5-draw-per-plate kinematic
  table already expresses; no read-quality difference at our extents.
  Coarsen the cause: the cause is "plates move," not "the mantle churns."
- **Lagrangian columns (crust/record advecting laterally with plates)** —
  the honest casualty (§ 3.2). At ≤ ~2 cells of motion per chapter,
  lateral record transport is at grid noise scale, while the cost — the
  entire recorder, collapse addressing, and refinement machinery is
  column-addressed — is a rewrite of everything. Eulerian columns under
  moving forcing keep every readable signature except terrane docking and
  lateral strike-slip offset of landscapes; fault events carry the
  latter's *record*. Permanent, stated simplification.
- **Full biharmonic flexure in v1** (forebulge, Te fields) — the
  forebulge is ~10 m-class at our scales, below the read floor under
  local relief jitter; smoothed-load Airy already yields the foreland
  moat (§ 6.2). Promoted only if the spike's foreland read fails.
- **Per-cell Airy at 460 m** — physically wrong wavelength, invites a
  grid-scale erosion↔rebound feedback (§ 6.1). Rejected *in favor of*
  the smoothed form, not deferred.
- **Per-cell event entries** (`Vec<StructEvent>` per column) —
  O(cells × events) duplication of what is analytically re-derivable
  from ~5 KB of chapter state; would also bloat the record the
  665 k → 71 k optimization exists to protect.
- **Inline event entries in the unit stack** (`enum { Unit, Event }`) —
  breaks `sum(units)==H`, the merge fast path, and pop-on-erosion
  semantics; every consumer pays a skip branch forever. Time-keyed events
  beside a time-stamped stack express strictly more for strictly less.
- **Stored per-chapter uplift planes** (K × 2.4 MB) — the analytic
  forcing makes them re-derivable at *better* than stored resolution
  (§ 8.3); storing them would re-introduce the 460 m quantization of
  structure this design exists to remove.
- **Continuous advection (repaint every iteration)** — K=200 chapters'
  worth of classification churn and record fragmentation for no
  read-quality gain over ramped chapters; the ramp already makes forcing
  continuous in time.
- **Stress/strain tensor fields for folding** — fold-phase-from-uplift is
  the S9-classified relax term and the orogeny-proven cheap path; a
  tensor field is resolution fidelity, not mechanism fidelity.
- **Re-running pregen hydrology per chapter** — deep time already
  re-routes every iteration at 32× finer resolution; the pregen chord
  network should *lose* its terrain authority (§ 7.3.1), not gain a time
  axis.
- **Per-cell density/composition vectors on columns** — no consumer;
  `crust_kind` + constants carries isostasy and the metamorphic axes.
  A later mantle/ore design can add planes behind its own flag.
- **Thermochronology / cooling-age tracking** — no consumer until a
  knowledge-system feature asks "when did this rock cool"; `exhum` +
  chapter stamps reconstruct a coarse answer for free if one ever does.

## 14. Findings against the ratified scope (things the sources contradict)

Stated plainly rather than designed around:

1. **"Rivers currently share the one-shot disease" is half-wrong, and
   the true defect is worse** (§ 7.1): deep-time drainage re-routes every
   iteration (not one-shot); but the river network actually carved into
   the world is pregen's 14.7 km chord network computed on the
   **pre-erosion** surface — a stale second authority over the eroded
   terrain. The scope item's *remedy* (couple drainage to chapters,
   export it) stands; its diagnosis needed correcting.
2. **Literal per-cell Airy — the brief's "local compensation at minimum"
   — is not a valid minimum** at 460 m (§ 6.1); the minimum honest form
   is compensation at the flexural wavelength. The scope's intent
   (rebound, persistence, exhumation) is fully served; the letter is not.
3. **`Erosion::new` caches `uplift_sum` once** — a small but real code
   assumption ("uplift is constant across iterations") that uplift(t)
   invalidates; the mass ledger must go per-chapter and then migrate to
   thickness-space entirely under the inversion (§ 3.4). Flagged so the
   spike doesn't discover it as a mysterious conservation failure.
4. **The plate-count clamp inverts province density with extent**
   (§ 10): Large worlds are ~3× sparser in provinces-per-km than Medium.
   Not in any Observed entry; found by arithmetic on the formula. The
   knob decision should subsume the fix.
5. **Chapter stamps break unit merging across chapter boundaries**
   (§ 3.3) — a bounded, deliberate record-growth cost the 665 k → 71 k
   history says must be *measured*, not assumed fine (§ SPIKE 2).

## 15. Open questions, sorted (none left unsorted)

### User-owned (feel / scope / appearance)

| # | question | recommendation |
|---|---|---|
| U1 | **Plate-scale knob** (§ 10) | Option B: `plate_scale_km`, default 65 |
| U2 | **Chapter count K** — how long a biography a world carries | K = 8 default, shipped as a knob |
| U3 | **Ritual-length fork** if readable chapters need >200 iterations (§ 11) | accept up to ~2× (≈ 28 s) if the spike shows relief-per-chapter demands it; the register supports "generating world history…" taking visible time |
| U4 | **Orogen width `W` and arc-offset defaults** (§ 4.1) — belt sharpness is an appearance call once measurable | W ≈ 25 km cc-collision, arc gap ≈ 50 km; judge from spike renders |
| U5 | **Advection scale** — how far boundaries migrate over a world's life (§ 3.2) | ~1 plate width per run |
| U6 | **Punctuation budgets** — how many flood basalts / landslides per world; impact yes/no by default (§ 9) | ≤1 impact structural; others ~1–2 rolls/world; each event type's own design pass decides its content |
| U7 | **The amplitude call (dismal-mountains cause 3)** — already user-owned; re-sequenced by this design to AFTER analytic forcing lands (§ 4.2) | make it once, against the new forcing, with § SPIKE 7 data |
| U8 | **Production flip** of `tectonic_history` (§ 12) | after spike screenshots, 0030-style |

### Measurable (the spike answers; see § SPIKE)

- M1 Clamp/knickpoint stability across chapter repaints (ramped vs
  stepped) — § SPIKE 1.
- M2 Record growth from chapter stamps + repaint churn; DepUnit sizeof —
  § SPIKE 2.
- M3 Ritual wall/memory at 200/300/400 iterations, production driver —
  § SPIKE 3.
- M4 Landform evidence: water gap, superposition, foreland wedge, angular
  unconformity, exhumed core zonation, rebound persistence — § SPIKE 4.
- M5 Forcing wavelength: measured gradation-to-peak ≈ designed W (the
  50 km artifact must die measurably) — § SPIKE 5.
- M6 Byte-identity off-flag; determinism on-flag — § SPIKE 6.
- M7 Relief distributions for the amplitude call — § SPIKE 7.
- M8 Isostasy numerics: λ_iso / Λ_flex stability region; whether the
  smoothing's running-sum rows need the gather form — § SPIKE 1/3.
- M9 Whether per-chapter channel masks earn their 0.3 MB (does the
  erosional-cave design consume them?) — deferred to that design; keep
  behind the flag meanwhile.

## § SPIKE — what the follow-up spike must measure

(The light.md § 10 discipline: measurements and proofs, named in advance;
name the code path that produces every cost number — corrections #12.)

1. **Stability under repainting.** Per chapter boundary, the 0029 stress
   protocol (high contrast, N iterations): no runaway, no stall, nothing
   non-finite; erodibility multipliers stay inside the clamp rails;
   never-incise invariant holds. Compare **ramped vs stepped** repaints:
   knickpoint-count and ΔH-variance transients after each boundary.
   Verify the isostasy loop's stability region over (λ_iso, Λ_flex) and
   report the safe defaults; confirm no grid-wavelength artifact appears
   in the rebound field (spectral check at the two cell sizes, 460 m and
   Large's 1.85 km).
2. **Recorder growth per chapter.** Units/cell and total units vs
   K ∈ {4, 8, 16} at Medium, biology on, against the ~71 k baseline;
   distribution tails (the 5+ unit columns); `sizeof(DepUnit)` with the
   chapter stamp; chapter-table and event-list bytes. The merge
   optimization must survive within a stated small factor — the spike
   states the factor, the user judges it.
3. **Ritual cost.** Wall time and peak/resident memory at Medium and
   Large caps, flag on vs off, at 200/300/400 iterations, **measured at
   the production entry point (`build_field`) on the byte-identical
   parallel path** — and once on the scalar path, labeled, for the
   determinism comparison. Report the per-phase breakdown so isostasy's
   real per-iteration cost is on record.
4. **Landform evidence criteria** (each a read test with a stated pass
   bar, screenshots for the user):
   a. **Water gap**: ≥1 transverse channel maintaining course through a
      late-chapter ridge with upstream area continuity across the axis
      (and none under stepped repaint — the negative control).
   b. **Superimposed orogenies**: a place bearing two belts of different
      chapters where the older shows lower relief, higher `exhum`, and a
      record/roughness signature distinct from the younger.
   c. **Foreland wedge**: sediment-thickness maximum adjacent to a belt
      flank, in the moat, thinning away — present with smoothed-Airy on,
      absent with isostasy off (the mechanism attribution control).
   d. **Angular unconformity**: columns where accumulated-tilt difference
      across an unconformity flag exceeds a legibility threshold at
      collapse resolution; count them; render one.
   e. **Exhumed core**: belt-interior basement exposure with `exhum`
      above threshold and outward zonation along a traverse.
   f. **Rebound persistence**: after a belt's convergence ends
      (chapter table goes quiet there), relief half-life in chapters —
      must exceed the no-isostasy control by a stated factor (the
      "relief persists for hundreds of Myr" claim, made falsifiable).
5. **Forcing wavelength.** Measured gradation-to-peak distance of a
   fresh belt vs designed `W` (the 50 km smear must collapse to the
   design value within tolerance); confirm no 14.7 km-cell signal
   survives in the uplift field's spectrum.
6. **Determinism.** Flag-off: byte-identical `DeepField` against shipped
   main (the existing fingerprint tests). Flag-on: double-run
   byte-identity; scalar↔parallel byte-identity (isostasy smoothing
   included); seed sensitivity; chapter-table replay identity.
7. **Amplitude-call inputs.** Relief/hypsometry distributions and belt
   cross-sections at 2–3 thickening-rate settings under the new forcing,
   packaged for the user's cause-3 decision (§ 14.1 of ROADMAP's
   sequencing: nothing more should be photographed for amplitude until
   this exists).
8. **Drainage export fidelity.** The exported final drainage matches the
   last iteration's routing exactly (trivial but load-bearing for
   3e-2/water); the pregen chord network's carving authority is retired
   behind the flag with a before/after render of a river valley.
