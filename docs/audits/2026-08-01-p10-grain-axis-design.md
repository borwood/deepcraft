# P10 — THE GRAIN-SIZE CONTINUUM: design pass (OPTIONS, none picked)

> **USER LEANING RECORDED (2026-08-01 — not a ruling; U1 stays open until FS-A is
> actually proposed):** *"'loose' gets an additional state (what grain is THIS loose
> block of this material id)."* I.e. the provenance-keeping encoding: grain size
> arrives as a **state on the loose FORM of the source identity**, never as replacement
> ladder materials — which also points C4's resolution (the standalone
> SAND/GRAVEL/CLAY/SCREE identities read as the scaffolding-era encoding) and forbids
> reading FS-A as identity-erasure. The full U1 ruling happens against this doc when
> FS-A is proposed. *(Mutable header; the body below is the dated design record.)*

**Arc anchor:** `ROADMAP.md:413-442` § Sequenced *"THE GRAIN-SIZE CONTINUUM"* — **SEQUENCED
2026-08-01 (user, "option 2 — I just don't want to lose track of anything needing done")**.
Graph row: `docs/dependency-graph.md:78` (P10).

**Produced by a read-only design-pass agent. It DECIDES NOTHING.** Every option below is
priced and its trade-offs named; the picks are the user's or the integrator's per § 6.
Anything the agent originated is marked **(assistant-proposed)**. **No cargo was invoked** —
every number is arithmetic over already-measured quantities, cited to source, or a layout
computation done by hand from the field list and flagged as such.

**This targets the MEMBER-GRADE record**, per the same-day P11 rulings
(`docs/audits/2026-08-01-members-into-history-design.md` header): the record names
**`MaterialId`**, representation **A-CLEAN** (no class view survives storage or physics),
transport planes **SPARSE DAY ONE**. Nothing here is designed against the 7-entry `Litho`
roster, which P11 dissolves.

**Status legend:** **DECIDED/RATIFIED** (user) · **BUILT** (code exists, verified here) ·
**PROPOSED** (recorded, not decided) · ⚠ **FLAGGED** (could not verify / stale input).

**Immutable body, mutable header** (CLAUDE.md read-first item 5). Read at commit `bea5344`
+ the P11 audit. Anything that later refutes or re-scopes this file gets a banner **here**,
stamped by the author of the correction.

**Read with:** `docs/audits/2026-08-01-members-into-history-design.md` (§§ 3.1–3.6 are the
direct parents of § 3 below) · `docs/audits/2026-07-29-fluvial-record-terms-priors.md`
(§ 1.5/§ 1.6 the proxy hazards, § 3 the literature bands, § 4 the cost frame) ·
`docs/design/material-behavior.md` § 13 (the transport family) / § 3 (the edge catalog) ·
`docs/design/materials.md` § *forms design pass* ruling 2 + § DECIDED 2026-07-22 ·
`docs/design/refinement.md` § 5 (the three Laws) · `docs/design/flow.md` § 11.1 (WINDOW) ·
`docs/design/stubs.md` #21 / #23 / #25 / #26 · `docs/spikes/S19-flow-record-cost-results.md`.

---

## Contents
0. The five findings that reorganise the question
1. DQ1 — what IS the size state, and where does it live
2. DQ2 — the evolution operators
3. DQ3 — the cost envelope
4. DQ4 — literature anchors and the P2 gate
5. DQ5 — the first BUILD slice candidate
6. NEEDS RATIFICATION — user-owned vs integrator-settleable
7. Contradictions and priors found in passing (reported, not fixed)
8. What this audit could not verify

---

## 0. The five findings that reorganise the question

**F1. The material registry ALREADY spans the full Wentworth scale, and the deep tier is
about to be able to name it.** `crates/dc-core/src/materials/mod.rs:362-690` — verified by
extracting every sheet:

| material | `grain_size_mm` | φ = −log₂ d | Wentworth class |
|---|---:|---:|---|
| scree | 100.0 | **−6.64** | boulder |
| gravel | 20.0 | **−4.32** | pebble |
| conglomerate | 8.0 | −3.00 | pebble |
| granite | 3.0 | −1.58 | granule |
| sand | 0.5 | **+1.00** | medium sand |
| sandstone | 0.3 | +1.74 | medium sand |
| silt | 0.02 | **+5.64** | coarse silt |
| siltstone | 0.02 | +5.64 | coarse silt |
| clay | 0.002 | **+8.97** | clay |
| mudstone | 0.004 | +7.97 | clay |

*(The full 26-row extraction is in § 3.0. The bolded five — scree, gravel, sand, silt, clay
— are a clean 5-rung φ ladder spanning boulder→clay, i.e. **the whole span the priors audit
§ 1.6 said was missing**.)* The priors audit's *"nothing in the clastic classes is gravel"*
was a statement about the **`Litho` roster**, not about the registry — and P11 slice 1
replaces `DepUnit::species: Litho` with `MaterialId`, at **zero bytes**. **P11 hands P10 a
Wentworth alphabet for free.**

**F2. That ladder is BUILT AND NOTHING IN WORLDGEN CALLS IT.** `MaterialId::SAND / GRAVEL /
CLAY / SILT / SCREE` appear **nowhere** in `crates/` outside their own registry entries,
one `dc-api` host test, a meshing sentinel buffer, and one probe table
(`crates/dc-worldgen/examples/entry_species_probe.rs:455-459`). The deep tier emits only
*lithified* rock, because `Litho::reference_material()`
(`crates/dc-worldgen/src/deeptime/lithology.rs:301-311`) maps every class to a lithified
sheet. This is a `spines.md` § 3 *"built, and nothing calls it"* row that is not listed
(§ 7, C3).

**F3. The transition graph already declares material-CHANGE edges, so comminution is
expressible today without any new machinery.** `is_declared_edge`
(`crates/dc-worldgen/src/deeptime/inventory.rs:212-214`) is
`from != to && !(both Void)` over `(MaterialId, InvForm)` pairs — so
`(GRANITE, Structure) → (SAND, Loose)` and `(GRAVEL, Loose) → (SAND, Loose)` are **both
already legal declared edges**, and a `Fact` can name them. `Agent::Abrasion` is a live
agent with a per-material susceptibility (`lithology.rs:90-127`, `:342`). **A release
spectrum and an abrasion operator both have a shipped home in the edge/agent shape.**

**F4. `DepUnit` has zero padding and align 8, so the record's grain axis is priced in
8-byte quanta.** `tag` 5 + `thickness_m: f64` 8 + `unconformity` 1 + `chapter` 1 +
`species` 1 = 16 B exactly (`stubs.md:914-919`; P11 § 3.1 re-derives it). Adding **one**
byte rounds to 24 → **+42.23 MiB** at 5.54 M live units. Adding **eight** bytes costs the
same 42.23 MiB. **So an 8-bin φ histogram costs exactly what one quantized byte costs**, and
the real question is binary: *does the record grow at all?* Neither the priors audit § 4.5
nor P11 § 3.2 states this quantum explicitly, and it dominates § 3's table.

