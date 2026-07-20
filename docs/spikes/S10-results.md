# S10 results — the biotic layer on the deep-time A-tier

Status: spike complete, 2026-07-20. Code in `crates/dc-worldgen/src/deeptime/biotic.rs`
(plus the `Biofacies` axis and pedogenic overprint in `recorder.rs`, two biotic
modifier planes in `grid.rs`, their consumption in `erosion.rs`, and the lagged
step order in `mod.rs`); measurement harness `examples/biotic_spike.rs`;
falsifiers in `tests/biotic.rs`. Numbers below from the Windows dev box, release
profile, seed `0x0D5E_ED57_2026`, Medium world (17×17 pregen cells, ~251 km),
200 iterations. The spike is **additive**: the biotic layer is off by default
(`DeepConfig::biotic`), and with it off every path is byte-identical to the
pre-S10 engine.

The question (ecology.md § 6): implement the **community vector + the six
processes** on the S9/3e-1 A-tier and measure (a) the cost against the ~14 s
world-creation ritual and (b) read-quality — do we get **coal seams**,
**paleosols**, **charcoal bands**, **retrogressive surfaces**? Evolution is
explicitly NOT in S10; S10 exists to prove the substrate evolution will ride.

**Verdict: all four signals present and legible; cost is +66 % wall / +6 MiB
persistent on the A-tier ritual, linear in cells and bounded by the existing
width cap. Recommendation: GO — NEEDS RATIFICATION (§ Recommendation).**

## What was built

A per-cell ecological community propagated over the same eons the two-plane
erosion runs, in our idiom:

- **Community vector + soil state** (`CellBiota`, 72 B/cell): vegetation cover
  over a 7-species vanilla roster, plus soil depth, N, available P, rock-derived
  P, base cations, time-since-disturbance, and the previous epoch's surface and
  alluvium thickness (the erosion-awareness the layer needs).
- **A species niche contract** (`Niche`): tolerance ranges over temperature,
  moisture and soil depth; N/P demands; and traits — N-fixing, shade/incumbency,
  litter yield, root cohesion, biological-weathering boost, flammability,
  waterlogging preference. A registry-defined organism pack would publish these;
  the spike ships a fixed vanilla roster (lichen → grass → N-fixer → forest;
  peat-sedge; sclerophyll; fire-grass).
- **The six processes per epoch**, as an ordered read/write phase list (below).
- **Biotic tags in the strata record**: a new `Biofacies` merge-key axis on
  `DepTag` — `Mineral` (default), `Soil`, `Peat`, `Coal`, `Charcoal`, `Retro`.
- **Biology as an erosion term**: two per-cell modifier planes on the grid that
  the *next* epoch's erosion consumes — a weathering multiplier (root acids /
  mycorrhizae) and a hillslope-creep resistance (root cohesion).

### The lagged coupling (ecology.md § 3, the stated architectural rule)

Biology modifies erosion; erosion modifies terrain; terrain sets climate and
soil; soil sets biology. A single-epoch pass graph would correctly refuse that
cycle. As designed, the loop is:

```
for each epoch:
    erosion.step(grid)          # consumes the modifiers biology wrote LAST epoch
    biotic.step(grid, erosion)  # reads the fresh terrain; writes NEXT epoch's
                                # modifiers + this epoch's organic record
```

On epoch 0 the modifiers are their identity values (`1.0` weathering, `0.0`
resistance), so epoch 0's erosion is byte-identical to a biology-free run.

### Pass-graph integration (no new pregen pass)

The biotic layer rides **inside** the existing `dc:pass/deep-time` pass, which is
already the *creator* of `Resource::DeepStrata`; `dc:pass/clastic-deposition` is
already a *reader* of it. So the biotic annotations reach collapse through an
existing creator→reader edge and no new pass was added — which means
registration-order independence is **inherited**, not re-derived
(`pipeline::tests::order_ignores_registration_order` still green). This was a
deliberate choice over minting a `DeepBiota` resource: declaring a write the
pass does not currently fulfil (production keeps biology off, below) would be a
dishonest declaration, and the graph rejects dishonesty by design.

## The six processes — implementation status

All six are **real**; none is a stub. Honest per-process notes:

| # | process | status | mechanism |
|---|---|---|---|
| 1 | **Suitability** | real | `min` over temperature / moisture / soil-depth / P / N / waterlogging terms — Liebig's minimum, the operator is `min`, never a weighted mean. |
| 2 | **Dispersal** | real, simple | Bounded 8-neighbour kernel on a frozen previous-cover snapshot (a *relaxing* process by S9's classification), entering competition as a **saturating gate** plus a weak background rain from the regional pool. |
| 3 | **Competition** | real | Finite capacity set by the best suitability present; shares allocated by intrinsic competitive rank (shade/stature) × suitability, then incumbency (established shade-casters resist displacement). |
| 4 | **Nutrient cycling** | real | Uptake → litter → decomposition return (closing the cycle) → finite rock-P weathering release → leaching/occlusion loss. **The Walker & Syers curve emerges** rather than being scripted (see § Retrogression). |
| 5 | **Niche construction** | real | Writes soil depth, the two erosion modifiers the next epoch consumes, and the fuel load. |
| 6 | **Disturbance** | real (fire), partial (flood) | Fire: deterministic addressed ignition from fuel × dryness, killing cover (fire-adapted species resist), releasing an ash nutrient pulse, and leaving a charcoal bed. Flood: drainage-area-gated succession reset — it resets the successional clock but leaves **no distinct tag of its own**, because the overbank mineral band the erosion recorder already writes *is* the flood's signature. |

**What the A-tier cannot express** (honest limits, not omissions):

- **Vegetation's effect on channel form.** "Vegetation invented meandering
  rivers" (ecology.md § 1) is real and we did not model it: at 460 m cells a
  channel is a cell-to-cell chord, so there is no planform to make sinuous. Root
  cohesion is coupled to *hillslope* diffusion and weathering only; the
  stream-power transport chain is untouched (which also keeps the S9b serial
  flux chains and their byte-identity undisturbed).
- **Individual plant placement.** ecology.md's derivation chain ends at
  "individual placement at chunk scale from the cell's community vector by
  addressed hashing." That is collapse-tier work and out of S10.
- **Fauna.** Untouched (ecology.md § 5 open fork 1 is still the user's).

## Cost — the ON/OFF curve

Medium world, 200 iterations, recorder on, same seed. "Peak" is the analytic
working set (grid planes + recorder + erosion scratch + biotic scratch + a
priority-flood heap high-water estimate) — the same honest, reproducible
accounting S9 used, not an OS sample.

| cell | w | cells | OFF wall | ON wall | ratio | OFF peak | ON peak | units OFF | units ON |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 km | 251 | 63 001 | 2.70 s | 4.85 s | **1.80×** | 12.4 MiB | 26.5 MiB | 71 883 | 131 032 |
| 700 m | 358 | 128 164 | 5.93 s | 10.32 s | **1.74×** | 24.3 MiB | 53.5 MiB | 134 203 | 279 254 |
| 500 m | 501 | 251 001 | 12.66 s | 21.06 s | **1.66×** | 46.4 MiB | 104.3 MiB | 251 251 | 564 478 |
| **460 m (A)** | 545 | 297 025 | **15.17 s** | **25.19 s** | **1.66×** | **54.6 MiB** | **123.2 MiB** | 292 386 | 673 102 |
| Small, 460 m | 160 | 25 600 | 1.16 s | 2.12 s | 1.82× | 5.2 MiB | 11.4 MiB | — | — |

**Headline (the A tier, the ~14 s ritual):** `15.17 s → 25.19 s` (**+10.0 s**,
1.66×); peak `54.6 → 123.2 MiB` (+68.6 MiB). The OFF re-measure (15.17 s /
54.6 MiB) is consistent with S9's recorded A datapoint (14.1 s / 52 MiB) on the
same machine.

### The shape of the curve — it is LINEAR, not nonlinear

The ON/OFF ratio *falls slightly* with grid size (1.80 → 1.74 → 1.66 → 1.66)
rather than growing. The biotic step is a pure per-cell pass with a fixed
8-neighbour gather: **O(cells), no heap, no global dependency chain**. Erosion,
by contrast, carries the priority flood's `n log n`. So biology's share of the
step *shrinks* as the grid grows, and the ratio asymptotes near 1.66×.

Absolute per-cell-per-iteration cost at the A tier:

- erosion (OFF): `15.17 s / (297 025 × 200)` = **0.255 µs**
- biotic delta: `10.02 s / (297 025 × 200)` = **0.169 µs**

**The width cap makes this extent-proof.** `DEEP_MAX_WIDTH = 550` already holds
the deep grid to ≤ 550² cells at any extent (Large runs a coarser ~1.85 km cell,
550² = 302 500 cells — within 2 % of Medium's 545²). So the biotic layer's
+10 s is paid at *every* extent, not scaled by it: **Small +1.0 s, Medium
+10.0 s, Large +10.1 s (projected from the equal cell count).**

### Memory: transient vs persistent — the number that matters is small

`DepUnit` is **16 B** (adding the `Biofacies` axis did **not** grow it — the
fourth 1-byte enum fits in `DepTag`'s existing padding). `CellBiota` is 72 B.

- **Transient** (dropped when the run ends, like the erosion scratch): the
  biotic working set ≈ 204 B/cell (`CellBiota` 72 + frozen prev-cover 28 +
  per-cell outcome ~104) plus 8 B/cell of modifier planes ≈ **60 MiB** at the A
  tier. This is the bulk of the +68.6 MiB peak.
- **Persistent** (what actually survives into the `DeepField` the world keeps):
  only the extra strata units — `(673 102 − 292 386) × 16 B` = **6.1 MiB**.

So the world-state cost of the biotic layer is **~6 MiB on a ~55 MiB field**,
and the rest is a transient spike during world creation. Record size grows
2.30× in units (mean 1.30 → 3.24 units per readable column).

## Read-quality — the four target signals

A tier (460 m), Medium, seed `0x0D5E_ED57_2026`, 200 iterations. 207 548
readable columns.

| signal | columns | % | count | verdict |
|---|---:|---:|---:|---|
| **Coal seams** | 1 133 | 0.55 % | 1 310 seams, max **24.03 m**, 1 398 m total | **present and legible** |
| **Paleosols** | 47 567 | 22.92 % | 79 175 buried soil horizons | **present and legible** |
| **Charcoal bands** | 41 884 | 20.18 % | 158 310 beds | **present and legible** |
| **Retrogressive surfaces** | 48 776 | 23.50 % | 48 838 horizons | **present and legible** |

(Also: 2 858 peat-bearing columns, 138 987 columns carrying any organic horizon.)

### 1. Coal seams — **present and legible**

Deep cell (466, 371) = **world voxel (107338, 58787)**, surface 57.5 m, H 98.37 m,
17 units (top first, abridged):

```
   2.083 m  [Sa/A/M   ]  subaerial
   0.033 m  [Sa/A/L·Ch]  CHARCOAL band (fire event)
   1.252 m  [Sa/A/M   ]  subaerial
   0.040 m  [Sa/A/L·Ch]  CHARCOAL band (fire event)
   4.586 m  [Sa/A/M   ]  subaerial
  24.028 m  [Sa/A/L·Co]  COAL SEAM (buried compacted peat)   <-- paleosol
   6.054 m  [Sa/H/M   ]  subaerial
   1.387 m  [Sa/H/L·Co]  COAL SEAM (buried compacted peat)   <-- paleosol
   0.520 m  [Ss/-/L   ]  marine
   9.046 m  [Ss/-/L   ]  marine
   ... (alternating subaerial / marine below)
```

Two stacked coal seams over a marine section, capped by fire-bearing alluvium.
**Why it works:** the swamp needs *waterlogging*, not merely rain — a low-lying
site near base level that collects drainage. The peat accumulates because
decomposition is throttled where the water table is high, and burial then
promotes it to coal. The seam is thick (24 m) because the site held that
condition for a long, quiet interval.

### 2. Paleosols — **present and legible**, with climate-at-deposition tags

Deep cell (341, 463) = **world voxel (43455, 105805)**, surface 8.2 m, H 9.11 m,
21 units — a genuine **cyclothem**:

```
   6.898 m  [Sa/A/L   ]  subaerial
   0.198 m  [Sa/H/L   ]  subaerial
   0.207 m  [Ss/-/L   ]  marine
   0.133 m  [Sa/H/L·So]  organic soil horizon   <-- PALEOSOL (humid at deposition)
   0.016 m  [Ss/-/L   ]  marine
   0.257 m  [Sa/A/L·So]  organic soil horizon   <-- PALEOSOL (arid at deposition)
   0.249 m  [Sa/H/L·So]  organic soil horizon   <-- PALEOSOL (humid)
   0.028 m  [Ss/-/L   ]  marine
   0.084 m  [Sa/A/L·So]  organic soil horizon   <-- PALEOSOL
   0.058 m  [Sa/A/L·Pt]  PEAT                   <-- PALEOSOL
   ... 11 more units, soils/peats alternating with marine bands
```

A paleosol is **derived by position** — an organic unit that is not the topmost
one, i.e. something was deposited over it — so it needs no stored flag, and each
carries its own at-deposition climate on `DepUnit::tag` (`Sa/H` humid vs `Sa/A`
arid, exactly as the brief asked). The alternation with marine bands is the
sea-level cycle interleaving with soil-forming intervals: coal-measure
stratigraphy, emergent.

### 3. Charcoal bands — **present and legible**

Deep cell (62, 282) = **world voxel (-99131, 13303)**, surface 680.7 m, H 33.93 m,
39 units — **19 charcoal beds** separated by mineral units:

```
   0.081 m  [Sa/A/L   ]  subaerial
   0.040 m  [Sa/A/L·Ch]  CHARCOAL band (fire event)
   0.390 m  [Sa/A/L   ]  subaerial
   0.037 m  [Sa/A/L·Ch]  CHARCOAL band (fire event)
   0.269 m  [Sa/A/L   ]  subaerial
   0.040 m  [Sa/A/L·Ch]  CHARCOAL band (fire event)
   ... 16 more charcoal beds down the column
```

An arid upland carrying a **fire-regime record**: repeated ignition on a
sclerophyll/fire-grass community (25 % / 13 % cover at run end), each burn
leaving a thin bed. Note the tags are all `Sa/A` — the fires are on the arid
side of the map, as they should be.

### 4. Retrogressive surfaces — **present and legible**

Deep cell (146, 324) = **world voxel (-56202, 34767)**, surface **1148.0 m**,
H 1.08 m, 5 units:

```
   0.047 m  [Sa/A/L·Rt]  RETROGRESSIVE horizon (P-starved sclerophyll)
   0.667 m  [Sa/A/L·So]  organic soil horizon        <-- paleosol
   0.029 m  [Sa/A/L·Rt]  RETROGRESSIVE horizon       <-- paleosol
   0.061 m  [Sa/A/L   ]  subaerial
   0.278 m  [Sa/A/L·Rt]  RETROGRESSIVE horizon       <-- paleosol
```

community: `lichen 4 %, n-fixer 9 %, forest 5 %, sclerophyll 5 %` · soil **2.00 m
(at cap)** · N 0.007 · **available P 0.0097** · **rock-P 0.439 remaining** · age
46 epochs.

This is the Walker & Syers chronosequence, and it is **emergent, not scripted**:
a high, stable surface that erosion never rejuvenates accumulates a deep soil,
its finite rock-derived phosphorus pool draws down, available P falls below the
retrogression threshold, and the phosphorus-miserly sclerophyll inherits ground
the forest can no longer hold. The geography is right too — retrogression
concentrates on ancient stable uplands, while eroding slopes and aggrading
floodplains stay fertile because stripping and fresh sediment both deliver new
mineral P. Measured available-P across land is strongly bimodal (p05 0.0006,
p50 0.137, p95 0.504) — that spread *is* the chronosequence.

## Determinism proofs (all run, all green)

`tests/biotic.rs`, 8 tests, 15.90 s:

| test | proves |
|---|---|
| `biotic_same_seed_is_byte_identical_planes_and_records` | Repeated runs agree bit-for-bit on bedrock, alluvium, strata records, **both modifier planes**, and the biotic mass ledger. |
| `biotic_parallel_equals_scalar_byte_identical` | The data-parallel driver reproduces the scalar one to the bit (grid asserted past the rayon fork floor) — the biotic step is a pure per-cell function of frozen inputs, same contract as the S9b erosion phases. |
| `biotic_off_leaves_the_record_purely_mineral` | The abiotic path is byte-identical to pre-S10: modifier planes stay **empty** (kernels read identity `1.0`/`0.0`, and `x * 1.0 == x` exactly), ledger is 0, no community state, every unit `Mineral`. |
| `recorder_total_equals_alluvium_with_biotic_on` | The pedogenic overprint (add thickness + retag + merge down) preserves `sum(units) == H` (worst < 1e-6). |
| `mass_ledger_accounts_for_biotic_carbon` | `Δ(ΣR+ΣH) == uplift_total + biotic_total`. Buried organic matter is a genuine **external** input (photosynthesis fixes carbon from air), like uplift — tracked, not smuggled. |
| `a_different_seed_diverges_biotically` | Seed sensitivity. |
| `the_four_target_signals_reach_the_record` | Read-quality regression guard: a full A-tier run must write all four signals. |
| `biology_changes_the_landscape_it_grows_on` | Biology is an **erosion term**, not decoration: the biotic run's bedrock surface differs measurably from the abiotic one, so the lagged coupling is load-bearing. |

**Entropy**: all biotic draws are addressed by `(seed, SALT, gx, gy, epoch)` with
new salts `0x5B00_*` (distinct from pregen `0x5700_*` and deep-time `0x5900_*`).
No wall clock, no ambient randomness. `Instant` appears only in the harness,
wrapped around runs.

**Registration-order independence**: inherited, not re-derived — no pregen pass
was added (§ Pass-graph integration).

**Test-suite time**: `cargo test --workspace --release` = **381.9 s** (39 suites,
0 failed), of which the new biotic suite is 15.90 s. Delta ≈ **+16 s** on the
known ~6 min baseline. No `--ignored` gating was needed.

## Design choices where ecology.md is silent (each FLAGGED)

1. **The vanilla 7-species roster and its niche values.** The doc defines the
   niche *contract*, not a roster. Chosen as the minimal successional series that
   exercises all four signals. Real rosters are pack/registry content.
2. **The top-K co-occurrence cap is not stressed.** ecology.md's K-cap (the S8
   mixture-cap lesson) is what makes a *large* roster affordable; at roster 7 the
   full vector already fits, so the spike stores all 7. A production-scale roster
   would store `[(species, cover); K]`. The cap's value is therefore *assumed*,
   not measured, by S10.
3. **Background propagule rain.** Without it the model deadlocks — a species
   that is nowhere can never be anywhere, so only pioneers ever exist. The
   background term ("these species exist in the region") is an order of magnitude
   weaker than the neighbour term, so spread stays neighbour-driven.
4. **Dispersal enters competition as a saturating gate, not a magnitude.**
   Otherwise pioneers' propagule advantage makes them the equilibrium winners and
   succession never proceeds.
5. **Carrying capacity = the best suitability present.** Competition decides
   *who*, suitability decides *how much*; normalizing shares alone would fill
   every cell to 100 % cover however hostile.
6. **Soil formation is a pedogenic OVERPRINT, not a deposited layer** — it
   retags and thickens the material already at the surface, merging down into an
   identically-tagged predecessor. (This is both better pedology and the single
   biggest lever on record size — see the journal entry.)
7. **Soil horizons require a depositional hiatus.** Where mineral sediment
   arrives faster than a threshold, litter is diluted into the mineral unit and
   no horizon forms. Real pedology; also what keeps floodplain columns sane.
8. **Waterlogging is modelled separately from rainfall** — climate moisture plus
   bonuses for sitting near base level and for receiving upslope drainage. A wet
   mountainside sheds water and grows forest; a low flat site collects it and
   grows peat. This is what puts coal swamps on lowlands.
9. **Charcoal preserves as a discrete bed only above a residue threshold**;
   thinner residues disperse into the soil. Physically motivated and the main
   control on fire-driven record growth.
10. **Coal = buried peat ≥ 0.4 m, promoted at finalize** (burial diagenesis).
11. **Paleosol is derived by position**, not stored.
12. **Uniform initial parent-material phosphorus.** Not coupled to pregen
    provenance — a natural refinement (a granite and a basalt do not carry the
    same P).
13. **Biotic carbon is an external mass input** in the ledger.
14. **Only two erosion couplings** (weathering multiplier, hillslope
    resistance). Fluvial transport is deliberately untouched.
15. **`production_config` keeps the biotic layer OFF** pending the GO decision —
    flipping one bool turns it on. This is why the shipped `DeepField` and its
    byte-identity are unchanged by S10.
16. **Epoch calibration.** At the ratified Phanerozoic register (~500 Myr over
    200 iterations) one epoch ≈ 2.5 Myr, so a recorded "fire event" is a fire
    *regime interval*, not a single burn. Rate constants are plausible, not fit
    to Earth — the same status as S9's physics constants.

## Recommendation — **GO** (NEEDS RATIFICATION)

**Ship the biotic layer on the always-on A tier: flip `production_config`'s
`biotic` to true, and sequence the collapse-tier organic materials immediately
behind it.**

The evidence:

1. **All four target signals arrived, and they read as geology, not as tags.**
   The paleosol column is a cyclothem; the retrogression column is a Walker &
   Syers chronosequence with the right *geography*; the charcoal column is a fire
   regime on the arid side of the map. Every one answers "how did this get here?"
   in process terms — the ratified validation bar.
2. **The cost is bounded and mostly transient.** +10 s once at world creation
   (15 → 25 s), and — because `DEEP_MAX_WIDTH` already caps the grid — that
   +10 s is the same at Large as at Medium. The world *keeps* only +6 MiB.
3. **The cost is linear in cells**, with no new global dependency chain, so it
   cannot become the thing that blows up later; its share of the step shrinks as
   grids grow.
4. **Determinism survives intact**, including scalar↔parallel byte-identity, so
   the layer does not spend the determinism budget S9b showed is scarce.
5. **Biology is genuinely an earth-process term**, proven by a falsifier: the
   landscape differs with biology on. That is what ecology.md § 1 promised, and
   it is why this is worth more than annotation.

**The honest gap — and the reason this is GO-with-a-follow-on:** the record now
contains coal, but **the world does not**. Collapse maps deep units to material
classes via `deep_class`, which reads only env/energy; an organic unit currently
collapses as ordinary clastic, and there is no coal material in the roster. **A
player cannot yet mine a coal seam.** S10 proves the substrate; the payoff needs
a material-tier follow-on (coal/lignite members + the `Biofacies` → class
mapping). Recommend sequencing that as the immediate next slice.

### NEEDS RATIFICATION (the user's calls)

1. **The ritual grows 15 s → 25 s (+66 %) at Medium and Large.** Acceptable, or
   is the world-creation budget tighter than that?
2. **Turning the layer on in `production_config`** (the actual GO action) — and
   with it, a changed `DeepField` for every new world.
3. **Signal abundance is a content/aesthetic call, not a correctness one:**
   coal in 0.55 % of columns, paleosols 22.9 %, charcoal 20.2 %, retrogression
   23.5 %. Is a fifth of the world's columns carrying a fire record the texture
   we want?
4. **The roster and ~25 rate constants** ride as plausible-not-tuned (no-bandaid:
   they are the mechanism's calibration, not a patch). Same status as S9's
   physics constants.
5. **Pack-addition blast radius** (ecology.md § 5 fork 2) is now a live question,
   because biology demonstrably changes landforms: adding an organism pack would
   change terrain, which materials packs never did. **This must be decided before
   biology couples to erosion in a shipped world** — S10 is exactly the moment it
   stops being hypothetical.

## Loose ends for the implementation milestone

- **Collapse-tier organic materials** (the gap above) — the highest-value next
  slice.
- **Parent-material phosphorus from provenance**, replacing the uniform pool.
- **Vegetation → channel planform** ("vegetation invented meandering rivers")
  needs the 3e-2 C-refinement's finer corridor to have any planform to bend.
- **Individual plant placement** from the community vector by addressed hashing
  (ecology.md's derivation chain terminus) — collapse-tier, needs the community
  vector to be *persisted* or recomputed on approach; today it is dropped.
- **The community vector is not persisted.** Only strata annotations survive. If
  ecology is to drive surface vegetation, C-refinement must re-derive it.
- **Top-K cap unexercised** — revisit when the roster grows past K.
- **Flood disturbance leaves no distinct tag** (deliberate; the mineral overbank
  band is its signature) — revisit if flood-specific facies are wanted.
