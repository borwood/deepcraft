# Fluvial member (refinement #1) — record-terms priors

**Read at `f94a568`** (branch `main`; this file added on top). Produced by a **read-only**
research agent for the **fluvial member design pass**, against the user's ruling of
2026-07-29: *record terms first* — deeptime must record a **grain distribution** and a
**mobility hint** before any channel operator builds
(`docs/dependency-graph.md:54`, E5 row: *"#1's live blockers are now the RECORD TERMS
(grain distribution + mobility hint, ruled records-first 2026-07-29) and P2"*).

**This is a dated research artifact.** Quotations and line numbers were verified at
`f94a568` and are **not** re-verified by later readers. Immutable body / mutable header
(CLAUDE.md read-first item 5): anything that later refutes or re-scopes this file gets a
banner **here**, stamped by the author of the correction.

**What this file is:** the priors the design conversation argues from — what already
exists in code, what the corpus has already ratified or sketched, what the literature
bands are, and what each storage shape costs. **What it is not:** a design. It picks no
term, no width, no layout. No code was run (**no cargo invocations**, per the brief); every
cost number below is arithmetic over already-measured quantities, cited to its source.

**Status legend** (same as `docs/audits/2026-07-29-refinement-coupling-priors.md`):
**DECIDED/RATIFIED** (user) · **PROPOSED** (recorded, not decided) · **passing remark**
(prose, no ratification event) · **BUILT** (code exists) · ⚠ **FLAGGED** (this file could
not verify it).

**Read with:** `docs/design/refinement.md` § 4 (the term schema) / § 7.1 (the channel
operator) / § 5 (Laws 1–3) · `docs/design/flow.md` § 5 limit 3, § 11.1 (WINDOW), § 11.5
(the three-mode confinement rule) · `docs/spikes/S19-flow-record-cost-results.md` ·
`docs/design/material-behavior.md` § 13 · `docs/design/stubs.md` #23 / #25 / #26.

---

## 0. The headline, in four sentences

1. **`grain_distribution` is almost entirely a RECORDING change, not a modelling change.**
   The per-face, per-species load composition is *already computed* — as an inner-loop
   temporary at `erosion.rs:2962` — and then **summed to a scalar** at `erosion.rs:2978`
   before it reaches `FluxEntry::load`. Widening the `face_total` accumulator from
   `[f64; 8]` to `[[f64; 7]; 8]` is the whole arithmetic.
2. **But the *identity* it would record is a PROVENANCE axis, not a grain-size axis**, and
   the two are non-monotone in the shipped roster (peat, 5 mm, outranks sandstone, 0.3 mm).
   A `grain_distribution` recorded as `Litho` shares **cannot express downstream fining in
   the Wentworth sense** — that part *would* be a modelling change (§ 1.6).
3. **`mobility_hint` has almost no corpus prior — one sentence** (`flow.md:564-565`) — and
   its nearest existing computation, **χ = A·S²** (`erosion.rs:612`), is also computed
   per-cell per-epoch and discarded. A large part of the hint may be **derivable from the
   record already shipped** (out-face count / magnitude dispersion per chapter), which S-2
   says must be checked before storing anything (§ 1.4, § 4.6).
4. **`FluxEntry` has 2 bytes of tail padding today** (layout arithmetic, § 4.2) — a
   zero-residency-cost budget of 16 bits for whichever term wants it first. Everything
   wider prices out on § 4's table.

---

## 1. Q1 — what already exists in code, and is discarded or aggregated away

### 1.1 Summary table

| quantity | where computed | fate | verdict for the term |
|---|---|---|---|
| **per-face per-species load share** | `erosion.rs:2962` (`share = wt * q_s`), accumulated at `:2964` into `face_total` | **summed to one f32** at `erosion.rs:2976-2979` → `FluxEntry::load` | **`grain_distribution` is a RECORDING change** |
| **per-cell load multiset** `qs_sp` (`n × 7`) | `erosion.rs:1465` (field), written by `exchange_cell`/`split_by_shares` | per-epoch scratch; zeroed at `:2806` | already the § 13.3 multiset, live |
| **per-cell per-species deposition** `dep_sp` (`n × 7`) | `erosion.rs:1474` | **argmax'd to one `Litho`** by `arriving_species` (`erosion.rs:1160`) → `DepUnit::species` | 7-vector → 1 byte; stubs #25 |
| **per-cell per-species creep** `creep_sp` (`n × 7`) | `erosion.rs:1490` | folded into the same argmax (`erosion.rs:3498`) | ditto |
| **transport capacity `cap`** (`k·A^m·S^n`, the stream-power proxy) | `erosion.rs:2593` (`self.energy[c] = cap`) | **quantized to 3-valued `EnergyBand`** (`energy_band`, `erosion.rs:1084`) into `DepTag`; the plane is **NOT in `DeepField`** | the continuous energy a mobility hint wants is thrown away |
| **share-weighted energy slope `S̄ = Σ w_d S_d`** | `erosion.rs:2876-2886` (`s_bar`) | pure local temporary; never stored | journal/0113's slope term, discarded |
| **channelisation index χ = A·S²** | `erosion.rs:612` | sets `p` / the channel switch; never stored, never exported | **the nearest existing thing to a mobility hint** |
| **`is_channel(χ)` — a per-cell-per-epoch confinement verdict** | `erosion.rs:619` (`MfdParams::is_channel`, `:324`) | consumed and dropped | ⚠ a *verdict*; recording it is the Law-2 / S-3 hazard (§ 6.3) |
| **MFD out-weights `mfd_w`** (`n × 8`) | `erosion.rs:1421` | scratch | the flow-direction fabric § 13.8 wants |
| **competence ceiling** | `competence_ceiling(cap, k_t)`, `erosion.rs:458` | derived on demand | derivable from `cap`; not a storage question |
| **settling velocities `w_settle` / `ws_order`** | `erosion.rs:1443-1444`, from `lithology::settling_table()` (`:647`) | static per run | a **property-sheet** fact, not a per-cell one |
| **avulsion (temporal divergence)** | `FluxRecord::out_face_count` (`flux.rs:507`) | **already derivable from the shipped record** | see § 1.4 |
| **simultaneous divergence** | `SimulDivergence` (`flux.rs:456`) counted at `flux.rs:826-833` | **kept**, 24 B, explicitly *not* a fact | already the honest instrument for the erased axis |

### 1.2 The sharpest finding: the composition is computed per face and summed away

`erosion.rs:2950-2981`, the MFD sorted path:

```rust
let mut face_total = [0.0f64; MFD_DIRS];
for s in 0..SPECIES {
    let q_s = self.qs_sp[sbase + s];
    …
    for (d, ft) in face_total.iter_mut().enumerate() {
        …
        let share = if d == last { q_s - given } else { wt * q_s };
        given += share;
        *ft += share;                                  // ← the composition dies here
        let j = self.mfd_neighbour(c, d);
        self.qs_sp[j * SPECIES + s] += share;
    }
}
if record {
    for (d, ft) in face_total.iter().enumerate() {
        if self.mfd_w[base + d] > 0.0 {
            self.out_face_load[base + d] = *ft as f32;  // ← only the sum survives
        }
    }
}
```

The `(d, s)` pair exists as `share` on line 2962. `face_total` deliberately collapses `s`.
The comment two lines above says exactly why the scalar is a sum and not a second split
(*"one arithmetic, so the record and the budget cannot disagree"* — flow.md § 3), and that
discipline **transfers unchanged** to a per-species face record: the scalar stays the sum of
the species, so no new arithmetic and no new drift risk is introduced by recording the parts.

`FluxEntry::load`'s own doc comment already names this seam as owed
(`flux.rs:399-402`): *"**Bulk only.** The *composition* of the load — which materials, in
what proportion, the thing a placer streak in channel gravel is made of — is a named seam
whose heir is **Movement 2b, material-aware transport**."* Movement 2b **shipped**
(journal/0110, /0112) — so the heir named in that comment has landed and **the caption is
now stale in the direction the gate cannot see** (§ 8.2).

The **single-receiver (non-MFD) path** does the same thing more simply: it moves the whole
multiset species-by-species (`erosion.rs:2842-2844`) and writes `qs_out` — the scalar total —
to the face (`erosion.rs:2859`). Both paths therefore have the composition in hand.

### 1.3 The transport solve *does* know which species, before the argmax

Yes, unambiguously:

- `qs_sp` (`erosion.rs:1454-1465`) is `n × SPECIES` metres of suspended material and is
  documented as **"the mass authority when it is non-empty"** — the scalar `qs` plane is not
  maintained on the sorted path at all.
- `dep_sp` (`:1471-1474`) is what the pass **set down** per species per cell per epoch.
- `arriving_species` (`:1160`) then takes **one argmax over the whole mixture** — fluvial
  `dep` plus creep `creep` — with ties to the incumbent, and that single `Litho` becomes
  `DepUnit::species`. `stubs.md` #25 is the record of that deliberate collapse.

So there are **two** aggregations in flight, at different granularities: the **face**
composition (§ 1.2, per face per chapter) and the **unit** composition (per stratum slot).
They answer different questions — *what moved through here* vs *what settled here* — and
`refinement.md` § 7.1 names both as channel-operator inputs (*"the load budget, grain
distribution … and the column inventory"*). Whether the grain term lands on the face or on
the unit is § 4.4's cost fork, not a detail.

### 1.4 Mobility: what the record can already answer without a new term

**S-2 (store only what derivation cannot predict) has to be discharged before any mobility
term is designed**, because the shipped record already answers a real mobility question:

- `FluxRecord::out_face_count(cell, chapter)` (`flux.rs:507`) — **≥ 2 is avulsion**: the
  flow left this cell by two different faces during the chapter. `flux.rs:441-449` states
  the semantics precisely and distinguishes it from the simultaneous kind.
- Measured on the shipped hybrid-`p` world (journal/0113:214): **temporal (avulsion)
  `(cell, chapter)` pairs = 417,981**, against 337,406 under D8 and 418,889 under uniform
  `p = 4`. Simultaneous: 7,228,964 `(cell, epoch)` pairs / 398,955 distinct
  `(cell, chapter)`; max lateral out-faces in one epoch = 8.
- The **magnitude dispersion** across a cell's out-faces in a chapter (how evenly the
  chapter's flux split) is likewise a read-time computation over `entries_for(i)`.

So *"did the channel wander here, and how much"* is **already in the record**. What is *not*
derivable is the **energy** half of `flow.md`'s *"mobility/energy hint"*: the transport
capacity `cap` (§ 1.1) and χ are both destroyed each epoch, and the `head` plane
(`field.rs:557-573`) is a **final-state** field, not per chapter.

### 1.5 Do materials carry grain / durability / settling properties? Yes — three of them

`dc-core/src/materials/mod.rs:288-292`: `density_kg_m3`, `grain_size_mm`, `cohesion`.
`geology.rs:455-457`:

```rust
pub fn settle_energy(props: &MaterialProps) -> f64 {
    (f64::from(props.grain_size_mm) * f64::from(props.density_kg_m3) / 1000.0).sqrt()
}
```

`lithology::settling_table()` (`:647-653`) reads it off each `Litho`'s
`reference_material()` (`:301-311`). Computed from the shipped sheets:

| `Litho` | reference material | `grain_size_mm` | `density_kg_m3` | `settle_energy` |
|---|---|---:|---:|---:|
| Basement | granite | 3.0 | 2700 | **2.846** |
| OrganicPeat | peat | 5.0 | 400 | **1.414** |
| ClasticCoarse | sandstone | 0.3 | 2350 | **0.840** |
| OrganicCharcoal | charcoal | 2.0 | 350 | **0.837** |
| OrganicCoal | coal | 0.05 | 1350 | **0.260** |
| ClasticFine | mudstone | 0.004 | 2400 | **0.098** |
| OrganicSoil | carbonaceous mudstone | 0.004 | 2200 | **0.094** |

*(Arithmetic by this audit from `mod.rs:506-690`; the 0.840 and 1.414 rows reproduce the
figures independently stated at `erosion.rs:402` and `stubs.md:799`, which is the check.)*

**No durability / abrasion-hardness axis for grains exists.** `LithoResistance`
(`lithology.rs:333`) is a per-(rock, *agent*) erosional resistance for the weathering and
incision terms — abrasion / dissolution / frost / wave — **not** a grain-attrition
coefficient. Sternberg-style abrasion (§ 3.3) therefore has **no property to hang on** today;
that is a modelling gap, not a recording one.

### 1.6 ⚠ The load-bearing caveat: `Litho` is a provenance axis, not a grain axis

From § 1.5's table, treated as a grain-size distribution:

- **Two classes collide exactly** — ClasticFine and OrganicSoil are both 0.004 mm; they
  differ only in density (2400 vs 2200).
- **The ordering inverts against reality** — peat (5 mm, ρ 400) reads as the
  second-heaviest species in the roster, above sandstone. This is `stubs.md` **#23**
  (*a-settling-law-that-cannot-see-buoyancy*, `stubs.md:789-820`): *"Real peat **floats**…
  the day a flow does entrain organic material it will sink like gravel."* Its heir is
  fluid identity (`flow.md` § 2.5), which is **not built**.
- **Nothing in the clastic classes is gravel.** ClasticCoarse's reference sheet is
  *sandstone at 0.3 mm* — medium sand — while the class is documented as
  *"sandstone/**conglomerate**"* (`lithology.rs:233`); `CONGLOMERATE` is 8 mm and
  `GRAVEL` 20 mm in the registry, and **neither is a `Litho`**. The deep tier's clastic
  span is therefore 0.004 mm → 0.3 mm, roughly clay → medium sand: **less than two
  Wentworth phi-decades**, against the four a real gravel→clay fining profile needs.
- `SPECIES = Litho::COUNT` and the reason is explicit (`erosion.rs:371-377`): *"one per
  `Litho`, which is the material granularity deep time can distinguish at all… resolving
  the load any finer than this would be inventing identity the tier does not have."*

**Consequence the design conversation must not paper over:** recording `Litho` shares per
face is cheap and honest and gives **provenance** — which is what journal/0112 measured a
65 % effect from. It does **not** give a grain-size distribution, and any operator that
reads the term *as* grain size (levee coarse → floodplain fine, § 7.1's *"deposition graded
by distance"*) is reading a proxy with one exact collision and one inversion in it. Making
the term a real grain distribution is a **modelling** change: either a second axis on the
load (grain bins independent of class) or a widening of `Litho` — both large, neither
sketched anywhere in the corpus.

---

## 2. Q2 — what the corpus has already ratified or sketched about grain / sorting / mobility

### 2.1 Grain, sorting, competence

| # | claim | status | citation |
|---|---|---|---|
| a | **Transport is a FAMILY** — water/wind/ice/gravity share one load/entrain/sort/deposit machine, differing in *field* and *competence curve* | **RATIFIED 2026-07-24** (user) | `material-behavior.md:832-841` (§ 13.2) |
| b | **The load is a multiset of `(material, quantity)`** riding the chain, pass-transient | **RATIFIED** (§ 13's status block: R/H unification + material-aware transport *"ratified this session"*) | `material-behavior.md:842-846` (§ 13.3); status at `:809-814` |
| c | **Deposition = capacity + competence + a settling sort**; *"sorting is the falling ceiling; we write the ceiling, not the sort"* | assistant framing, **user-endorsed "sound and promising"** — i.e. **PROPOSED-strong**, explicitly *"refine on build"* | `material-behavior.md:857-866` (§ 13.5); status at `:809-814` |
| d | **Entrainment is Hjulström-shaped**, `τ_c` U-shaped in (velocity, grain size), from `grain_size_mm` + `cohesion`; fines-first when marginal → lag/armouring/desert pavement | same status as (c) — **PROPOSED-strong**, and **NOT built** (§ 13.8b: the rain-out *"is deliberately **not** armouring, which needs § 13.4's selective entrainment"*) | `material-behavior.md:847-856` (§ 13.4) |
| e | The **7-species load, identity-travels, `DepUnit::species`, per-species mass closure** | **BUILT 2026-07-26** (journal/0110), on by default | `material-behavior.md:900-952` (§ 13.8b); `recorder.rs:273` |
| f | **Creep joins the family by nullifying competence** — *"its competence curve is that there isn't one"* | **BUILT 2026-07-26** (journal/0112) | `material-behavior.md:953-1028` (§ 13.8c) |
| g | **`COMPETENCE_SCALE = 420` is anchored, not fitted** — on the shipped facies rule's own Low/Medium capacity boundary crossing coarse clastic's `settle_energy` | **BUILT**, with the derivation written out; re-expressed relative to `k_transport` 2026-07-26 (corrections #59) | `erosion.rs:379-424`; `material-behavior.md` § 13.8b |
| h | **Placers fall out of grain sorting almost free** — dense gold rides the coarse fraction | **DECIDED-adjacent / ratified direction** (geology backbone) | `geology.md:57`, `:73`, `:261`; `ores.md:58-64`, `:221` |
| i | **Flow-biased sub-cell fill** — collapse should read flow direction + topography to bias grain placement (coarse near the paleochannel, fines in distal lows, cross-beds downflow) | **named deferred layer**, ROADMAP-Sequenced ("STRUCTURE-AWARE FINE EXPRESSION") | `material-behavior.md:1032-1038` (§ 13.8) |
| j | **Coal partings** — deterministic position-seeded mineral bands inside thick organic units, a *collapse-tier* procedural detail, never simulated at deep time | **user-raised 2026-07-20, scoped**; adopted as refinement member #2 | `ideas.md:440-464`; `refinement.md:292-295` |
| k | **"Rock is not monolithic" — jitter at the partials scale, dependent on a material property**, softening boundaries especially for soft materials | **user direction, 2026-07-20**, sketch-tier (in `ideas.md`, "sketches, NOT decisions") | `ideas.md:402-439` |
| l | **Grain size gates pore fill** — `filler_grain_size ≤ K_PORE × host_grain_size`, `K_PORE ≈ 0.25` | **DECIDED-tier material rule** | `materials.md:175-188` |
| m | **`settle_energy` cannot see buoyancy** | **STUB #23**, heir = fluid identity, unbuilt | `stubs.md:789-820` |
| n | *"this gravel looks water-sorted…"* — sortedness as **diegetic player-readable knowledge** | **user sketch** (`ideas.md` charter: sketches, not decisions) | `ideas.md:98-100` |
| o | **Signatures wanted**: *"fining-upward river stacks; dune cross-beds"*; sim note *"energy-threshold settling (S8, proven) generalizes per agent"*; and the recorder should *"widen recorder tags (agent axis + **grain continuum**)"* | **design-doc statement of intent** (earth-processes is a design doc, future-tense) — **the only place in the corpus that asks for a grain CONTINUUM in the record** | `earth-processes.md:185-187`, `:398`, `:409` |

### 2.2 Mobility — and the honest finding is that there is almost nothing

A corpus-wide grep for `mobility` returns **eight** hits, and after excluding this file
they are: `flow.md:565`, `refinement.md:138/282/287/290`, `dependency-graph.md:54`, and
one unrelated use (`grid.rs:82`, *vegetation* mobility governed by root cohesion).

| # | claim | status | citation |
|---|---|---|---|
| a | *"**Sub-cell lateral channel migration** (meander belts at 460 m) is refinement-tier; **the record can carry a mobility/energy hint, not the path.**"* | **RATIFIED 2026-07-25** as a named limit of the flow model (the § 5 adversarial pass, user-requested) | `flow.md:563-565` |
| b | The channel operator's inputs include *"the mobility hint"*; *"scars from mobility + position noise"*; *"Named limit honored: the record carries a mobility hint, never the meander path."* | **CAUTIOUSLY RATIFIED 2026-07-29** as a *direction*, explicitly **not a build order** | `refinement.md:279-291`, and the § 7 banner `:249-252` |
| c | *"floodplain/levee/oxbow/**terrace** as a channel that *moved*"* is listed **representable** | **RATIFIED 2026-07-25** (the same adversarial pass) | `flow.md:540` |
| d | Every divergence in today's record is **temporal — avulsion** — and that is *"the actual physical origin of braid plains, alluvial fans and distributary networks"* | **BUILT + RATIFIED**; the falsified half is struck at `flow.md:285` | `flow.md:272`, `flux.rs:24-38` |
| e | **χ = A·S² is Montgomery & Dietrich (1988, 1992)'s channel-initiation criterion** and *"once flow is channelised it is confined, and confined flow takes one path"* | **BUILT 2026-07-26** (journal/0113); the *index* is cited literature, **the two threshold constants are STUB #26** | `flow.md:384-388`; `erosion.rs:221-245`; `stubs.md:944-985` |
| f | *"**Vegetation invented meandering rivers** — pre-Devonian channels are overwhelmingly braided; rooted banks are what let a sinuous channel hold form."* | in `ecology.md`, whose header is *"substrate **RATIFIED** 2026-07-19"* — and CLAUDE.md's existence-is-not-standing clause **explicitly does not reach this doc** | `ecology.md:63-66` |
| g | **River terraces from uplift pulses are sub-460 m landforms**; chaptered forcing ramps are *"what buy water gaps, terraces"* | design-doc statement of intent | `tectonics.md:596`; `earth-processes.md:97` |
| h | *"**Oscillatory / tidal flow** nets to ≈ 0 and would read as 'no flow.' Needs a **gross-energy term separate from net flux**"* | **RATIFIED as a named limit**, partially paid: `flux.rs:88-90` keeps both directed halves rather than a signed net, *"a partial down payment"* | `flow.md:557-560`; `flux.rs:88-90` |

**The asymmetry is the finding.** `grain_distribution` sits on a ratified family with two
shipped slices, a measured 65 % effect, and eight corpus anchors. `mobility_hint` sits on
**one clause of one named limit**, promoted into `refinement.md` § 4's schema by the same
design pass that wrote the schema. Nothing in the corpus says what a mobility hint *is*,
what units it has, or what an operator does with it. Item (f) is the only *physical* prior
that constrains it, and it constrains the wrong axis (bank cohesion, i.e. biology, not flow).

---

## 3. Q3 — the literature bands these terms must be derivable against

Per CLAUDE.md § *A closed system cannot detect its own scale error*: **a constant derived so
a measured quantity lands in a published band is evidence; one tuned until an output looks
right is a number pretending to be a mechanism.** These are the bands.

### 3.1 Grain-size classes — the Wentworth / Udden scale

The standard clastic partition, φ = −log₂(d in mm):

| class | diameter | φ |
|---|---|---|
| boulder | > 256 mm | < −8 |
| cobble | 64–256 mm | −8…−6 |
| pebble/gravel | 4–64 mm | −6…−2 |
| granule | 2–4 mm | −2…−1 |
| sand | 0.0625–2 mm | −1…4 |
| silt | 0.0039–0.0625 mm | 4…8 |
| clay | < 0.0039 mm | > 8 |

**Calibration target:** any `grain_distribution` term should be expressible in φ bins, and
a fining profile should be readable as a **downstream decrease in mean φ**. Against § 1.5:
the shipped roster's clastic classes occupy **φ ≈ 1.7 (sandstone) and φ ≈ 8.0 (mudstone)** —
two points, one of them at the clay boundary. There is no gravel bin. This is the single
hardest literature-vs-code gap in this audit.

### 3.2 Entrainment thresholds — Hjulström and Shields

- **Hjulström (1935)** curve: the critical *erosion* velocity is **minimum at ~0.2–0.5 mm
  (fine–medium sand), around 0.2 m/s**, and rises on **both** sides — coarser by weight,
  finer by cohesion. `material-behavior.md` § 13.4 already states this shape and names
  `cohesion` as the fine-side term. ⚠ The 0.2 m/s figure is this audit's recall of the
  standard curve, not re-verified against Hjulström's paper.
- **Shields (1936)** dimensionless critical stress θ_c = τ_c / ((ρ_s − ρ_f) g D). For
  particle Reynolds numbers Re_p > 500 (coarse gravel, fully turbulent), **θ_c ≈ 0.03–0.06**;
  **0.045–0.047** is the widely used mixed-gravel value (Meyer-Peter–Müller / Yalin &
  Karahan); the curve has a **minimum θ_c ≈ 0.03–0.04 near R\* ≈ 10–20**, which is the peak
  of relative particle mobility. θ_c is **not constant** — it trends upward with channel-bed
  slope ([Lamb et al. 2008](https://agupubs.onlinelibrary.wiley.com/doi/full/10.1029/2007JF000831);
  [Bunte et al. 2013](https://agupubs.onlinelibrary.wiley.com/doi/full/10.1002/2012WR012672)).
- **Note the dimensional mismatch, which is a hazard not a detail.** Both bands are in SI
  (m/s, Pa). The sim's flux magnitudes are *"one cell-epoch of the drainage solve's seeded
  source"* (`flux.rs:385-389`) and `cap = k_transport · A^m · S^n` with `A` in **cells**
  (`stubs.md:954-962`). **There is no dimensional bridge from the record to a Shields
  number today.** Any claim that the sim's competence ceiling lands in a published band is
  currently unmakeable, and `COMPETENCE_SCALE`'s own derivation is careful to be an
  *internal* anchor (§ 2.1(g)) rather than a literature one.

### 3.3 Downstream fining — Sternberg's law

`D(x) = D₀ · e^(−α x)`, α the diminution coefficient (km⁻¹). Field values from gravel-bed
rivers:

| river | α (km⁻¹) | source |
|---|---|---|
| Williams River, Australia (max size) | **0.018** | [Szabó et al. 2013, JGR-ES](https://agupubs.onlinelibrary.wiley.com/doi/full/10.1002/jgrf.20142) |
| Rio Chagres, Panama — felsic / mafic | **0.013 / 0.017** | [Trends of grain sizes on gravel bars in the Rio Chagres](https://www.sciencedirect.com/science/article/abs/pii/S0169555X06001796) |

**Band to calibrate against: α ≈ 0.01–0.02 km⁻¹** for lithologically ordinary gravel, i.e.
an e-folding length of **50–100 km**. Two partition facts matter for the model's shape:

- **Sorting vs abrasion split by lithology** — *"abrasion and sorting are equally important
  in controlling the fining rate of limestone, while sorting is primarily responsible for
  the fining of more resistant quartzite"* ([Miller et al. 2014,
  JGR-ES](https://agupubs.onlinelibrary.wiley.com/doi/full/10.1002/2014JF003156); see also
  [Downstream variation of grain size in gravel rivers: abrasion versus selective
  sorting](https://link.springer.com/chapter/10.1007/BFb0011201)).
- **The gravel–sand transition is abrupt** — median grain size can fall by **> 10 mm
  (> 90 %) over a few channel widths**; ~**38 % of a pebble's mass** is lost over 10 km
  headwater-to-transition ([The gravel-sand transition and grain size gap in river bed
  sediments](https://www.sciencedirect.com/science/article/abs/pii/S0012825221003391)).

**What this means for the term.** Sorting is what the shipped falling-ceiling mechanism
already *is* (§ 2.1(c)); **abrasion is entirely absent** and has no property to attach to
(§ 1.5). At a 460 m cell, an α = 0.015 km⁻¹ e-folding is **~145 cells** — which is a
statement about the *deep tier*, not about a refinement operator, and it means a fining
gradient is a whole-network claim measurable off the archive rather than something a
per-cell operator can be judged on.

### 3.4 Lateral channel migration — the mobility hint's only real band

| population | migration rate | source |
|---|---|---|
| Amazon Basin, reach-averaged | **0.62 – 29.37 m/yr**, median **2.12** | [Pace of global river meandering influenced by fluvial sediment supply, EPSL 2024](https://www.sciencedirect.com/science/article/pii/S0012821X24001079) |
| US rivers, reach-averaged | **0.05 – 6.23 m/yr**, median **0.77** | same |
| lower Mississippi, pre-modification | **1 – 123 m/yr** | same |

Controls, as published: **discharge/stage variability raises mobility**; **fine (clay)
cohesive banks lower it**; **sediment supply** is a first-order control in both field
meanders and braided flume channels ([Controls on lateral channel-migration rate of braided
systems in coarse non-cohesive sediment,
ESPL](https://onlinelibrary.wiley.com/doi/full/10.1002/esp.4710); [High variability in flood
discharge and stage accelerates river mobility, Sci.
Adv.](https://www.science.org/doi/10.1126/sciadv.adv7637)).

**The scale arithmetic that decides whether a mobility hint means anything at 460 m.** A
chapter is ~62.5 Myr (500 Myr / 8, `recorder.rs:229-239`). At the US median of 0.77 m/yr,
one chapter is ~4.8 × 10⁷ m of lateral migration — **~10⁵ cell widths**. Even at the *slowest*
published rate (0.05 m/yr) a chapter is ~3,100 km. **Over one chapter, every channel has
swept its entire valley many times over.** So:

> A per-chapter mobility hint cannot be *"how far did the channel move"* — that answer is
> saturated for every chapter of every world. It can only be *"how confined was it while it
> moved"* — a belt-width / confinement statement, which is what χ and the out-face count
> already speak to (§ 1.4).

This is the strongest constraint this audit found on the shape of the mobility term, and it
comes from the literature rather than from the code, which is exactly the intended use of
the rule.

### 3.5 ⚠ Which of these bands are meaningless until P2 lands

`EROSION_CALIBRATION = 45.0` exists but **`calibrated_rates` ships `false`**
(`grid.rs:599`, and `field.rs:118-121` states the three measured reasons). ROADMAP P2
(`dependency-graph.md`, P2 row) is *"`EROSION_CALIBRATION` re-pick + flag flip — 45 was
fitted to the broken solve and inverts under the fixed one"* (the fixed solve = journal/0122).

On the shipped (1×) world, measured:

- catchment-averaged denudation **0.0110 m/Myr** — 9× below the slowest landscape ever
  measured on Earth, 493× below the global ¹⁰Be outcrop median (Portenga & Bierman 2011);
  target band 1–10 m/Myr (`field.rs:98-102`).
- **fluvial transport is 0.109 % of sediment routing**; creep moves 918× more
  (`material-behavior.md` § 13.8b).
- **no cell anywhere can carry sand** — max competence ceiling **0.283** against coarse
  clastic's **0.840**, and max transport capacity **6.74 × 10⁻⁴**, three times below the
  Low/Medium boundary the ceiling is anchored on (`material-behavior.md` § 13.8b).

Therefore, **meaningless until P2**:

| band | status pre-P2 |
|---|---|
| Hjulström / Shields entrainment thresholds (§ 3.2) | **meaningless.** Nothing entrains sand; the whole coarse half of the U-curve is unreachable. |
| Sternberg α / downstream fining (§ 3.3) | **meaningless.** journal/0110 measured the grain-size gradient *"unchanged to four decimals"*; § 13.8b's null. |
| the gravel–sand transition (§ 3.3) | **meaningless**, doubly — no gravel bin exists either (§ 1.6). |
| lateral migration rates (§ 3.4) | **the scale argument survives P2** (it is arithmetic on chapter length, § 3.4), but any *magnitude* comparison does not. |
| Wentworth classes (§ 3.1) | **survives** — it is a definitional partition, and § 1.6's finding that the roster does not span it is true at any calibration. |

**And note the corollary CLAUDE.md attaches to a null** (journal/0110's facies null): *rivers
do nothing — true — but partly because nothing does anything, and that was never a fact about
rivers.* Any "the grain term records nothing interesting" measurement taken **before** P2 is
subject to exactly that corollary, and must say so.

---

## 4. Q4 — the cost envelope

### 4.1 The measured frame

From S19 (`docs/spikes/S19-flow-record-cost-results.md`) and journal/0096:

| quantity | value | source |
|---|---|---|
| deep grid | 545² = **297,025 cells**, 460 m, **8 chapters** | S19 § 1 |
| `DeepField` **without** the record, pre-shrink / merged main | **162.57 / 108.55 MiB** | S19 § 3; journal/0096:266-267 |
| **flow record total** | **40.66 MiB** = 39.53 (entries) + 1.13 (CSR index) | journal/0096:255-258 |
| **entries** | **2,590,372** × 16 B | journal/0096:256 |
| face sparsity | **8.386 %** (13.627 % of the lateral rectangle) | journal/0096:259 |
| entries carrying **load > 0** | **494,296** — *"99.86 % of all lateral faces"* | journal/0096:296-298 |
| marine-sink entries (carry only the seeded constant) | **79.38 %** of all entries = 31.37 MiB; dropping them leaves **9.29 MiB** — a sized, deliberately unpulled lever | journal/0096:279-291 |
| strata: live / capacity units | 5,535,837 / 9,076,380 — slack **54.02 MiB**, **since reclaimed** by `shrink_to_fit` | S19 § 3(a); journal/0096:270-271 |
| `size_of::<DepUnit>()` | **16 B, zero padding left** | stubs #25 (`stubs.md:914-919`) |
| S19's payload verdict | *"The CSR index floor alone is 9.06 MiB = 0.056× T, **paid at any sparsity**… it is not the constraint; **the payload is**."* | S19 § 5(a) |

⚠ **The entry count is a 2026-07-25 measurement and predates two changes.** FLOW
continuation (a) populated the **vertical** faces from the head field (`flux.rs:47-58`,
`head_field: true` in `grid.rs:580`), and journal/0124 records that *"the flow record's
**vertical** faces moved (a fix) and gained `GOLDEN_FLUX`"*. So **2,590,372 is a lower
bound on today's count**, and every arithmetic below is a lower bound with it. Re-measuring
needs `flow_cost_probe` / `flux_record_probe`, which this audit may not run.

### 4.2 The free 16 bits — `FluxEntry` has 2 bytes of tail padding

`FluxEntry` is `#[repr(C)]` (`flux.rs:379-380`). Field-by-field:

| field | type | align | offset |
|---|---|---:|---:|
| `magnitude` | `f32` | 4 | 0 |
| `load` | `f32` | 4 | 4 |
| `fluid` | `FluidId(u16)` | 2 | 8 |
| `chapter` | `u8` | 1 | 10 |
| `face` | `FaceKey` (`repr(u8)`) | 1 | 11 |
| `form` | `FlowForm` (`repr(u8)`) | 1 | 12 |
| `cause` | `FlowCause` (`repr(u8)`) | 1 | 13 |

Fields occupy **14 bytes**; struct alignment 4 rounds `size_of` to **16** — so bytes 14–15
are **padding**. `flux.rs:964-966`'s `a_flux_entry_is_sixteen_bytes` pins the *total*, which
is satisfied by 14 or 16 bytes of fields alike, so **the test does not forbid using them**
and would still pass after they are used.

⚠ **This is layout arithmetic, not a compiler-verified fact** — the audit could not run
`offset_of!`. The doc comment at `flux.rs:376-377` (*"laid out so the two `f32` payloads
pack the tags into the tail padding"*) is consistent with it but does not state the residual.
**Confirm with an `offset_of!`/`size_of` assertion before designing against it.**

**Capacity:** 16 bits. Enough for e.g. a `u8` quantized coarse-fraction + a `u8` sortedness
index, or a `u16` packed 5-bin grain histogram at ~3 bits/bin, or a `u8` mobility hint plus
a `u8` spare. **Not** enough for 7 × f32 (28 B) or 7 × u8 (7 B) *and* a mobility byte.

### 4.3 Widening `FluxEntry`

Added residency = Δbytes × entries, at **2,590,372** entries (§ 4.1's lower bound):

| shape | new `size_of` | Δ/entry | added residency | × the 40.66 MiB record | × the 108.55 MiB field |
|---|---:|---:|---:|---:|---:|
| use the 2 padding bytes | 16 | **0** | **0** | **1.00×** | **1.00×** |
| + 7 × `u8` quantized shares (5 fit in padding? no — 7 needs 8) | 24 | 8 | **19.76 MiB** | 1.49× | +18.2 % |
| + 7 × `f32` exact shares | 44 | 28 | **69.17 MiB** | 2.70× | +63.7 % |
| + 7 × `f16`/`u16` shares | 32 | 16 | **39.53 MiB** | 1.97× | +36.4 % |
| + a single `u8` mobility hint (into padding) | 16 | **0** | **0** | 1.00× | 1.00× |

Arithmetic: 2,590,372 × 8 = 20,722,976 B = 19.76 MiB; × 16 = 39.53 MiB; × 28 = 69.17 MiB.

**Note the 16-byte assert is a deliberate tripwire**: *"Residency is sacred; a silent
widening here multiplies by millions"* (`flux.rs:961-962`). Any widening must delete or move
that assertion **on purpose**, which is the A-1 discipline the module already practises.

### 4.4 A parallel array populated only where `load > 0`

**The fraction is measured:** 494,296 of 2,590,372 entries = **19.08 %**. (And it is
structurally *"99.86 % of all lateral faces"* — the complement is the marine-sink and
vertical entries, which carry zero load by construction: `flux.rs:394-397` states both zeros
and why they are honest.)

| shape | per-loaded-entry | rows | added residency | × record |
|---|---:|---:|---:|---:|
| side array, 7 × `u8` + `u32` entry index (+1 pad) = 12 B | 12 | 494,296 | **5.66 MiB** | 1.14× |
| side array, 7 × `f32` + `u32` index = 32 B | 32 | 494,296 | **15.08 MiB** | 1.37× |
| **second CSR** over cells for the loaded subset: 7 × `u8` payload (7 B, no index needed per row) + `u32` row offset per cell | 7 + (297,026 × 4 / rows) | 494,296 | 3.30 + 1.13 = **4.43 MiB** | 1.11× |
| **second CSR**, 7 × `f32` payload | 28 | 494,296 | 13.20 + 1.13 = **14.33 MiB** | 1.35× |

Arithmetic: 494,296 × 12 = 5,931,552 B = 5.66 MiB; × 32 = 15,817,472 B = 15.08 MiB;
× 7 = 3,460,072 B = 3.30 MiB; × 28 = 13,840,288 B = 13.20 MiB; CSR index 297,026 × 4 =
1,188,104 B = 1.13 MiB.

**S19's own warning applies and points the same way:** *"**Do not build it as `Vec` per
(cell, slot).** This is measured, not asserted… 89 % of its 150 MiB heap is empty headers"*
(S19 § 6). A per-face `Vec<[f64; 7]>` at full entry length repeats that mistake at 81 %
emptiness; the parallel-CSR shape is the one `flux.rs` and journal/0100's `FactLedger`
conversion already vindicated.

**The alignment trap.** A second CSR indexed by *cell* does not let a reader get from a
`FluxEntry` to its grain row without a search, because the loaded subset is not the whole
row. The honest cheap form is a **parallel array of the same length as `entries`** (2.59 M),
which costs `2,590,372 × 7 = 17.29 MiB` for `u8` shares — *worse* than filling the padding
and worse than the sparse side array — or a stored `u32` index per entry, which is 9.88 MiB
of pure index. **This alignment cost is the real reason the padding option dominates**, and
it is arithmetic, not taste.

### 4.5 A per-`DepUnit` term instead of per-face

`DepUnit` is **16 B with zero padding** (`stubs.md:914-919`: `tag` 5 + `thickness_m` 8 +
`unconformity` 1 + `chapter` 1 + `species` 1). Any new axis makes it **24 B**.

| basis | units | added residency |
|---|---:|---:|
| live units (stubs #25's own figure) | 5,535,837 × 8 B | **42.23 MiB** |
| capacity slots (S19 § 3(a), *pre*-`shrink_to_fit`) | 9,076,380 × 8 B | 69.25 MiB |

The live-unit figure is the right one **because the `Vec` slack was reclaimed**
(journal/0096:270-271, −54.02 MiB). stubs #25 quotes *"roughly +42 MiB… against a whole-field
residency of ~169 MiB. That is a 25 % residency increase to add one axis"* — against the
**merged-main 108.55 MiB** baseline it is **+38.9 %**, which is worse than the entry stub
states. Flagging rather than editing stubs.md (read-only brief).

**stubs #25 also names the free alternative and it is directly reusable here:** a **packed
byte** — `Litho` has 7 inhabitants (3 bits), `FlowCause` has 7 (3 bits), so `(species,
mover)` fits one byte with two bits spare. The same trick sizes a grain term: **a 3-bit
argmax species + a 3-bit sortedness/dispersion index in one byte** is a *free* per-unit
grain hint if a byte can be found — but `DepUnit` has none, so it lands only inside the
`(species, mover)` repacking slice, *"a representation change across ~38 read sites"*.

### 4.6 The ranking, on arithmetic alone

1. **0 MiB** — the 2 padding bytes in `FluxEntry` (§ 4.2), if 16 bits suffice.
2. **4.43 MiB** (+11 % on the record, +4.1 % on the field) — parallel CSR, `u8` shares.
3. **5.66 MiB** — sparse side array with an explicit `u32` entry index, `u8` shares.
4. **14.3–15.1 MiB** — the same shapes at `f32` precision.
5. **19.76 MiB** — widen `FluxEntry` to 24 B (7 × `u8` + the padding).
6. **42.23 MiB** — a per-`DepUnit` axis.
7. **69.17 MiB** — 7 × `f32` on every entry.

For orientation against S19 § 5(a): the record crosses **1× the old T (162.57 MiB)** at
4.30 % face sparsity for a 16 B atom; today it sits at **8.386 %** with a 16 B atom and
costs 25 % of the pre-shrink field / **37 %** of merged main. Doubling the atom to 32 B puts
it at roughly **1.97×** its current 40.66 MiB — i.e. **~74 % of the merged-main field**. That
is the number the design conversation should hold in view when anyone proposes exact f32
shares per face.

---

## 5. Summary table

| candidate term | exists today? | ratification status | literature anchor | cheapest cost shape |
|---|---|---|---|---|
| **`face_flux`** (magnitude) | **BUILT** — `FluxEntry::magnitude` (`flux.rs:390`), per chapter, per directed half-face | **RATIFIED 2026-07-25** (flow.md § 2); pairing **CAUTIOUSLY RATIFIED 2026-07-29** mode 1 | none applicable — units are cell-epochs of seeded source, **no SI bridge** (§ 3.2) | **shipped, 0** |
| **`load_budget`** (bulk) | **BUILT** — `FluxEntry::load` (`flux.rs:403`); 494,296 entries carry it, 1,864.8 m total | **RATIFIED** same event; caption says *"bulk only"* with a heir that has since landed (§ 8.2) | sediment yield / denudation bands, **pending P2** (§ 3.5) | **shipped, 0** |
| **`grain_distribution`** — as **provenance shares** (7 × `Litho`) | **computed, discarded** at `erosion.rs:2962→2978` | family **RATIFIED 2026-07-24**; sorting mechanics **PROPOSED-strong, user-endorsed**; the *record axis* **not proposed anywhere** | Wentworth φ (§ 3.1) — ⚠ roster spans only φ 1.7…8.0, two points, one collision, one inversion (§ 1.6) | 2 padding bytes if quantized to ≤16 bits; else **4.43 MiB** parallel CSR (`u8`) |
| **`grain_distribution`** — as a **real grain-size distribution** | **does not exist** — no independent grain axis, no abrasion property (§ 1.5) | nothing; the only ask is `earth-processes.md:398`'s *"grain continuum"* (design-doc intent) | Wentworth (§ 3.1) + Sternberg α **0.013–0.018 km⁻¹** (§ 3.3) | **a MODELLING change** — unpriced, because the axis does not exist to size |
| **`mobility_hint`** — as **confinement** (χ = A·S², or its `p`) | **computed, discarded** at `erosion.rs:612`; χ percentiles measured (journal/0113:236) | index is **cited literature** (Montgomery & Dietrich); thresholds are **STUB #26**; the *record axis* is **one clause** of `flow.md:565` | migration bands 0.05–29 m/yr (§ 3.4) — but **saturated over a 62.5 Myr chapter**, so only the *confinement* reading survives | 1 padding byte (quantized χ), **0 MiB** |
| **`mobility_hint`** — as **avulsion / wander** | **already derivable** — `out_face_count` (`flux.rs:507`), 417,981 `(cell, chapter)` pairs measured | ratified as an instrument (`flux.rs` § 2.6 / journal/0113) | same as above | **0** — S-2 says derive it, don't store it (§ 1.4) |
| **`mobility_hint`** — as **energy** (`cap`) | **computed, discarded**; only a 3-valued `EnergyBand` survives, inside `DepTag` | `EnergyBand` **BUILT and ratified** as a facies axis | stream power / unit stream power bands — ⚠ **no SI bridge** (§ 3.2) | 1 padding byte (a finer quantized band), **0 MiB** |
| *(context)* **mover attribution** | **not recorded** — one argmax over both movers | **STUB #25**, heir named | — | packed `(species, mover)` byte, **0** *inside* the repack slice |

---

## 6. Open hazards the design conversation must not miss

### 6.1 The WINDOW axis — terms aggregate per chapter, and not all terms may

`FluxAccum` sums into a per-chapter dense scratch and flushes at chapter boundaries
(`flux.rs:746-777`, `:822-825`). `magnitude` and `load` are **chapter integrals**, and
`FluxEntry::magnitude`'s doc says so explicitly: *"It is a *chapter integral*, not a rate:
that is what a mass budget wants"* (`flux.rs:384-389`).

- **A grain distribution is mass-additive** — summing per-species metres over 25 epochs is
  the same operation as summing the bulk, so it inherits the window with no new semantics.
  *One caveat:* if shares are stored **normalised**, the normalisation must happen at flush,
  never per epoch, or the chapter's composition becomes an unweighted mean of epoch
  compositions rather than the composition of the chapter's mass.
- **A mobility hint is NOT mass-additive.** Averaging a wander statistic over 25 epochs is
  precisely the erasure `SimulDivergence` exists to warn about: *"Whether two faces carried
  flux **in the same epoch**… is a property of the *solve*, and it is erased by the
  aggregation before any reader sees the archive"* (`flux.rs:443-449`). Any mobility term
  that is a max, a count, a variance or an entropy over epochs is **window-dependent in a
  way the existing terms are not**, and `flow.md` § 11.1 is the ratified rule that such a
  window must be **declared, not assumed** — *"a window that decides an acceptance number
  must be declared"*. The WINDOW axis is **Sequenced, not built** (`flow.md:757-758`).

### 6.2 Wire discipline

`postcard` is positional: **no `skip_serializing_if`** (corrections #3, CLAUDE.md
§ Conventions). Every additive axis in this tree has landed by being **appended last** and
**defaulted through every constructor** — `DepUnit::chapter` (`recorder.rs:237-239`),
`DepUnit::species` (`:271-273`), `DepTag::eolian` (`:166-170`), `DeepConfig::mfd_chi_hi`
(`grid.rs:483`). A new term must do the same. Note the second half of that discipline: an
axis whose only inhabited value is the default **keeps the record byte-identical and keeps
the merge key intact** (`Biofacies::Mineral`'s trick, `recorder.rs:52-56`). A grain term
that is *"the bulk load's composition, which is `litho_of_tag` when unsorted"* can be added
byte-identically the same way — and **must** be, because the goldens are tripwires and an
unexplained hash move is a defect to chase (CLAUDE.md § *the testing world is a scratch pad*).

### 6.3 Law 2 / S-3 — do not record a verdict

`is_channel(χ)` (`erosion.rs:324`) is a **boolean verdict**. `refinement.md` Law 2 forbids
*"a categorical tag expressed across a 460 m span"*, and S-3 says a categorical answer is
the argmax **of** the sample, never a field stored beside it. So:

- Recording **χ** (continuous) is S-3-safe. Recording **`is_channel`** is the stored-verdict
  defect `CoarseField` exists to forbid.
- The same test applies on the grain side: recording the **shares** is safe; recording *"this
  face carried gravel"* is not. `DepUnit::species` is already an argmax and already carries a
  stub for exactly this reason (#25) — **do not add a second argmax next to it** (A-4).

### 6.4 Determinism and purity

- All entropy from caller-owned seeds; no wall clock, no ambient randomness
  (CLAUDE.md § Conventions). A quantization of χ or of shares must be a **pure function** of
  the value, not a dithered draw, or the record stops being reproducible in `(seed, extent)`.
- `split_by_shares` (`erosion.rs:482-498`) is the **single** place bulk becomes named
  quantities, and journal/0110 *"had to state the anti-leak rule twice"* — three callers now
  route through it so *"no caller can invent its own budget."* A face-level grain record must
  read the shares that function produced, **never re-derive them from the face total** — the
  hazard is spelled out at `erosion.rs:2930-2949` and it is the same defect twice.
- Refinement operators are **pure fns of (record, shared face data, position)**
  (`flow.md` § 4 / `refinement.md` § 2.1). A grain term on a **directed half-face** is
  automatically shared-by-construction (`flux.rs:81-86`); a grain term on a **`DepUnit`** is
  not — columns do not align across a face (`flux.rs:92-108`), so a unit-level term is only
  legal to read via the three-mode pairing rule, mode 1.

### 6.5 The two aggregations are at different granularities and answer different questions

*What moved through* (face, per chapter) vs *what settled* (unit, per stratum slot).
`refinement.md` § 7.1 asks for **both** (*"grain distribution … and the column inventory"*).
The design conversation should decide which question the term named
`grain_distribution` answers — because the cheap option (§ 4.2's padding) can only serve the
**face**, and the **unit** option is the 42 MiB row.

### 6.6 The gate cannot see the probes' captions

`flux_record_probe` / `flow_cost_probe` / `colluvium_probe` print claims the gate cannot
check, and CLAUDE.md's rule is explicit: *"A printed caption is a published claim the gate
cannot check… When a slice fills a hole a probe narrates, the caption is part of the diff."*
`flux.rs:399-402` and journal/0096:300-304 both narrate the load-composition hole. **A slice
that fills it owes those captions in the same commit**, and the probes are `test = true`
targets so their assertions will run.

### 6.7 The channel operator rides mode 1 and must say so

`flow.md` § 11.5's banner: *"**Mode 1 is built (`flux.rs`, lateral free flux) and the fluvial
member rides it exclusively, stated loudly.**"* Any term added to `FluxEntry` inherits that
pairing; any term added to `DepUnit` inherits `slot_for_chapter`'s `None` semantics
(*"that chapter's unit is gone — erosion stripped it"*, `flux.rs:665-670`), which is a real
tri-state a term's reader must handle.

### 6.8 Existence is not standing, applied here

`RiverSeg` / `carve_rivers` / `BANK` / `RIVER_REACH` are the retiring exemplar
(`refinement.md:56-58`, `flow.md` § 6) and their riverbed flag is *"discarded at both call
sites"*. Nothing in this audit is an argument to preserve any of it, and no term should be
sized to feed it.

---

## 7. What this audit could not verify — flagged, not smoothed

1. **`FluxEntry`'s 2 spare padding bytes** (§ 4.2) — layout arithmetic over `#[repr(C)]`;
   **no `offset_of!` was run**. High confidence, but confirm in code before designing on it.
2. **The 2,590,372 entry count** (§ 4.1) — measured 2026-07-25, **before** continuation (a)
   populated the vertical faces and before journal/0124 moved them. Every § 4 figure is
   therefore a **lower bound**. Re-measure with `flow_cost_probe`.
3. **Whether the `load > 0` fraction (19.08 %) still holds** — same reason; the vertical
   faces added entries that carry **zero** load by construction (`flux.rs:394-397`), so the
   fraction has probably **fallen**, making § 4.4 cheaper than stated. Direction known, size
   not.
4. **The Hjulström minimum-velocity figure (~0.2 m/s at 0.2–0.5 mm)** — this audit's recall
   of the standard curve, not re-verified against the 1935 paper. The Shields band and the
   Sternberg / migration numbers **are** from cited sources (§ 3.2–3.4).
5. **Whether any `Litho` share vector is *already* reachable at the face in a live probe** —
   `Erosion::load_species()` (`erosion.rs:1809`) and `deposited_species()` (`:1801`) are
   public accessors, but this audit did not trace whether any example currently reads them
   per face. If one does, the recording slice has a free instrument.
6. **The mean regolith / denudation figures in § 3.5** are quoted from `field.rs`'s doc
   comment and journal/0111/0114; **not** re-measured here.
7. **`DepUnit`'s zero-padding claim** — taken from stubs #25's arithmetic
   (5 + 8 + 1 + 1 + 1 = 16); consistent with S19's `size_of::<DepUnit>() = 16 B` and with the
   assertion stubs #25 says exists, but not independently checked by `offset_of!`.

---

## 8. Corpus defects found in passing (reported, not fixed — read-only brief)

### 8.1 `stubs.md` #26's opening numbers contradict its own body **and** the code

`stubs.md:945-946` opens: *"`DeepConfig::mfd_chi_lo = 1e-4` / `mfd_chi_hi = 1e-2` are the
ends of the ramp…"* — but the same entry later defends *"the shipped `1.2×10⁻¹`"*
(`stubs.md:968`), and the code is `mfd_chi_lo: 3.0e-2, mfd_chi_hi: 1.2e-1`
(`grid.rs:588-589`, matching `MfdParams::default()` at `erosion.rs:364-365`). Three values,
two of them in one entry. This is the doc-topology shape: **a claim refuted in its own
neighbourhood.** The `1e-4/1e-2` pair looks like a pre-`chi_hi`-sweep draft that the
sweep's conclusion never went back and corrected.

### 8.2 `FluxEntry::load`'s "named seam whose heir is Movement 2b" caption is now stale

`flux.rs:399-402` says the composition's heir is *"**Movement 2b, material-aware
transport**"*. Movement 2b shipped in two slices (journal/0110 on 2026-07-26, journal/0112
same day) and the composition **is** computed — it is simply not recorded (§ 1.2). The
caption still reads as though the mechanism does not exist. This is exactly CLAUDE.md's
*"a printed caption is a published claim the gate cannot check"* applied to a doc comment, and
the fluvial member's recording slice is its natural owner.

### 8.3 `lithology.rs:638-639`'s peat figures are correct; noting it because § 1.5 re-derived them

*"peat, at 5 mm and 400 kg/m³"* — verified against `mod.rs:642-647`. **No defect**; recorded
so a later reader does not re-derive it. (stubs #23's `settle_energy = 1.414` likewise checks
out.)

---

## Appendix — the χ distribution, for whoever sizes a quantized mobility byte

Over **44,204 draining land cells** on the shipped world, `S` dimensionless
(journal/0113:231-236):

| | p1 | p10 | p50 | p75 | p90 | p99 | max |
|---|---|---|---|---|---|---|---|
| **χ = A·S²** | 6.3e−5 | 2.6e−3 | 2.0e−2 | 4.5e−2 | 1.07e−1 | 3.3e−1 | 5.8e−1 |

Shipped thresholds: `chi_lo = 3.0e-2` (≈ p60), `chi_hi = 1.2e-1` (≈ p91). A log-spaced
`u8` over [6e−5, 6e−1] — four decades in 256 steps — resolves ~1.7 % per step, far finer
than the `p`-ramp's own `.round()` (`erosion.rs:352`). **Both threshold constants are
stub #26** and move under P2, so a quantization must span the range, not the current
percentiles.

---

*Sources for § 3 (literature):*
[Szabó et al. 2013, JGR-ES — Williams River abrasion model](https://agupubs.onlinelibrary.wiley.com/doi/full/10.1002/jgrf.20142) ·
[Trends of grain sizes on gravel bars in the Rio Chagres, Geomorphology](https://www.sciencedirect.com/science/article/abs/pii/S0169555X06001796) ·
[Miller et al. 2014, JGR-ES — abrasion vs selective transport](https://agupubs.onlinelibrary.wiley.com/doi/full/10.1002/2014JF003156) ·
[Downstream variation of grain size in gravel rivers: abrasion versus selective sorting](https://link.springer.com/chapter/10.1007/BFb0011201) ·
[The gravel-sand transition and grain size gap in river bed sediments, Earth-Sci. Rev.](https://www.sciencedirect.com/science/article/abs/pii/S0012825221003391) ·
[Lamb et al. 2008, JGR-ES — is θ_c slope-dependent?](https://agupubs.onlinelibrary.wiley.com/doi/full/10.1029/2007JF000831) ·
[Bunte et al. 2013, WRR — critical Shields values in coarse-bedded steep streams](https://agupubs.onlinelibrary.wiley.com/doi/full/10.1002/2012WR012672) ·
[Pace of global river meandering influenced by fluvial sediment supply, EPSL 2024](https://www.sciencedirect.com/science/article/pii/S0012821X24001079) ·
[Controls on lateral channel-migration rate of braided systems, ESPL](https://onlinelibrary.wiley.com/doi/full/10.1002/esp.4710) ·
[High variability in flood discharge and stage accelerates river mobility, Sci. Adv.](https://www.science.org/doi/10.1126/sciadv.adv7637)