**F5. WINDOW aggregation destroys the exact signature P10 exists to buy — unless
fining-upward is an EXPRESSION, not a recording.** A φ histogram is mass-additive (verified,
§ 2.3), so `Sum` over a 25-epoch chapter is well-defined. But `Sum` is **order-blind**, and
*fining-upward* is an **ordering within the window** — the same hazard `flow.md` § 5 limit 2
names for turbidites and `ideas.md`'s WINDOW sketch lists as a first forced customer. The
resolution that costs nothing (**assistant-proposed**, § 2.3): a unit records *how much of
each φ class it contains*; the refinement operator lays that distribution out **in settling
order within the unit's thickness**. That is Law-1 legal (expression varies with a recorded
continuous quantity) and Law-3 conserving. **Fining-upward is a layout of a recorded
distribution, not a recorded sequence.** If this reading is accepted, P10 is **not** blocked
on WINDOW.

---

## 1. DQ1 — what IS the size state, and where does it live

The question has three separable halves, and the corpus has been running them together:

- **(a) the RELEASE SPECTRUM** — a property of a *material definition*: what does weathered
  granite shed?
- **(b) the IN-TRANSIT STATE** — what the load carries and the transport pass evolves.
- **(c) the RECORDED STATE** — what a `DepUnit` stores at deposition.

They can be answered with **different representations**, and the cheapest answers differ.

### 1.1 The load-bearing fork: is grain an IDENTITY or an AXIS?

Everything below hangs on this, and it is a **user question** because a user ruling already
answers half of it (§ 7, C1):

> **materials.md § forms design pass, ruling 2 (DECIDED 2026-07-21, user):** *"Sand is a
> FORM of an existing clastic material. No new 'sand' identity: sand is the loose form of
> the clastic-coarse member — 'one material has one property [sheet]'; **within-identity
> grain-size gradation is not modelled** (revisit later if a wall appears; user: 'go with
> forms. maybe it gets revisited at a later time but go with form')."*

P10's WHAT — *"source rocks release a size **distribution** (not their sheet's single
`grain_size_mm`)"* — **is** within-identity grain-size gradation. The ruling anticipated
being revisited; this is the wall. **It is not contested here; it is surfaced (§ 6.1).**

### 1.2 Options for the size state

| # | option | what it is | record bytes | plane multiplier | expresses fining? | expresses bimodality? | placers? |
|---|---|---|---:|---:|---|---|---|
| **G1** | **implied grain** | no new state; grain = `props().grain_size_mm` of the recorded `MaterialId` | **0** | ×1 | ✗ | across species only | via `settle_energy`, as today |
| **G2** | **grain IS identity + comminution edges** | the φ ladder is a set of registered materials; abrasion is a declared `(GRAVEL,Loose)→(SAND,Loose)` edge | **0** | ×1 | ✓ (identity migrates) | ✓ (a cell holds several rungs) | ✓ |
| **G3** | **product space (provenance × φ)** | the load/record plane is keyed by `(MaterialId, φ_k)` | +8 B step | **×K** | ✓ | ✓ | ✓ |
| **G4** | **moments per material** | `(mass, mass·φ)` or `(mass, mass·φ, mass·φ²)` per `(cell, MaterialId)` | +8 B step | ×2 or ×3 | ✓ | ✗ *within* a material | ✓ |
| **G5** | **dual** | histogram in the transient load, moments in the record | +8 B step | ×K transient, ×2–3 recorded | ✓ | transient only | ✓ |
| **G6** | **grain decoupled from species** | one φ histogram per cell/face **total**, species shares separately | +8 B step | ×1 + K | ✓ | ✓ | ✗ **loses the size↔density correlation** |

**Reading the table.**

- **G1 is the null option and it is not empty.** With P11's `MaterialId` record and the
  F1 ladder, "grain" already varies across the roster by 15 φ. What G1 cannot do is
  *evolve* — a grain never gets smaller, so Sternberg has nowhere to land, and the ROADMAP
  WHAT is unmet. G1 is the honest **baseline to beat**, and it is what P11 alone delivers.
