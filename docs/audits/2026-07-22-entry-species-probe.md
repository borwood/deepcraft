# Entry-species probe — how far six proxies sit from the class-aggregate

*2026-07-22. Read-only measurement of the deep sim's material interface, step 2
of the DECIDED (substance, form) sequence (geology.md, 2026-07-22). No generation
code changed. Measured numbers + file:line citations in the `docs/audits/`
register; this document measures and proposes, it does not ratify.*

**HEADLINE.** For the production Medium record (seed `0x0D5EED572026`,
285 267 of 297 025 deep cells recorded, 96.0 %):

- **The SUBSTANCE error is noise-scale in vanilla.** The class-aggregate that
  DECIDED step 2 would ship moves the per-cell erosion rate by **≤ ±10 %** on the
  live mechanical agents and **0 % of cells past ±1.25×** on *any* agent. Four of
  seven lithologies are single-member classes, so their aggregate is **bit-identical
  to the proxy** by construction. Vanilla reads close to proxy.
- **The FORM error is landform-scale — 1.8× to 4.8×**, saturating the ±5× rate
  clamp on wave/eolian/frost. The record mirrors `H`, the *loose* regolith plane,
  while every clastic proxy is a *lithified* rock: the sim asks how hard loose
  river sand is and answers with sandstone, a **3–5× rate error**.
- **The form error is the bigger term by 20–50×.** This vindicates the DECIDED
  decision to skip the aggregate step and ship `f(substance, form)` once: the
  intermediate "real aggregate, still no form" world would move terrain < 10 %
  — a flip nobody wants to keep.
- **Dissolution is the axis of unbounded latent headroom**: every roster material
  is insoluble, so the dissolution rate is identically 0 in both worlds; a
  carbonate/evaporite pack member takes a cell from 0 to nonzero — a change no
  proxy can approximate.

Probe: [`crates/dc-worldgen/examples/entry_species_probe.rs`]. Reproduce with
`cargo run --release -p dc-worldgen --example entry_species_probe`.

---

## Why this exists

`deeptime::lithology::Litho::reference_material` (lithology.rs:300-310) maps six
lithology classes to six fixed rocks (MUDSTONE / SANDSTONE / CARBONACEOUS_MUDSTONE
/ PEAT / COAL / GRANITE, plus CHARCOAL), and every erosion agent reads its
resistances from those sheets (`resistance_of_material`, lithology.rs:332),
discarding the property sheets of ~30 other registered materials. Two errors are
stacked (geology.md, DECIDED 2026-07-22): a **substance** error (six arbitrary
proxies stand for whole classes) and a **form** error (the record is the *loose*
`H` plane, the proxies are *lithified* rocks). This probe measures both at zero
terrain cost — the S13 precedent: measure where the effect goes before moving
anything.

The proxy's original justification (pack-safety) expired the day it was noticed
(spines.md § A-2): under the content-set freeze a pack cannot move an existing
world's terrain, so sampling the roster is no longer forbidden. This measurement
tells us whether the substance term or the form term is the one that matters.

## Method

- **Record.** The production Medium `DeepField` (`Pregen::run`, seed
  `0x0D5EED572026`), read through its committed `strata` — the same record
  `outcrop_blend_probe` reads.
- **(a) PROXY** — the shipped path: `susceptibility_table(agent, 2.5, 5.0)` built
  from the six `reference_material` sheets, blended over each cell's near-surface
  window with `blend_susceptibility(exposed_shares(units), tab)`
  (erosion.rs:855-888, 943-970, 1371, 1511). Contrast 2.5 / cap 5.0 are
  `DeepConfig::default` — every agent's table uses them (grid.rs:260-262).