- **G2 honours the user's forms ruling exactly** — no within-identity gradation; grain
  gradation is *between* identities, which is what "one material has one property sheet"
  means. It costs **zero bytes anywhere** (F4's 42.23 MiB step is never paid), reuses the
  declared-edge authority (F3), and makes the unconsumed ladder consumed (F2). Its
  weaknesses are real and must be stated: **(i) provenance loss** — granite gravel abrading
  to `SAND` is no longer distinguishable from sandstone-derived sand, which discards exactly
  the 65 % provenance effect journal/0112 measured; **(ii) registry pressure** — a
  provenance-preserving ladder needs `granitic-sand`, `basaltic-sand`, … i.e. the product
  space smuggled into the registry, and STUB #21's `EdgeId` packing **hard-caps the registry
  at 51 materials** (compile-asserted, `inventory.rs:250-255`); **(iii)** it makes abrasion
  a *discrete* jump between rungs, so Sternberg's exponential becomes a hop-probability.
- **G3 is the faithful option and the expensive one.** ×K on four `n × MaterialId` planes
  (§ 3.2) and the +8 B record step. It is the only option that carries provenance **and**
  size independently and continuously.
- **G4 is the cheap faithful option (assistant-proposed as a named candidate).** Carrying
  `Σm`, `Σm·φ` (and optionally `Σm·φ²`) per `(cell, material)` is **exactly mass-additive**
  (§ 2.3), gives a mean φ and a method-of-moments sorting σ at read time, and costs ×2–3
  instead of ×K. **What it cannot represent is a bimodal distribution within one material**
  — and the literature (§ 4) says the **gravel–sand gap is a first-order feature**, so this
  is a real loss, not a rounding one. It is partially compensated: bimodality *across*
  materials survives, and with the F1 ladder that may be where bimodality actually lives.
- **G6 is the trap.** Decoupling grain from species is the cheapest non-trivial option and
  it destroys placers — a placer *is* the correlation between high density and the coarse
  fraction (`geology.rs:449-457`'s own doc comment: *"a dense ore grain's threshold lands in
  the coarse band despite its small size, so it settles with the gravel — which is what a
  placer is"*). Listed so it is rejected explicitly rather than rediscovered.

### 1.3 Where the RELEASE SPECTRUM lives

A release spectrum answers *"weathered granite sheds what?"*. Four homes, all buildable:

| # | home | shape | cost | fit with the corpus |
|---|---|---|---|---|
| **R1** | a fixed-K array field on `MaterialProps` | `release_phi: [f32; K]` | K×4 B × 26 = **2.6 KiB at K=8**, static | simplest; but a fixed-N field on a sheet is `ShareVec`'s problem again (P11 § 3.6) and caps K at compile time |
| **R2** | **a declared EDGE with products** — `(GRANITE, Structure) → {(SAND,Loose) 0.4, (CLAY,Loose) 0.35, (GRAVEL,Loose) 0.25}` | a per-edge product table | one table, sparse | **the north-star shape**: material-behavior § 3's edge catalog is *"the catalog of material processes"*, edges are already declared and already material-changing (F3), and `materials.md` § DECIDED 2026-07-22 puts transformations **on the material definition as the ONE authority both sims consult** |
| **R3** | a parametric pair on the sheet | `release_median_phi`, `release_sigma_phi` | 8 B × 26 | continuous, K-free, calibratable against published grus/saprolite distributions; but a lognormal cannot express a bimodal release (granite sheds quartz sand **and** clay — genuinely bimodal) |
| **R4** | derive from `grain_size_mm` + one new spread field | `release_sigma_phi` only | 4 B × 26 | cheapest; but ⚠ it **reuses `grain_size_mm` as a release median**, and that field has three shipped consumers reading it as something else (§ 7, C6) |

**R2 is the one the north star points at** and it is the only one that composes with G2 for
free (a comminution edge and a release edge are the same kind of object). **R3/R4 compose
with G3/G4.** R1 composes with G3 and inherits `ShareVec`'s fixed-N problem.

⚠ **Note the bimodality argument cuts against R3/R4 the same way it cuts against G4** —
and it is the *same* physical fact appearing twice. Whichever option is picked, the corpus
should record once whether **bimodality is a requirement or a nicety**; it decides four
sub-picks at a stroke.

### 1.4 How grain composes with the per-face composition term

The record-terms ruling (2026-08-01) records **load composition per face**, per `MaterialId`
after P11's re-grade, and holds **ruling 3 (face-vs-unit)** pending P11 slice 2. Three
composition shapes, priced in § 3.3:

| # | shape | what a reader gets | cost at 26 materials |
|---|---|---|---|
| **C-a** | **per-face TOTAL φ histogram**, beside the existing per-material shares | "this face carried 3 m of sediment, of which 40 % was sand-grade, and it was 30 % granitic" — **the two facts do not join** | K bytes/loaded entry: **4.90 MiB at K=8** |
| **C-b** | **per-material-per-face** (the product) | the full joint distribution | K × 26 bytes/loaded entry: **99.2 MiB at K=8**, 25.7 at K=2 |
| **C-c** | **per-material moments per face** (G4) | mean φ and σ per material per face | 2–3 × 26 bytes: **25.7 / 38.0 MiB** |

**"Per parcel" is not available and should be struck from the vocabulary.** There is no
parcel in the record: the load is a *pass-transient multiset*
(`material-behavior.md` § 13.3, and `erosion.rs:1460-1466` — `qs_sp` is rewritten in place
by `exchange_cell` and *"a cell's slice is never read after it is spent"*). A parcel exists
only inside one epoch's downstream walk. **Per-parcel grain state is a gen-time concept
with zero residency and no record shape** — which is exactly what makes § 2.1's abrasion
answer cheap.

---

## 2. DQ2 — the evolution operators

### 2.1 Abrasion — and the finding that the missing path length does not matter

**The problem as stated in the brief:** Sternberg is `D(x) = D₀ e^{−αx}` and the record has
no path length.

**The finding (assistant-proposed): for an exponential law, the state IS the integral, so
no path length is needed anywhere.** The transport pass is a strictly downstream-ordered
walk along the pinned receiver (`material-behavior.md` § 13.1; `erosion.rs`'s sorted path).
Each hop is exactly one deep cell — `DEEP_CELL_M = 460.0`
(`crates/dc-worldgen/src/deeptime/field.rs:42`). An exponential decays multiplicatively, so:

```
per-hop factor = exp(−α · 0.460 km)
α = 0.015 km⁻¹  →  0.993124 per hop
```

Applying that factor once per hop to the carried grain state reproduces `e^{−αx}` **exactly**
for any path, without storing x anywhere. The grain state itself is the memory of how far it
has come — which is what a real pebble is.

⚠ **One correction, and the fix already ships.** The hop is **not** a constant 460 m: MFD
splits flow across 8 neighbours including diagonals, so a diagonal hop is 460·√2 ≈ 650 m.
`erosion.rs:161` already holds `MFD_DIST: [f64; 8] = [√2, 1, √2, 1, 1, √2, 1, √2]` in cell
units for exactly this reason (the slope denominator). So the exact per-face factor is
`exp(−α · 0.460 · MFD_DIST[d])` and **the machinery to make the exponential exact under
multi-flow already exists.** Nothing here needs a flux-weighted approximation.

**The scale arithmetic, which also bounds K (§ 1.2):**

| α (km⁻¹) | e-folding | in 460 m cells | φ-halving length | in cells |
|---:|---:|---:|---:|---:|
| 0.010 | 100 km | 217 | 69.3 km | 151 |
| **0.015** | **66.7 km** | **145** | **46.2 km** | **100** |
| 0.020 | 50 km | 109 | 34.7 km | 75 |

The deep grid is 545 cells ≈ **250.7 km** wide (545 × 0.46 km; S19 § 1 for the 545²).
A full-width traverse at α = 0.015 is **250.7 / 46.2 ≈ 5.4 φ halvings**. **So abrasion alone
can move a grain about five φ classes across the whole world** — which is the arithmetic
that says K = 8 is generous, K = 4–5 spans what the world can actually produce, and K = 2
throws most of it away. *(This is a statement about the deep tier; a refinement operator is
never judged on a whole-network fining gradient — priors § 3.3 makes the same point.)*

**Does the state survive deposition and re-entrainment?** This is the one requirement.

- Under **G2** (grain ≡ identity): yes, automatically — the deposited material *is* the
  rung, and re-entrainment reads its sheet. **Zero extra state.**
- Under **G3/G4/G5**: only if the record carries the grain state (the +8 B step, F4).
  Otherwise a grain resets to its material's nominal size every time it is set down, and
  abrasion cannot accumulate across epochs. ⚠ **This is a hard coupling and it should be
  ratified as one: choosing G3/G4 without paying the record step gives an abrasion operator
  that silently forgets.**

**The proxies, ranked and stated honestly** (needed only if a *non-exponential* attrition
law is wanted, or for reporting):

| # | proxy | availability | verdict |
|---|---|---|---|
| 1 | **per-hop Δx = `DEEP_CELL_M`** | free, exact | **exact for an exponential law; needs no proxy at all** |
| 2 | **hop count carried on the parcel** | free (a per-parcel counter inside one epoch's walk) | redundant with (1) for Sternberg; required for a law with memory of x |
| 3 | **flow length field** — accumulate downstream distance in the same sweep as `area` | new `n × f32` plane = **1.13 MiB**; no such field exists today (grepped: no `flow_length` / `dist_to_outlet` in `deeptime/`) | honest and reusable, but it measures *distance to outlet*, not *distance travelled by this grain* |
| 4 | **Hack's law from `area`** — `L ≈ 1.4 · A^0.6` (A km², L km) | free, `area` is already a field | a *basin-length* estimate, not a path length. ⚠ coefficient/exponent quoted from memory, **not re-verified** (§ 8) |
| 5 | epoch/chapter count as a time proxy | free | **reject** — Sternberg is in distance, not time |

**What α hangs on.** The priors audit § 1.5 states *"Sternberg-style abrasion has **no
property to hang on** today"*. That is narrowly true (there is no attrition coefficient) and
**too strong as written**: `LithoResistance::abrasion` **is** `resistance(DamageType::Smash)`
read straight (`lithology.rs:340-343`), it is derived **per `MaterialId`**
(`resistance_of_material`), and `Agent::Abrasion` is a live agent. So `α(material) = α_ref ·
(smash_ref / smash_material)` is a defensible derivation with a shipped input. Whether that
mapping is *right* is a calibration question (§ 4), not an availability one. (§ 7, C2.)

### 2.2 Sorting — the falling ceiling, extended

`material-behavior.md` § 13.5 is user-endorsed (*"sound and promising… refine on build"*):
capacity is the mass limit, competence is the size/density ceiling, and *"sorting is the
falling ceiling; we write the ceiling, not the sort."* It is **BUILT** as a per-species
ceiling: `competence_ceiling(cap, k_transport)` (`erosion.rs:458`) against
`settle_energy = sqrt(grain_mm × SG)` (`geology.rs:455-457`), with the order precomputed
once per run into `w_settle: [f64; SPECIES]` / `ws_order` (`erosion.rs:1443-1444`).

**With a grain axis, the ceiling becomes a threshold over `(material, φ)` instead of over
`material`. Two consequences:**

1. **The precomputed order survives — but ONLY if the φ bins are fixed.** The code comment
   at `erosion.rs:1438-1443` justifies precomputation on the grounds that *"the ordering is
   a property of the materials, not of any particular load"*. That stays true for a **fixed**
   bin ladder: `settle_energy(material, φ_k)` is a static S×K table, so `ws_order` widens to
   `[usize; S*K]` and nothing becomes load-dependent. **With adaptive/per-record bins it
   becomes load-dependent and the inner loop gains a sort.** This is an independent argument
   for fixed bins, alongside the additivity one (§ 2.3).
2. **Competence gains real teeth.** Today the roster's clastic span is φ 1.7…8.0 (priors
   § 1.6) — the ceiling has almost nothing to discriminate between. With the F1 ladder it
   spans φ −6.6…+9.0, and the same shipped mechanism starts producing gravel-in-steep-reaches
   / mud-in-the-lake for free. **No new sorting operator is needed; the existing one gets a
   wider alphabet.** That is the strongest cheap-win argument in this document.

**Stub #23 gets worse before it gets better, and that should be said out loud.** `peat` at
5 mm / 400 kg/m³ reads `settle_energy` 1.414 — heavier than sandstone. Giving materials an
explicit release spectrum makes that number *explicit* rather than implied, so it will be
visible in more places. P10 does **not** fix it (the heir is still fluid identity,
`flow.md` § 2.5, unbuilt) but P10 **does** decouple size from density in the state, which is
half of what a Stokes/drag form needs. Recommend the stub entry gets a pointer to P10 as a
*partial enabler, not the heir*.

### 2.3 WINDOW interaction — the additivity claim, VERIFIED not inherited

**The brief asks for verification rather than inheritance. Here it is.**

| quantity | additive under `Sum` over epochs? | why |
|---|---|---|
| **mass in φ-class k** (a histogram bin) | **✓ YES** | mass is extensive; Σ over epochs of masses in a fixed bin is the total mass in that bin |
| **`Σ m`** (total) | ✓ | same |
| **`Σ m·φ`** (first raw moment) | **✓ YES** | a mass-weighted sum is a sum |
| **`Σ m·φ²`** (second raw moment) | **✓ YES** | same |
| **mean φ** = `Σmφ / Σm` | **✗ NO** — but **derivable** at read time from two additive quantities | a ratio of additive quantities is not itself additive |
| **method-of-moments σ** = `√(Σmφ²/Σm − mean²)` | ✗ NO — **derivable** from three additive quantities | same |
| **median φ / D₅₀** | **✗ NO and NOT derivable from moments** | a percentile of a mixture is not a function of the mixtures' percentiles |
| **Folk & Ward sorting** `(φ₈₄−φ₁₆)/4 + (φ₉₅−φ₅)/6.6` | **✗ NO** — derivable from a **histogram** by interpolation, not from moments | percentile-based |

**Three qualifications the claim needs and does not usually carry:**

1. **Additivity requires FIXED, SHARED bin boundaries.** Two histograms with different bin
   edges cannot be added. This is a precondition, not a detail, and it is the same
   conclusion § 2.2 reaches from the sort order.
2. **Mass-additive ≠ meaning-preserving.** `Sum` over a 25-epoch chapter gives the chapter's
   *aggregate* size distribution and **erases the within-chapter ordering** — which is
   `flow.md` § 5 limit 2 exactly (*"a turbidite averages away inside a 25-epoch bucket while
   its graded bed is the whole signature"*) and is listed in `ideas.md`'s WINDOW sketch as a
   first forced customer.
3. **Therefore: fining-upward is not recoverable from an additive record, and does not need
   to be** (F5, **assistant-proposed**). A `DepUnit` says *"this bed is X m thick and
   contains these proportions of each φ class"*; the refinement operator lays them out in
   **settling order within the thickness**. Under `refinement.md` § 5: **Law 1** holds
   (expression varies only with recorded continuous quantities), **Law 2** holds (no
   categorical tag is painted), **Law 3** holds (the layout integrates to the recorded
   totals). **If this is accepted, P10 needs nothing from WINDOW and is not blocked by it.**
   If it is *not* accepted — if the user wants a recorded sequence — then P10 acquires a
   hard WINDOW dependency and should be re-sequenced behind it.

**A closed-vocabulary note for the WINDOW pass** (`ideas.md`'s Sum/Max/Mean/Last sketch): a
φ histogram wants **`Sum`** and nothing else; the moment triple wants **`Sum`** on all three
components **jointly** — which is a small but real requirement, because aggregating `Σmφ`
with `Sum` while aggregating `Σm` with `Mean` silently produces a meaningless mean φ.
**Component-coupled aggregators are a vocabulary requirement P10 discovers**; filing it as
input to the WINDOW design pass, not deciding it.

---

## 3. DQ3 — the cost envelope

### 3.0 The frame, and which numbers are stale

| quantity | value | source | staleness |
|---|---:|---|---|
| deep cells `n` | **297,025** (545², 460 m) | S19 § 1 | stable |
| `DEEP_CELL_M` | **460.0 m** | `field.rs:42` | stable |
| live `DepUnit`s | **5,535,837** | S19 § 3(a) | ⚠ **predates journal/0111 (~1000× denudation) and /0112 (creep)** — P11 § 3.0 flags this as its weakest number; every per-unit figure here inherits it |
| `size_of::<DepUnit>()` | **16 B, zero padding, align 8** | `stubs.md:914-919`; P11 § 3.1 | field arithmetic, not `offset_of!` |
| loaded `FluxEntry`s | **494,296** (19.08 %) | journal/0096:296-298 | ⚠ lower bound (FLOW (a) populated vertical faces) |
| `DeepField` w/o record | **108.55 MiB** | journal/0096:266-267 | 2026-07-25 |
| flow record | **40.66 MiB** | journal/0096:255-258 | 2026-07-25 |
| four `n × SPECIES` f64 planes today (S=7) | **63.45 MiB** gen-time | P11 § 3.3 | derived |

**One plane at (S=1, K=1) is `297,025 × 8 B = 2.2661 MiB`; four planes are `9.0645 × S × K`
MiB.** (Check: 9.0645 × 7 = 63.45 ✓ reproduces P11 § 3.3.)

### 3.1 The record side — the 8-byte quantum (F4)

| record option | added bytes | `size_of::<DepUnit>()` | added residency @ 5.54 M |
|---|---:|---:|---:|
| **G1 / G2** — grain implied by `MaterialId` | **0** | 16 | **0 MiB** |
| one quantized mean-φ byte | 1 | **24** | **42.23 MiB** (+38.9 % of the field) |
| `(mean φ, σ)` two bytes | 2 | 24 | **42.23 MiB** |
| **K = 8 φ bins, u8 each** | 8 | 24 | **42.23 MiB** |
| K = 8 bins + a mover byte (stubs #25, `earth-processes.md`'s *"agent axis"*) | 9 | 32 | 84.47 MiB |
| K = 16 bins, u8 | 16 | 32 | **84.47 MiB** |

Arithmetic: 5,535,837 × 8 = 44,286,696 B = **42.23 MiB**; × 16 = **84.47 MiB**.

> **The shape of this table is the finding.** Between 1 and 8 added bytes the cost is
> **flat**. So *"can we get away with two bits?"* is the wrong question — if the record grows
> at all, it should carry the full 8 bytes. And STUB #25's mover byte + the `earth-processes`
> agent axis are the natural co-riders: **they are free once the step is paid, and they
> double the cost if paid separately** (§ 7, C7).

**Two ways to avoid the step entirely (assistant-proposed, both need their own ratification):**

| lever | frees | risk |
|---|---:|---|
| `thickness_m: f64 → f32` | **4 B** | f32 gives ~7 significant digits; unit thicknesses run cm→km. ⚠ **It is the mass authority**, so Law-3 / per-species closure would have to be re-proven — the same caveat P11 ruling 3 attaches to a store-f32 split of the budget planes |
| bit-pack `DepTag` (5 fieldless enums, 5 B) | **≥3 B** | `DepTag` is read everywhere in `litho_of_tag` / `energy_band` / the recorder; a packing is a representation change across many sites |

### 3.2 The transport planes — grain multiplies the species axis

P11 ruling 3: **SPARSE DAY ONE**. Dense is priced first only to show what sparse is avoiding.

**Dense** (`9.0645 × S × K` MiB, four planes, gen-time working set):

| S ↓ / K → | 1 | 2 | 4 | 8 |
|---|---:|---:|---:|---:|
| 7 (`Litho`, today) | **63.45** | 126.90 | 253.81 | 507.61 |
| 14 (vanilla members) | 126.90 | 253.81 | 507.61 | 1015.22 |
| **26 (`MATERIAL_COUNT`)** | **235.68** | 471.35 | 942.71 | **1885.42** |
| 51 (STUB #21 ceiling) | 462.29 | 924.57 | 1849.14 | 3698.28 |

**Dense × K = 8 at member grade is 1.9 GiB of gen-time working set.** That is the number
that makes the sparse ruling non-negotiable for P10, not merely preferred.

**Sparse (CSR over cells, `u8` id array + `K × f64` payload + `u32` row offsets):**

```
per plane = n·p·(1 + 8K) bytes  +  n·4 bytes index
n = 297,025 · index = 1.133 MiB/plane · p = mean distinct materials present per cell
```

⚠ **`p` is UNMEASURED.** P11 § 3.4 flags the same gap (*"the load's true sparsity per face
is unmeasured — the one cheap measurement that would settle the whole cost family"*). Both
audits now want it; it is one probe run.

Four planes, MiB:

| K → | 1 | 2 | **3 (moments)** | 4 | 8 |
|---|---:|---:|---:|---:|---:|
| **p = 3** | **35.13** | 62.32 | **89.51** | 116.70 | 225.48 |
| **p = 5** | **55.52** | 100.84 | 146.16 | 191.49 | 372.78 |
| p = 8 | 86.10 | 158.64 | 231.17 | 303.70 | 593.83 |

Arithmetic (p = 3, K = 4): 297,025 × 3 × 33 = 29,405,475 B = 28.043 MiB + 1.133 index =
29.176 MiB/plane × 4 = **116.70 MiB**.

**Read against today's 63.45 MiB:** at p = 3, sparse **K = 1 is cheaper than today**
(35.13), **K = 2 is roughly at par** (62.32), and **K = 4 doubles it**. So the honest
statement is: *the sparse re-grade P11 is already buying pays for about one bit of grain
axis, and everything beyond K = 2 is new spend.*

**G4 (moments) sits at K = 3 = 89.51 MiB** — cheaper than a K = 4 histogram, more expressive
along the mean/σ axes, blind to bimodality (§ 1.2).

### 3.3 The face composition term, crossed with grain

Parallel CSR over the **494,296 loaded entries**, `u8` shares (the shape P11 § 3.4 ranks
first at every roster width), + 1.133 MiB CSR index:

| shape | K=1 | K=2 | K=4 | K=8 |
|---|---:|---:|---:|---:|
| **C-a** per-face **total** φ histogram (no species join) | 1.60 | 2.07 | 3.02 | **4.90** |
| **C-b** per-material-per-face at 26 materials | **13.39** | 25.65 | 50.16 | **99.18** |
| C-b at 14 members | 7.73 | 14.34 | 27.54 | 53.96 |
| **C-c** per-material moments at 26 (K≡2 or 3) | — | 25.65 | 38.02 (K=3) | — |

Arithmetic: 494,296 × 26 = 12,851,696 B = 12.256 MiB (×2 = 24.512, ×4 = 49.02, ×8 = 98.05),
plus 1.133 index. C-a: 494,296 × 8 = 3,954,368 B = 3.771 + 1.133 = 4.90 MiB.

**Against the 40.66 MiB flow record and the 108.55 MiB field:** C-a at K = 8 is **+12 % of
the record, +4.5 % of the field** — genuinely cheap. C-b at K = 8 is **2.44× the entire flow
record**. **The gap between C-a and C-b is a factor of 20, and what it buys is the joint
distribution** — i.e. exactly the placer/provenance correlation § 1.2 says G6 must not lose.
⚠ **A sparse `(material, φ, share)` triple list would decouple this from both widths**
(assistant-proposed) and is the shape to price once `p` is measured.

### 3.4 What is free

| consumer | change | Δ |
|---|---|---:|
| `settle_energy` / `ws_order` | widens to an S×K static table (§ 2.2), **if bins are fixed** | 0 |
| `competence_ceiling` | unchanged — it is a scalar ceiling in `settle_energy` units | 0 |
| declared edges / `EdgeId` | comminution and release edges are already legal (F3) | 0 at ≤ 51 materials |
| `VoxelContents` / `Portion` | already `MaterialId`-grade | 0 |
| the loose clastic ladder | already registered (F1/F2) | 0 |

### 3.5 The cost the probes cannot see

**Repeating P11 § 3.3's flag because P10 multiplies it:** the four transport planes live in
the erosion solver, **not** in `DeepField`, so a residency probe reports **zero change** for
the entire § 3.2 table. A gen-memory regression of hundreds of MiB is invisible to every
instrument we ship. On a machine whose build rules exist because memory pressure has hung
it, that is a real hazard. **Owed: a gen-time working-set line in whichever probe measures
this**, before any K > 1 lands.

---

## 4. DQ4 — literature anchors and the P2 gate

Per CLAUDE.md § *A closed system cannot detect its own scale error*: **a constant derived so
a measured quantity lands in a published band is evidence; one tuned until an output looks
right is a number pretending to be a mechanism.**

### 4.1 The anchors, with per-number confidence

| # | anchor | value / band | confidence | source |
|---|---|---|---|---|
| **L1** | **Wentworth/Udden φ partition** | boulder >256 mm · cobble 64–256 · pebble 4–64 · granule 2–4 · sand 0.0625–2 · silt 0.0039–0.0625 · clay <0.0039 | **high** — definitional, standard | priors § 3.1 |
| **L2** | **Sternberg α, gravel-bed rivers** | **0.013–0.018 km⁻¹**; band **0.01–0.02**, e-folding 50–100 km | **medium-high** — two rivers, two papers | Szabó et al. 2013 JGR-ES (Williams R., 0.018); Rio Chagres 0.013/0.017 (priors § 3.3) |
| **L3** | **abrasion vs sorting split is lithology-dependent** | limestone: abrasion ≈ sorting; quartzite: sorting dominates | **medium** — one paper, qualitative | Miller et al. 2014 JGR-ES (priors § 3.3) |
| **L4** | **gravel–sand transition is ABRUPT** | D₅₀ falls **>10 mm (>90 %) over a few channel widths**; a grain-size **gap** exists in bed sediments | **medium** — one review | *The gravel-sand transition and grain size gap*, Sed. Geol. 2021 (priors § 3.3) |
| **L5** | **mass loss to abrasion** | ~**38 % of a pebble's mass** over 10 km headwater→transition | **medium** | same as L4 |
| **L6** | **Hjulström minimum** | critical erosion velocity minimum at **0.2–0.5 mm**, ~**0.2 m/s**; rises both sides | **medium-low** — ⚠ priors § 3.2 flags the 0.2 m/s as recall, not re-verified | priors § 3.2 |
| **L7** | **Shields θ_c** | 0.03–0.06 for Re_p > 500; **0.045–0.047** mixed gravel; trends up with bed slope | **high** for the band, **medium** for which value to use | Meyer-Peter–Müller; Lamb et al. 2008; Bunte et al. 2013 (priors § 3.2) |
| **L8** | **downstream sorting improves** | sorting coefficient decreases downstream in gravel-bed rivers | **low as stated here** — ⚠ this audit did **not** find a numeric band, and it did not resolve **Folk & Ward vs method-of-moments** σ, which changes the number (§ 8) | — |
| **L9** | **granite release spectrum (grus)** | granite weathers to a bimodal grus: quartz sand + clay-grade feldspar alteration + lithic granules | **low as a NUMBER** — the qualitative bimodality is textbook; ⚠ no proportion band retrieved here | — |

### 4.2 The P2 gate — verified and extended

The priors audit § 3.5 split the bands on P2 (`EROSION_CALIBRATION` re-pick + flag flip;
pre-P2 nothing entrains sand — max competence ceiling **0.283** vs coarse clastic's
**0.840**). **Verified against that section; extended below.**

| anchor | pre-P2 status | this audit's extension |
|---|---|---|
| **L1 Wentworth** | **survives** (definitional) | ✓ and now *satisfiable*: F1 shows the registry already spans it |
| **L2 Sternberg α** | priors: **"meaningless"** | ⚠ **SPLIT.** *Validating* fining is meaningless pre-P2 (nothing travels). But **deriving α is P2-independent**: α is km⁻¹, the hop is `DEEP_CELL_M` in metres — **both SI, with no `k_transport` anywhere in the chain**. Contrast the competence/capacity constants, which priors § 3.2 shows have **no SI bridge at all**. **Abrasion is the first deep-time rate constant with a clean dimensional bridge to the literature**, and it can be written, derived and defended today |
| **L3 abrasion/sorting split** | not covered | **partly pre-P2**: it is a statement about which *mechanism* dominates per lithology, and the abrasion half is derivable now |
| **L4/L5 gravel–sand transition** | **"meaningless, doubly"** (no gravel bin either) | ⚠ **half discharged.** The "no gravel bin" half **dies with P11 slice 1** (F1). The "nothing moves gravel" half **stands** and is squarely P2 |
| **L6/L7 Hjulström/Shields** | **meaningless** | ✓ confirmed, and the **dimensional** objection is independent of P2: there is no SI bridge from `cap = k·A^m·S^n` (A in *cells*) to a Shields number at any calibration. **P2 does not create one** |
| **L8 sorting-vs-distance** | not covered | **meaningless pre-P2** (same reason as L2's validation half) |
| **L9 release spectra** | not covered | ⚠ **NOT GATED ON P2 AT ALL.** What a rock sheds when it weathers is a weathering-product question; weathering runs today. **The release-spectrum half of P10 is calibratable and walkable now** — which is the single strongest argument for the § 5 first-slice pick |

> **The gate splits P10 cleanly in two.** *Release spectrum + abrasion derivation* are
> pre-P2 work with real literature contact. *Sorting, competence, the gravel–sand
> transition, and every validation of downstream fining* are post-P2. **Sequence
> accordingly.**

**And the null-reading corollary applies** (CLAUDE.md; journal/0110's facies null): any
pre-P2 measurement reading *"the grain axis records nothing interesting"* is subject to
*"rivers do nothing — true — but partly because nothing does anything"*, and must say so in
the same breath.

---

## 5. DQ5 — the first BUILD slice candidate

**Constraints the pick must satisfy:** seam-first (cheapest cold seam that teaches the
shape) · P11 is TOP PRIORITY and P10 *resumes against the member-grade record* (ROADMAP's
own continuation slot) · calibration is P2-gated except for release spectra (§ 4.2) · the
scratch-pad doctrine permits goldens to move with the why recorded.

### FS-A — the release spectrum + a byte-honest pass-through record ★ RECOMMENDED (assistant-proposed)

**What lands.** Materials gain a **declared release spectrum** (R2's edge shape or R3/R4's
parametric shape — that sub-pick is § 6). Its first and only consumer is the existing
`structure → loose` weathering edge: instead of emitting the source rock's own identity, the
pass emits **products drawn from the F1 ladder** (`SAND` / `SILT` / `CLAY` / `GRAVEL` /
`SCREE`). The record stores those `MaterialId`s — which P11 slice 1 has already made free.
**No evolution. No new plane. No new record byte. No φ arithmetic in any hot loop.**

**Why it is the cheapest seam that teaches the shape:**

- **It costs 0 MiB** — record (F4 step unpaid), planes (K = 1), faces (unchanged).
- **It is pre-P2 walkable and pre-P2 calibratable** (§ 4.2, L9) — the only half of P10 that
  is.
- **It answers the fork empirically before it has to be answered theoretically.** If a
  release spectrum expressed purely as *product identities* (G2) produces a legible world,
  the expensive options G3/G4 may never be needed. If it visibly cannot — if provenance
  collapse is obvious in a cut face — that is the evidence for paying the axis.
- **It makes F2's dead ladder live**, which is a `spines.md` § 3 row discharged rather than
  a new mechanism built beside an existing one (A-1).
- **It uses the shipped edge/agent authority** (F3) rather than inventing a parallel one —
  and it cashes `materials.md` § DECIDED 2026-07-22 (*transformations declared on material
  definitions, the ONE authority both sims consult*) for its first real customer.

**What it costs / what moves.** Goldens move (the deep record starts naming loose clastics
where it named sandstone/mudstone) — announced, with the why recorded, per the scratch-pad
doctrine. `derive_base`'s `reference_material()` up-conversion loses another caller.
`litho_of_tag`/`exposed_litho`'s resistance lookups start reading *loose* sheets for
transported material — which is `entry_species_probe` PART 4's measured **form error**
finally being paid rather than reported. ⚠ **That last one is a real behavioural change with
a measured magnitude already in-tree; it should be sized from the probe before the slice, not
after.**

**Hard prerequisite: P11 slice 1** (`DepUnit::species: Litho → MaterialId`). Without it the
record physically cannot name `SAND`. **FS-A is a P11-slice-1 continuation, not a parallel
arc.**

**Acceptance:** a walk (fullbright + edges, material question) at a tour-mapped exemplar of
a stripped upland vs a distal basin — does the cut face show *different rock* where the sim
says different energy? Plus the probe assertions that keep the release spectrum's shares
summing to 1 (a Law-3-shaped itemisation check, scale-free).

### FS-B — the φ ladder audit only

Register/verify the fixed φ bin boundaries and the loose↔lithified pairing as a typed thing;
no behaviour. **Cheaper, teaches less** — it is a definition without a consumer, which is
`CoarseField` on 2026-07-22 and MM-3 on 2026-07-29. **Not recommended standalone**; fold it
into FS-A.

### FS-C — abrasion first

Per-hop Sternberg on the existing load. **Rejected as a first slice:** pre-P2 nothing travels
far enough for it to do anything (§ 4.2), so it is unobservable on the shipped world; and it
needs a size state to act on, which is the thing FS-A is deciding the shape of.

### FS-D — the record step, paid up front

Pay F4's 42.23 MiB and land K = 8 bins + the mover byte together (§ 7, C7). **Not a first
slice** — it commits the most expensive sub-pick before any evidence exists that it is
needed, and it is the option FS-A is designed to make an informed decision about.

---

## 6. NEEDS RATIFICATION

### 6.1 User-owned (design/content forks; an assistant may not settle these)

| # | question | why it is the user's | where it bites |
|---|---|---|---|
| **U1** | **Is grain an IDENTITY or an AXIS?** — i.e. **does `materials.md` § forms ruling 2 (*"within-identity grain-size gradation is not modelled"*, DECIDED 2026-07-21) get revisited?** | A standing user DECIDED, whose own text says *"maybe it gets revisited at a later time"*. **P10's stated WHAT is exactly what it excludes.** | Decides G2 (0 bytes) vs G3/G4/G5 (the 42.23 MiB step + ×K planes). **Everything else in this document is downstream of it** |
| **U2** | **Is fining-upward a RECORDED SEQUENCE or an EXPRESSED LAYOUT?** (§ 2.3.3) | It is the shape of a signature the user named as the prize (*"a channel gravel with its placer streak, fining upward"*) | If *expressed*: P10 is not blocked by WINDOW. If *recorded*: P10 acquires a hard WINDOW dependency and re-sequences |
| **U3** | **Is BIMODALITY a requirement or a nicety?** | Appearance-class; L4 says the gravel–sand gap is first-order in the real world | Decides G4-vs-G3 **and** R3/R4-vs-R1/R2 at a stroke (§ 1.3) |
| **U4** | **How many φ classes: 8 / 5 / 4 / 2?** | Appearance-class, content-facing | § 2.1's arithmetic bounds it: a full-world traverse buys ≈ **5.4 φ halvings**, so K = 8 is generous and K = 2 discards most of what the world can make. § 3.2 prices each |
| **U5** | **Does the record grow at all** — pay F4's 8-byte quantum (42.23 MiB, +38.9 % of the field) or stay at zero? | Residency is user-visible and *"residency is sacred"* is a shipped discipline | Under G3/G4 **not** paying it means abrasion silently forgets across deposition (§ 2.1) — so U5 and U1 are **coupled and should be ruled together** |
| **U6** | **Is the F1 loose ladder the grain alphabet**, or is a φ-graded member set authored? | Content is user-owned (roster questions always are) | If the ladder is adopted, F2's dead rows come alive; if not, the registry grows toward STUB #21's 51-material ceiling |
| **U7** | **Are release spectra pack-authored content or engine-derived?** | The engine/pack partition (`dependency-graph.md`) is a user-owned line | R1/R2 (authored) vs R3/R4 (derived from the sheet) |

### 6.2 Integrator-settleable (once the above are ruled)

| # | question | the arithmetic that settles it |
|---|---|---|
| I1 | **Fixed vs adaptive φ bins** | **Fixed**, forced twice: additivity requires shared boundaries (§ 2.3.1) and the precomputed settle order requires a static table (§ 2.2.1) |
| I2 | Per-hop Δx for Sternberg | **`DEEP_CELL_M` = 460 m, exact** for an exponential law (§ 2.1); no path-length state needed |
| I3 | Whether to add a `flow_length` field plane | **Only if** a non-exponential attrition law is wanted. 1.13 MiB, computable in the existing downstream sweep |
| I4 | Histogram vs moments in a transient plane | Both additive; histogram iff U3 says bimodality is required |
| I5 | Face-composition shape (C-a / C-b / C-c) | § 3.3's 20× gap; and it re-enters **record-terms ruling 3** after P11 slice 2 |
| I6 | Sparse `(material, φ, share)` triple list vs a dense-in-K CSR payload | **Blocked on measuring `p`** — one probe run, wanted by both this audit and P11 § 3.4 |
| I7 | Where the gen-time working-set line goes | § 3.5 — a probe must be able to see the § 3.2 table at all |
| I8 | α's per-material derivation | `LithoResistance::abrasion` = `resistance(Smash)` is the shipped candidate input (§ 2.1); the mapping shape is engineering |

### 6.3 Owed measurements before any pick that costs bytes

1. **`p`** — mean distinct materials present per cell in the load (I6; P11 § 3.4 wants it too).
2. **Re-run `flow_cost_probe` / `flux_record_probe` / the strata itemisation** — the live-unit
   count predates two recalibrations (§ 3.0), and every per-unit figure multiplies by it.
3. **The form error's magnitude**, from `entry_species_probe` PART 4 — FS-A pays it
   deliberately, so it should be sized first.

---

## 7. Contradictions and priors found in passing (reported, not fixed)

**C1 — ⚠ `materials.md` forms ruling 2 vs the ROADMAP P10 arc.** *"Within-identity
grain-size gradation is not modelled"* (DECIDED 2026-07-21, user) against *"source rocks
release a size DISTRIBUTION (not their sheet's single `grain_size_mm`)"* (SEQUENCED
2026-08-01, user). Both are user-originated, 11 days apart, in documents that do not point at
each other. **This is not a CONTESTS-and-stop** — the earlier ruling explicitly anticipates
revision (*"maybe it gets revisited at a later time"*) and the later one is the user's own —
but it is **the top NEEDS-RATIFICATION item (U1)** and it is exactly the doc-topology shape:
a decision superseded in one place and still asserted in another. **Owed: whichever way U1
lands, `materials.md` ruling 2 gets a banner.**

**C2 — the priors audit's *"abrasion has no property to hang on"* is too strong.**
`docs/audits/2026-07-29-fluvial-record-terms-priors.md` § 1.5 says Sternberg *"has no
property to hang on today; that is a modelling gap"*. Narrowly true (no attrition
coefficient exists) but **`LithoResistance::abrasion` is per-`MaterialId`, is derived from
`resistance(DamageType::Smash)` read straight (`lithology.rs:340-343`), and `Agent::Abrasion`
is live**. A per-material α has a defensible shipped input. **Reported as a narrowing, not a
refutation** — but it changes the feasibility read on P10's abrasion half.

**C3 — a `spines.md` § 3 row that is not listed.** `MaterialId::SAND / GRAVEL / CLAY /
SILT / SCREE` are registered with full property sheets and **have no worldgen consumer**
(F2; grepped over `crates/`, `plugins/`, `tools/` for both the constants and the names).
Their only non-registry appearances are a `dc-api` host test, a meshing sentinel buffer, and
`entry_species_probe`'s pairing table. **"Built, and nothing calls it" — and it is precisely
the machinery P10 needs.** Reported for § 3's index; not added (read-only brief).

**C4 — ⚠ TWO ENCODINGS OF "LOOSE SAND" ARE IN THE TREE AT ONCE, AND THEY DISAGREE.**
`entry_species_probe.rs:455-459` asserts CLAY↔MUDSTONE, SILT↔SILTSTONE, SAND↔SANDSTONE,
GRAVEL↔CONGLOMERATE are *"ONE substance in two forms (the ratified forms pass)"* — i.e.
`MaterialId::SAND` **is** the loose form of sandstone. But the inventory's form axis makes
`(SANDSTONE, InvForm::Loose)` an equally valid and structurally distinct encoding of the same
thing, and `materials.md` ruling 2 says the *second* is the intended one (*"no new sand
identity"*) while the registry ships the *first*. **This is a two-authorities defect that
P10 walks straight into**, because the F1 ladder is made of exactly these identities.
**It should be settled as part of U1/U6, not discovered during the build.**

**C5 — the priors audit's *"nothing in the clastic classes is gravel"* (§ 1.6) has a short
shelf life.** It is true of the `Litho` roster and false of the registry (F1), and **P11
slice 1 discharges it**. Per the correction-author-stamps rule, whoever merges P11 slice 1
owes that section a banner.

**C6 — ⚠ a release spectrum must not redefine `grain_size_mm`.** That field has **three**
shipped consumers reading it as *the* grain size of the material: `settle_energy`
(`geology.rs:455-457`), the pore-fill rule `filler_grain ≤ K_PORE × host_grain`
(`materials.md:175-188`), and a **tested registry invariant** that sieve
`extraction_resistance` **equals** grain size (`mod.rs:295-297`). **Adding a spectrum beside
it is additive and safe; making the scalar mean "the release median" breaks a tested
invariant.** R4 (§ 1.3) is the option that flirts with this and the reason it is ranked last.

**C7 — the record's 8-byte quantum makes two owed items co-riders.**
`earth-processes.md:398` asks for *"widen recorder tags (**agent axis** + grain continuum)"*
— **one sentence, two asks, one byte-step.** STUB #25's mover attribution is the agent-axis
half. Paid together they cost **42.23 MiB**; paid separately, **84.47 MiB**. If U5 says the
record grows, **#25 should ride in the same commit.** Sequencing note, not a decision.

**C8 — `stubs.md` #23 should gain a P10 pointer.** P10 decouples size from density in the
state, which is half of what a Stokes/drag settling law needs, and it makes peat's fake 5 mm
grain size visible in more places. **P10 is a partial enabler, not the heir** (the heir
remains fluid identity). Reported so the entry is not mis-stamped in either direction.

---

## 8. What this audit could not verify — flagged, not smoothed

1. **`p` (mean distinct materials per cell) is unmeasured.** Every sparse figure in § 3.2 is
   parametric in it. Same gap P11 § 3.4 flagged.
2. **No cargo was run.** No `size_of`, no `offset_of!`, no probe. All layout arithmetic
   (`DepUnit` = 16 B / align 8; the 8-byte quantum) is hand-derived from field lists and
   cross-checked against `stubs.md:914-919` and P11 § 3.1 — **not compiler-verified here.**
3. **The 5,535,837 live-unit count predates journal/0111 and /0112** (P11 § 3.0 calls it its
   weakest number). Direction of error unknown. Every per-unit MiB figure inherits it.
4. **The 494,296 loaded-entry count is a lower bound** (FLOW (a) populated vertical faces).
5. **L8 (sorting-vs-distance) has no numeric band here**, and this audit **did not resolve
   Folk & Ward vs method-of-moments σ**. That matters: G4's moment triple gives the
   method-of-moments σ and **cannot** give Folk & Ward's percentile form (§ 2.3). If the
   calibration target turns out to be Folk & Ward, G4 loses its calibration story.
6. **L9 (granite release proportions) is qualitative here.** The bimodality of grus is
   textbook; the *numbers* were not retrieved, and FS-A's calibration story depends on them.
7. **Hack's law coefficients (1.4, 0.6) in § 2.1 proxy 4 are recalled, not re-verified.**
8. **L2/L3/L4/L5/L6/L7 are quoted from the priors audit's citations**, not independently
   re-fetched. The priors audit itself flags L6's 0.2 m/s as recall.
9. **The C3 "no consumers" claim is grep-based** over `crates/`, `plugins/`, `tools/` for
   both the `MaterialId::` constants and the quoted names. A data-driven or dynamically
   constructed reference would be missed — though `from_qualified_name` has only three
   non-test callers, all checked.
10. **The § 2.1 hop-exactness argument was checked against the MFD splitting code and
    survives, with one correction folded into § 2.1** — the hop distance is **not** a
    constant 460 m, because MFD splits across 8 neighbours including diagonals. The fix is
    already in the tree (`MFD_DIST`), so this is a resolved flag rather than an open one.
    What remains unverified: whether the per-species share loop
    (`erosion.rs:2950-2981`) is the right place to apply a per-face factor, or whether it
    belongs in `exchange_cell`. That is a build question, not a design one.