- **(b) TRUE AGGREGATE** — for each `Litho`, the **abundance-weighted class-mean**
  of `{smash, cohesion, permeability, solubility}` over the members of the class
  it resolves to (geology.md's *"abundance-weighted class-mean properties"*),
  pushed through the *same* `resistance_of_material` formula, then blended over the
  same window shares. `Litho::Basement` (no recorded tag) is aggregated over the
  igneous-intrusive class — the petrologically honest basement composition, and
  what the *recorded* intrusive basement itself resolves to (granite + diorite).
- **Reference.** The rate ratio is reported **self-consistent** (each world
  normalizes to its own `REFERENCE_LITHO = ClasticFine`, the shipped choice), so a
  pure-fine cell is 1.000 by construction and every reported deviation is a genuine
  differential change. A **fixed-reference** decomposition (both worlds on the
  proxy reference) isolates each class's own substance error — single-member
  classes then read exactly 1.000.
- **Robustness.** Selection is `fitness × abundance`; the abundance-only aggregate
  is validated against the two paleo-precip regimes the record carries (arid 0.12 /
  humid 0.60, `geology.rs::deep_precip`) — they move the class-mean by < 0.5 %
  (Part 5), so abundance-only is faithful.

---

## Part 1 — the intra-class variance table (the mithril headroom)

Per class, the abundance-weighted mean of each axis and the min..max spread across
members (max/min, or Δ when min is 0). This is the reference-free "how far could a
class's members sit apart" number — the size a single wildly-different pack member
*could* carve.

| lithology | class | proxy | members | smash | cohesion | permeability | solubility |
|---|---|---|---|---|---|---|---|
| fine | `clastic-fine` | mudstone | 2 | 4.00–4.20 (1.05×) | 0.90–0.95 (1.06×) | 0.02–0.08 (**4.00×**) | 0 |
| coarse | `clastic-coarse` | sandstone | 2 | 4.60–4.80 (1.04×) | 0.80–0.85 (1.06×) | 0.30–0.35 (1.17×) | 0 |
| soil | `organic-soil` | carbon. mudstone | **1** | — (1.00×) | — | — | 0 |
| peat | `organic-peat` | peat | **1** | — (1.00×) | — | — | 0 |
| coal | `organic-coal` | coal | **1** | — (1.00×) | — | — | 0 |
| charcoal | `organic-charcoal` | charcoal | **1** | — (1.00×) | — | — | 0 |
| basement | `igneous-intrusive` | granite | 2 | 5.50–5.60 (1.02×) | 1.00 (1.00×) | 0.02 (1.00×) | 0 |

**Findings.**

- **Four of seven classes ship a single member** — soil, peat, coal, charcoal.
  For these the class-aggregate is bit-identical to the proxy; there is *no*
  substance headroom to recover in vanilla, and the proxy is not arbitrary at all,
  it is the class.
- The three multi-member classes carry **tiny mechanical spreads**: smash within
  1.02–1.05×, cohesion within 1.06×. The members are near-clones (mudstone≈siltstone,
  sandstone≈conglomerate, granite≈diorite). The only large *relative* spread is
  fine-class permeability (4.00×, 0.02→0.08) — but at negligible absolute magnitude,
  and permeability only enters frost/ice weakly (`smash·(1−0.5·perm)`).
- **Solubility is 0 across the entire roster** — the whole dissolution axis is
  latent headroom (see Part 2).

The mithril-headroom conclusion: **vanilla's own intra-class variance is small.**
The "unique materials leave unique signatures" payoff is a statement about *packs*
(a member deliberately far from the class norm), not about the vanilla roster,
whose members were authored close together.

## Part 2 — per-agent rate ratio (TRUE class-aggregate / PROXY)

Self-consistent reference; distribution over all 285 267 recorded cells.

| agent | p01 | p10 | p50 | p90 | p99 | min | max | mean | past ±1.10× | ±1.25× | ±1.50× |
|---|---|---|---|---|---|---|---|---|---|---|---|
| abrasion | 0.936 | 0.943 | 0.995 | 1.000 | 1.000 | 0.925 | 1.000 | 0.983 | **0.00 %** | 0.00 % | 0.00 % |
| frost/ice | 0.907 | 0.917 | 0.992 | 1.000 | 1.000 | 0.873 | 1.000 | 0.974 | **4.30 %** | 0.00 % | 0.00 % |
| wave | 0.886 | 0.900 | 0.991 | 1.000 | 1.000 | 0.886 | 1.000 | 0.968 | **15.56 %** | 0.00 % | 0.00 % |
| eolian | 0.947 | 0.949 | 0.996 | 1.000 | 1.0001 | 0.947 | 1.001 | 0.984 | **0.00 %** | 0.00 % | 0.00 % |
| dissolution | — | — | — | — | — | — | — | — | *0/0 undefined* | | |

**Findings.**

- The substance swap moves the rate field by **at most ±12 %** anywhere, on any
  agent; **no cell on any agent crosses ±1.25×.** Median deviation is < 1 %.
- Direction is consistent: ratios sit *below* 1.0 (the aggregate is marginally more
  resistant), because the second member of each clastic class is slightly harder
  (conglomerate > sandstone) or the fine reference softens (siltstone < mudstone),
  which lifts everything else relative to fine.
- **Wave is the most-affected agent (15.6 % of cells past ±1.1×)** — it keys on
  `smash·cohesion`, so the fine-reference softening compounds. Even so it stays
  inside ±1.25× everywhere.
- **Dissolution is identically 0 in both worlds** (no soluble member) — the ratio
  is 0/0. This is not a null result: it is the axis where the proxy error is
  *unbounded*, because any soluble pack member moves a cell from exactly 0 to
  nonzero.

**Fixed-reference decomposition** (each class's substance error in isolation;
single-member classes read exactly 1.0000):

| class | abrasion | frost/ice | wave | eolian |
|---|---|---|---|---|
| fine | 1.051 | 1.084 | **1.110** | 1.056 |
| coarse | 0.960 | 0.934 | 1.016 | 1.057 |
| soil/peat/coal/charcoal | 1.000 | 1.000 | 1.000 | 1.000 |
| basement | 0.983 | 0.983 | 0.983 | 1.000 |

The largest single-class substance error is **fine on the wave agent, +11 %** — and
that is the whole of it. Everything else is within a few percent; the single-member
classes are exact.

## Part 3 — worst-case columns (abrasion)

The largest |ln ratio| cells are exactly the ones whose window concentrates on a
multi-member class:

| rank | deep-cell (gx,gy) | ≈ world (m) | dominant | proxy → true | ratio |
|---|---|---|---|---|---|
| #1 | (380, 254) | (49 680, −8 280) | coarse 86 % | 0.8420 → 0.7784 | **0.9245** |
| #2 | (373, 248) | (46 460, −11 040) | coarse 77 % | 0.8561 → 0.8003 | 0.9348 |
| #3 | (451, 0) | (82 340, −125 120) | basement 100 % | 0.5096 → 0.4768 | 0.9357 |
| #4 | (455, 0) | (84 180, −125 120) | basement 100 % | 0.5096 → 0.4768 | 0.9357 |

Even the single worst cell in the whole 285 k-cell record is a **7.6 %** rate
change. There is no landform-scale substance signal anywhere in the vanilla record.

## Part 4 — the FORM error (the bigger term)

The record mirrors `H`, the *loose* regolith plane; the proxies are *lithified*
rocks. These loose↔lithified pairs are **one substance in two forms** (the ratified
forms pass). The ratio is loose-form rate ÷ lithified-proxy rate (> 1 ⇒ the record's
loose material actually erodes *faster* than the proxy the sim charges it as):

| pair (loose → lithified) | abrasion | frost/ice | wave | eolian | worst |
|---|---|---|---|---|---|
| clay → mudstone | 1.58 | 1.64 | 1.81 | 1.15 | **1.81×** |
| silt → siltstone | 1.75 | 1.92 | 3.87 | 4.35 | **4.35×** |
| sand → sandstone | 1.42 | 1.96 | 4.75 | 3.79 | **4.75×** |
| gravel → conglomerate | 3.24 | 4.77 | 4.54 | 3.25 | **4.77×** |

**Findings.**

- The form error is **1.8×–4.8×**, saturating the ±5× stability clamp on
  wave/eolian/frost for the coarse pairs. It is driven by **cohesion** (loose sand
  0.05 vs sandstone 0.85 — a 17× gap; loose gravel 0.02 vs conglomerate 0.80 — 40×),
  which is exactly the axis wave and eolian key on.
- Against the substance error's ceiling of ~1.12×, **the form error is 20–50×
  larger.** The form error is the term that reaches landform scale; the substance
  term does not.
- The form error is present at *every* clastic cell, not a tail — because the veneer
  and the near-surface record are loose material everywhere the sim deposited.

## Part 5 — context sensitivity (robustness)

The abundance-only aggregate (Part 2) is faithful: re-weighting by
`fitness × abundance` under the record's two paleo-precip regimes barely moves it.

| regime | fine-mean smash | coarse-mean smash | pure-coarse ratio | pure-basement ratio |
|---|---|---|---|---|
| arid (0.12) | 4.1107 | 4.6750 | 0.9102 | 0.9321 |
| humid (0.60) | 4.1176 | 4.6750 | 0.9140 | 0.9360 |
| abundance-only | 4.1176 | 4.6750 | — | — |

The arid/humid spread is < 0.5 % — the clastic members' windows overlap so heavily
that formation context does not meaningfully change which members the class averages.
The abundance-only class-mean is the honest aggregate.

---

## What this measurement does and does not justify

- **It does NOT justify shipping the aggregate step now.** The substance error is
  noise-scale in vanilla (≤ ±12 %, 0 % of cells past ±1.25×, 4 of 7 classes exact).
  The intermediate "real aggregate, still no form" world would re-baseline every
  golden and cost a terrain-flip trip past the user's eye for a change smaller than
  the erosion knobs' own slack. This is direct measured support for the DECIDED
  rejection of aggregate-then-form.
- **It DOES justify going straight to `f(substance, form)`.** The form error is
  1.8–4.8×, the term that actually carves. Fixing substance without form buys almost
  nothing; fixing form is where the landform signal is.
- **The rework's justification does not rest on vanilla's substance headroom** —
  that headroom is small. It rests on (1) the **measured form error** (this
  document), and (2) the **pack-signature argument** — unique pack materials
  leaving unique erosional signatures — which the user has **already ratified** as
  sufficient direction (things-that-will-happen.md, the mithril framing,
  2026-07-22). The dissolution axis (Part 2) is the sharpest instance: infinite
  latent headroom sitting behind a uniform zero.

So the recorder's entry-species fork proceeds as one decision — `Litho` becomes
`(substance mixture, form)` — with **form** as the load-bearing half, not substance.

## Register

- **Probe:** `crates/dc-worldgen/examples/entry_species_probe.rs` (deterministic,
  seeded, no wall clock; read-only over generation code).
- **Seam under measurement:** `Litho::reference_material` (lithology.rs:300),
  `resistance_of_material` (lithology.rs:332), `susceptibility_table`
  (lithology.rs:618), `exposed_shares` / `blend_susceptibility` (lithology.rs:538,
  599). Selection: `GeologySet::select` (dc-core geology.rs:381), `deep_class`
  (dc-worldgen geology.rs:348), `deep_precip` (geology.rs:289, mirrored in probe).
- **Property sheets:** dc-core `materials/mod.rs:237-538` (the `REGISTRY`).
- **Related audits:** `2026-07-22-seam-inventory.md` (the seam and its 30-material
  blast radius), `2026-07-22-deeptime-vector-audit.md` (form-awareness is zero
  across all 24 vectors), `2026-07-22-threshold-quantization-audit.md` (the A1
  share-blend this probe reuses).
