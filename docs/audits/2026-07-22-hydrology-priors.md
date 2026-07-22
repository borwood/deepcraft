# Hydrology priors — exhaustive corpus sweep (2026-07-21)

Read-only sweep of every `.md` in the repo (101 files), plus Rust doc-comments
carrying design reasoning. Status vocabulary used below:

- **DECIDED-by-user** — a dated user ratification. Do not re-litigate.
- **measured** — a spike/probe number.
- **doc-assertion** — written in a design doc, not user-ratified, not measured.
- **sketch** — ideas.md / unratified proposal / notebook capture.
- **code-only** — exists in source, not in any doc as a decision.

**The single most important orientation fact: `docs/design/water.md` is already
a field notebook with its own priors sweep (§ "PRIORS ALREADY IN THE CORPUS",
swept 2026-07-20) plus a § "Session capture, 2026-07-21".** Almost everything
below either lives there or is reachable from it. This sweep's value-add is
(a) the things NOT in water.md, (b) the contradictions, (c) the built/designed/
sketched separation, and (d) the exact verbatim wording of the user-ratified items.

---

## Q1. Is surface water and subsurface water ONE system, or two?

### The framing is ALREADY RATIFIED as one quantity — but with a load-bearing caveat

| # | Claim | Where | Status |
|---|---|---|---|
| 1.1 | **"Water is ONE conserved quantity existing in two regimes, not two systems."** User: *"one quantity, two regimes — that's quite right."* | `water.md` § DECIDED 2026-07-20 (user), L124–141 | **DECIDED-by-user** |
| 1.2 | The regimes: **bound water** (pores + empty eighths of loose partials; moves by settling/permeability; = groundwater) and **free water** (open space; moves by flow; = rivers/lakes/waterfalls/sea) | `water.md` L128–133 | DECIDED-by-user |
| 1.3 | The argument that carried it: *"every phenomenon named in the notebook is a **transition between the regimes**, not a behaviour of either one. Absorbing into the next porous layer is free→bound. A spring is bound→free. Dripping from a ceiling is bound→free at low rate into a void. Waterlogged debris and quicksand are bound at saturation."* | `water.md` L135–141 | DECIDED-by-user (verbatim rationale) |
| 1.4 | **CAVEAT that qualifies "one system": the two regimes have different LOCALITY, and that difference is architectural, not cosmetic.** Bound water is a bounded relaxation (derivable in a halo); **free water is CONNECTIVITY, which is not local at any radius** — "no halo sees it; re-deriving per frame would mean flood-filling the world." | `water.md` § "The hypothesis S11 tests" L188–195; `journal/0028` L21–24 | doc-assertion → **measured** by S11 |
| 1.5 | Consequently: **persist BODIES, derive VOXELS** for free water; **derive** bound water within a halo. Storage rule: *"store only what the derivation cannot predict."* | `water.md` L197–212; `S11-results.md` § Recommendation | measured (GO) |
| 1.6 | **Conservation is the invariant to protect** — "whatever the two regimes use for representation, the transition between them must neither create nor destroy water." Natural test target: S10's mass ledger shape. | `water.md` L165–168 | DECIDED-by-user (consequence 5) |
| 1.7 | S11 measured that invariant holding in the bound regime: conservation drift **2.4 × 10⁻⁷** over 64 steps; and free-water volume conserved **exactly** in the lake-into-cave breach (7 776 in, 7 776 out). | `S11-results.md` § Q1 invariants, § scenario 1 | measured |

### The transport half of the user's framing — where the corpus already commits

| # | Claim | Where | Status |
|---|---|---|---|
| 1.8 | **The user already said the eroding/depositing part explicitly, in the notebook**: *"in the deepsim time, one imagines water moving through porous limestone eating it away and depositing it elsewhere"* — captured verbatim, and unpacked by the integrator as *"dissolution + re-precipitation as ONE transport process — i.e. the karst conduit and the speleothem are the same mechanism read at two ends."* | `water.md` § second pass L83–86, unpack L108–112 | user words (captured), integrator unpack = doc-assertion |
| 1.9 | **"Karst is stream-power's chemical twin"**: the erode–transport–deposit architecture of `erosion.rs` in the solute phase (capacity = solubility × flow, not slope × discharge). Speleothem/tufa is the deposit end of the same transport. Mass ledger extends: `Δ(ΣR+ΣH) == uplift + biotic − solution export`. | `water.md` § Session capture 2026-07-21 | **PROPOSAL** (explicitly "nothing below is DECIDED"); user judged the session's analysis "likely sound" |
| 1.10 | The deep-time engine **already is** an explicit erode→entrain→transport→deposit mass-conserving chain down the D8 receiver chain. Every metre leaving a cell becomes suspended flux; whatever the flow cannot carry deposits downstream; whatever reaches a sink deposits there. Falsifier test asserts `Δ(ΣR+ΣH) = N·Σuplift`. | `crates/dc-worldgen/src/deeptime/erosion.rs` module header L1–13 | **code-only** (built, production path, on by default) |
| 1.11 | **Weathering, not incision, is the rate-limiting term** — and it is explicitly framed as *"the sum of every agent's attack on rock that has not yet moved; today that sum has one term (mechanical), and when the dissolution agent lands it becomes a sum over agents… **Karst arrives by adding a term to the phase this fix already couples.**"* | `corrections.md` #17 (2026-07-20) | measured + **the named integration point for dissolution** |
| 1.12 | **Erosion resistance is agent-specific, never a scalar** — `LithoResistance` carries one axis per agent (Abrasion / Dissolution / FrostIce / Wave / Eolian). Dissolution reads `MaterialProps::solubility` (inverse). The `Agent` enum is deliberately NOT `non_exhaustive` so adding an agent is a compile error at every site. | `deeptime/lithology.rs` header; `earth-processes.md` § 3; `geology.md`; ROADMAP Shipped 2026-07-20 | **DECIDED (journal/0029) + code-only, live in production** (`full_agents` ON since 0047) |
| 1.13 | **Dissolution is the ONE agent still dormant.** "only the dissolution (karst) agent remains designed-but-unbuilt." | `erosion.rs` header; `lithology.rs` agent table | code-only, explicit |
| 1.14 | **Every material in today's roster has `solubility = 0.0`** — "none of them dissolve, so every entry is honestly `0.0`. The carbonate/evaporite milestone is what fills this column in." | `crates/dc-core/src/materials/mod.rs` L200–218 | code-only (hard gate on karst) |
| 1.15 | earth-processes § 8 is literally **"Groundwater, karst, hydrothermal"**: dissolution caves on water tables; mineral-charged fluids depositing veins/ore along fractures and contacts. Sim = water-table interplay + vein/inclusion emplacement via pore partials + fracture networks from deformation history. | `earth-processes.md` § 8 | doc-assertion (engine roster entry) |
| 1.16 | The **present tier needs zero cave-specific code**: "A cave is an air component in the connectivity index; a flooded conduit is a body; a spring is a body outlet where the table meets the surface (derived, not authored). Conduit-vs-matrix flow — the thing that would wreck a naive saturation field — **is exactly the free/bound two-regime split**, rediscovered by karst hydrology's own conduit/matrix distinction." | `water.md` § Session capture 2026-07-21 | PROPOSAL |
| 1.17 | **Fluid is a generalized material state, not a water special case.** It fills loose partials AND structure pores. "Settling is the fluid's own rule and explicitly NOT the solids' rule" — user: *"fluid obeys different rules than mere weight."* Settling defined as *"attempting to leave one block and occupy spaces below"*; two expressions: dripping from a ceiling, absorbing into the next porous layer. | `water.md` § second pass (user verbatim) | user words, captured; not formally DECIDED |
| 1.18 | **Ore genesis already assumes lateral fluid transport as the honest heir** — redbed copper "sits exactly at the recorded vertical contact — a stated coarsening, not a fake"; heir = *"a real fluid-flow/diagenesis pass would move ore laterally along aquifers."* | `ores.md` § 8 table, redbed row | doc-assertion (heir named) |
| 1.19 | Bog iron heir: *"a future groundwater/seep sim would refine within-wetland placement."* | `ores.md` § 8 table | doc-assertion |

**Verdict on Q1: the corpus commits to ONE quantity, and the 2026-07-21 session
capture already proposes ONE transport system (karst = stream power's chemical
twin, deposit end = speleothem). What is NOT decided is the representation of
each regime and whether the deep-time transport chain and the present-tier
body/saturation machinery are one implementation or two coupled ones.** See the
contradictions section for the locality wedge that constrains this.

---

## Q2. Cementation as deposition-in-pores

**This is the thinnest area of the corpus. There is no cementation model, and
almost no prior naming "cement" at all.** What exists:

| # | Claim | Where | Status |
|---|---|---|---|
| 2.1 | **The compaction loop is already the stated deep-time↔gameplay unifier**: *"Compaction closes the deep-time loop: debris under overburden, over ledger time, migrates into a structure slot as sedimentary stone. Worldgen strata, gameplay middens, and geology are one process at different tick rates."* | `materials.md` § stratification, L72–74 | doc-assertion (2026-07-18), **never built** |
| 2.2 | The stated mechanism is **loose eighth → structure slot under overburden × time**. Note: it says nothing about a cementing *substance* — it is a phase change of the same material. | same | doc-assertion |
| 2.3 | earth-processes § 5 "Burial, diagenesis, metamorphism": *"per-column burial-depth history from the epoch stack → P/T path → class transform (the deep-time compaction loop, materials.md)."* | `earth-processes.md` § 5 | doc-assertion (sketch-level engine entry) |
| 2.4 | **Pore packability rule — DECIDED 2026-07-20 (user).** `filler_grain_size ≤ K_PORE × host_grain_size`, `K_PORE ≈ 0.25` (derived: ideal-packing interstices 0.22–0.41 D; geotechnical filter criterion D/4–D/5). Heterogeneous host → **minimum** grain among structural components sets the throat. Enforcement in one shared `fits_in_pores(filler, host)` helper **consulted by every *transport-time* depositing process**; `VoxelContents` stays pure volume accounting. | `materials.md` § Pore packability | **DECIDED-by-user** |
| 2.5 | **The exemption is the load-bearing bit for cementation**: *"formation-context emplacement (worldgen authored — magmatic/**diagenetic** inclusion, e.g. olivine in basalt) bypasses the mechanical rule: those crystals grew in place; the pore is representational (3d decision). **The rule governs infiltration, not genesis.**"* | `materials.md` § Pore packability, Exemption | **DECIDED-by-user** — and it explicitly names *diagenetic* inclusion as exempt |
| 2.6 | Code confirms the split: the packing check "never consults [the rule] at formation… governs infiltration, not formation." | `crates/dc-core/src/materials/geology.rs` L641–645 | code-only |
| 2.7 | Composability invariant: **sieve resistance ≡ grain size** (registry invariant, tested), so what-packs-in and what-sieves-out-first are the same axis by construction. | `materials.md`; `dc-core/materials/mod.rs` L191–195 | DECIDED + code |
| 2.8 | `fits_in_pores` conservative fall-throughs: empty host structure → `false` (no grains = no throat); zero grain sizes cannot occur (registry-asserted). | `crates/dc-core/src/materials/packing.rs` L44–60 | code-only |
| 2.9 | **Cementation is named exactly twice in the whole corpus, both in passing.** (a) corrections #16: *"Cohesion is the correct modifier for the **wave** agent (sea cliffs fail along joints, **a cementation question**) and the wrong one for fluvial abrasion."* (b) ideas.md § soil is loose but packable: soil transitions loose→structural "under weight + time". | `corrections.md` #16; `ideas.md` | measured (a); sketch (b) |
| 2.10 | Freeze–thaw with water in pores is filed as an OPEN QUESTION, not a decision: *"Freeze–thaw: water packed in pores + cold → cracking/spalling (delicious, deferred)."* | `materials.md` § Open questions | sketch |
| 2.11 | Also open in the same list: *"Pore size model: single scalar from structure material + porosity, or per-material pore spectra?"* and *"does **active** sorting (alluvial deposition) get its own fast path near water?"* | `materials.md` § Open questions | sketch |
| 2.12 | **Root-lattice soil model — DECIDED 2026-07-21 (user).** "Anything with roots carries a root material in STRUCTURE form — porous, holding the soil in its pores. This is an anti-erosion model for free… Soil placements come from simulation, in both forms: layers of structural dirt… with loose layers typically above, and **packing downward into porous rock below**." | `materials.md` § forms design pass, ratification 3 | **DECIDED-by-user** — a structure-holds-fines-in-pores precedent |
| 2.13 | **Packable soil (user, 2026-07-20)**: porosity/permeability differ sharply between loose and packed soil; "most sub-surface soil levels generate already packed", only top horizons genuinely loose. "Connects to the water thread: **porosity/permeability differ sharply between loose and packed soil, so this is upstream of groundwater**." | `ideas.md` § Soil is loose but packable | sketch (user direction) |
| 2.14 | S11 measured the aquitard case: a **sharp** loose-vs-packed permeability contrast makes the water table halo *smaller*, not larger — "the aquitard case the brief specifically flagged as the risk is the **most** local case measured." | `S11-results.md` § Q1 | measured |

**Verdict on Q2: the user's inference (cement = the deposition half of transport,
operating at pore scale) has NO contradicting prior, and two priors point at it —
2.4/2.5 (the packability rule governs infiltration and explicitly exempts
diagenetic genesis, i.e. the rule as written does not block deposition-from-
solution) and 2.1 (compaction already framed as the loose→structure transition).
BUT: nothing in the corpus models a dissolved load, a solute concentration, or a
precipitation event at pore scale. The one thing that would carry it — `solubility`
— exists on the property sheet and is 0.0 for every material shipped.**

---

## Q3. Rain and the water cycle

| # | Claim | Where | Status |
|---|---|---|---|
| 3.1 | **`precip` is a normalized 0–1 scalar, not a mass or a volume.** `march()` doc: *"Writes normalized precip (0..1) into `grid.precip`."* The plane is precipitation, not residual humidity (orogeny's corrected lesson), re-marched on *current* topography every ~20 iterations. | `crates/dc-worldgen/src/deeptime/climate.rs` L1–11, L46–48 | **code-only, definitive** |
| 3.2 | Pregen hydrology: priority-flood depression filling seeded from the ocean, steepest-descent flow on filled elevations, **discharge accumulated down the flow forest**, river edges above a threshold. Discharge is a flow-accumulation count, not a water volume. | `S7-results.md` § 3 Hydrology | measured/built |
| 3.3 | **There is NO evaporation term anywhere in the sim.** The only occurrences of "evaporation" in the corpus are (a) ores.md's evaporite *narrative* mechanism, (b) water.md's integrator observation. | corpus-wide grep | genuine absence |
| 3.4 | **The user's own stance on conservation (2026-07-20), verbatim**: *"i appreciate the fight for water volume conservation and present history. at outset i didn't believe it actually possible even if it's obviously ideal. … i anticipate the answer is probably 'where we compromise' and shades of procedural tricks. it's going to be lossy / not fully conservative of volume - **neither is reality (the water cycle is half gaseous)** - and ultimately players get to exploit the gaps in the sim, and this won't be the only one. may come with the territory. shouldn't stop us from thinking up genius architectures, but still. and we'll use every perf hack in the book, which goes for the whole game."* | `water.md` § Stance on conservation | **user words, recorded as project stance** |
| 3.5 | Recorded as project stance from 3.4: perfect volume conservation is **the ideal, not the requirement**; players exploiting gaps comes with the territory; **every perf hack in the book is sanctioned**; **but the compromise is the LAST step, not the first**. | `water.md` same section | project stance (user-derived) |
| 3.6 | **The evaporation-as-honest-sink idea is a PROPOSAL, explicitly not ratified**: *"the user's gaseous aside may be load-bearing rather than rhetorical… If the liquid model carries an honest evaporation term, then 'lossy' stops being a compromise and becomes 'conserved with a modelled sink' — a much stronger position, and it gives the slop a principled home rather than scattering it through the implementation."* Notes S10's biology already demands water, so a consumer waits. **"Worth deciding WHERE the loss lives."** | `water.md` § Stance on conservation, PROPOSAL | **PROPOSAL, unratified** |
| 3.7 | Same shape re-proposed for the solute path: *"dissolved load reaching the sea is a real sink, same epistemic status as the evaporation observation."* | `water.md` § Session capture 2026-07-21 | PROPOSAL |
| 3.8 | **Waterlogging is deliberately modelled SEPARATELY from rainfall** (S10 design choice 8): climate moisture + bonuses for sitting near base level and for receiving upslope drainage. *"A wet mountainside sheds water and grows forest; a low flat site collects it and grows peat. This is what puts coal swamps on lowlands."* | `S10-results.md` § design choice 8; `journal/0026` | measured/built (production) |
| 3.9 | **That proxy has a DEFINED RETIREMENT**: "waterlogging becomes 'the water table is at or near the surface here', read from the field. **Biology reads the real quantity; the proxy is deleted.** Two systems privately approximating one physical quantity is the thing to avoid." | `water.md` § consequence 4 | **DECIDED-by-user** (consequence of the two-regimes ratification) |
| 3.10 | **Soil depth already stopped being a rainfall function** — carry-`H` (journal/0053) made both the soil band and the clastic veneer budget read the recorded regolith plane. Finding: **`Σ(recorded unit thicknesses) ≡ H` exactly.** | `geology.md` § Expression of the ledger, holdout (c); `journal/0053` | shipped 2026-07-21 |
| 3.11 | **A live thing-that-will-happen already asserts the water-balance framing**: *"A land can be a desert in every way that matters to its rocks and its life, yet look green from a hill — because 'arid' is a verdict about **water balance**, not a colour."* | `things-that-will-happen.md` | doc-assertion (design target) |
| 3.12 | Climate is chaptered/re-marched, **not** a static field: the orographic march re-runs every ~20 iterations on current topography; paleo-sea-level is a deterministic sinusoid (stub #9, amp 35 m / period 50). | `climate.rs`; `stubs.md` § 9 | code + stub with named heir (epoch-indexed pregen curves) |
| 3.13 | **Cost priors that cut in favour of more simulation**: *"World-generation time is not a design constraint we optimize against by default."* User: "we'll likely take much longer." Mitigation is ready-made worlds, not a cheaper ritual. U3 relaxed the ritual ceiling to *"5 min if that's what it takes."* | `ecology.md` § S10 note; `ideas.md` § entering a world without generating one; `tectonics.md` U3 | **DECIDED-by-user** (standing rule) |
| 3.14 | **Counter-cost prior**: the eolian strata record already costs **+92.76 MB at Medium** (DeepField 52.8 → 145.5 MB), and it is filed as an undiagnosed memory bill. Any new per-epoch recorded plane (e.g. a paleo water table, paleo-channels) inherits that class of cost. | ROADMAP § Observed; `journal/0047` | measured |
| 3.15 | Weather is committed as a feature ("when, not if") and couples to the materials sim (snow/sand deposition, wetness/porosity). Wetness is a *render* channel, sim-driven, not yet wired. | `visuals.md`; `PIPELINE.md` § 5 | doc-assertion; `fog_params.z = weather wetness` reserved |

**Verdict on Q3: `precip` is a normalized 0–1 climate scalar (3.1) and there is
no volumetric water cycle anywhere. The corpus contains NO decision ruling a mass
cycle in or out. The user has pre-conceded lossiness (3.4/3.5) and the integrator
has proposed — unratified — that evaporation be the principled home for the loss
(3.6). Cost arguments run both ways: gen time is explicitly not a constraint
(3.13), but per-epoch recorded planes have a measured memory bill (3.14).**

---

## Q4. Caves

### The standing architectural commitment

| # | Claim | Where | Status |
|---|---|---|---|
| 4.1 | **"Depth is a worldgen axis: deep-time geological history generates literal strata; caves/aquifers/lava at region scale, not per-column noise hacks."** | `ARCHITECTURE.md` § World structure, DECIDED 2026-07-18 | **DECIDED** — caves-from-process is the standing commitment; today's noise caves are the placeholder it already disowns |
| 4.2 | Caves are to be **the record of water, not noise.** Four families = four different *agents*: **karst** (dissolution of soluble rock), **littoral** (sea caves, wave energy at a coastline), **erosional** (mechanical carving, undercut channels, abandoned conduits), **glacial** (ice as agent). Each is (agent × rock × time), the same shape geology.md uses for rock formation. "This **replaces** today's caves." | `water.md` § 1 (user verbatim: *"most important things to anticipate: water -> caves. karst, littoral, erosional, glacial."*) | user words, captured; families not yet sequenced |
| 4.3 | **Groundwater ↔ CAVES flagged by the user** — speleogenesis as the eventual cave story. Ratified 2026-07-19 as part of the water design pass scope. | `water.md` banner; ROADMAP § Water-model design pass | DECIDED-by-user (scope) |

### The orogeny-proven karst recipe — FOUND, quoted verbatim

> **"Caves read solubility from the recorded volume (conduits live inside
> coalesced carbonate bodies on a subdued, perched water table) — solubility is
> a **volume property, not a surface map**. Determinism hygiene: 'judge the whole
> volume at the same scrambled position.'"**
> — `docs/design/orogeny-recon-2026-07-19.md` § Filling the 3D world, L124–127

Cross-referenced and re-quoted at `water.md` § PRIORS L360–365 and
`earth-processes.md` § Quarry results ("caves reading solubility from the recorded
volume" is in the steal-directly list, RATIFIED 2026-07-19).

Mechanism decomposed:
1. **Solubility is read from the recorded volume**, not painted on a surface map —
   i.e. the strata record is the input, and the same 3D volume is judged
   everywhere rather than a heightfield-derived proxy.
2. **Conduits live inside coalesced carbonate bodies** — dissolution is confined
   to soluble lithology that the record says is contiguous.
3. **On a subdued, perched water table** — the table is a low-relief surface
   perched above the regional base, and conduits form at it.
4. **Determinism hygiene: judge the whole volume at the same scrambled position** —
   one addressed draw per volume, not per sample point, so the body is coherent.

Adjacent orogeny lessons in the same doc that bear on caves: the cost philosophy
**"coarsen the cause, never delete it and fake the appearance"** (RATIFIED
2026-07-19); the bounded/unbounded theorem (erosion bounded, drainage global);
"decay-length halos beat propagation-speed halos ~4×".

### The rest of the cave corpus

| # | Claim | Where | Status |
|---|---|---|---|
| 4.4 | **geology.md already sequences it**: *"Next after v1: chemical sediment (carbonate) — **caves-in-carbonate-on-water-tables is orogeny-proven and gameplay-rich**"*; candidate passes include **karst** and **glacial (later)**. Filed open question: "caves/water-table interplay (aquifers in porous stone)". | `geology.md` § v1 content; § erodibility | DECIDED (sequence), open question filed |
| 4.5 | **Vadose vs phreatic falls out for free, and with it cave morphology.** "Below the water table is phreatic (saturated — where dissolution happens, the orogeny-proven karst regime). Above it is vadose (air-filled — where dripping, flowstone and speleothems happen). The same conduit changes character when the table drops past it… **No cave-morphology system is needed; it is the regime boundary moving through rock over time.**" | `water.md` § consequence 2 | **DECIDED-by-user** (consequence of the two-regimes ratification) |
| 4.6 | The **cheapest-first-family proposal**: erosional caves — deep time already computes drainage every epoch, so paleo-channels exist "in the record"; an abandoned conduit is a former channel the water table later dropped below. *"Possibly derivable from data already held."* | `water.md` § What survived the sweep, PROPOSAL | PROPOSAL |
| 4.7 | **BUT the deep sim's per-epoch drainage is COMPUTED AND DISCARDED** — `DeepField` keeps only `surf` + `strata` (plus, post-U8, the *final* drainage export). Paleo-channels need a **recorder axis** (channels, and the table per chapter). Cost unmeasured; estimate hundreds of KB; the eolian record already carries a memory FLAG, so measure first. | `water.md` § Session capture finding 2 | PROPOSAL / work-shaped finding |
| 4.8 | **Collapse-tier handoff proposal**: recorded conduit capacity refines (bounded, addressed, scrambled-position judged) to **void intervals per column**; the collapse-time water table decides which voids are flooded at year zero — "the genesis handoff that seeds the body graph." | `water.md` § Session capture | PROPOSAL |
| 4.9 | **Column-model verdict (proposal)**: don't kill `ColumnRec` — kill the contract's growth. A per-column **interval log** is fully general for 3D solids (caves = void intervals beside deposition events; orogeny field-proved the shape at 147k columns). What must stop is new consumers baking in "one height, solid below". Proposed one-paragraph ARCHITECTURE decision. **User-owned scope call, NOT made.** | `water.md` § Session capture finding 4 | **PROPOSAL awaiting user** |
| 4.10 | ARCHITECTURE already generalizes in that direction: *"a column's fill is an **ordered list of spans**, each carrying (contents, form, fractional occupancy). Today's single-height, solid-below-surface column is the degenerate case of that list."* And it names "the caves/water thread's voids and occupants" as one of the four threads that forced the fill contract. | `ARCHITECTURE.md` § The fill contract, DECIDED 2026-07-21 | **DECIDED-by-user** |
| 4.11 | **Today's caves are S1 noise carving in the LEGACY client generator only.** `CAVE_THRESHOLD = 0.58`, 3D OpenSimplex at ~20 m features, in `dc-client/src/worldgen.rs` — the legacy S1 `TerrainGen` on keys 3/4, NOT the worldgen authority. **The production worldgen path emits no caves at all.** | `crates/dc-client/src/worldgen.rs`; ARCHITECTURE § One world-answer surface | code-only |
| 4.12 | **Cave rendering/LOD**: FF2b (coarse volumetric summaries) is explicitly **paired with "the caves/underground thread of the water design pass"** — "when overhangs exist, the summary goes 3D; couples to S3 region storage." FF2a left the extension point ready (`ColumnSpan` payload is one of a potential stack). | ROADMAP § Sequenced FF2b; `visuals.md` L91–96 | Sequenced, unbuilt |
| 4.13 | Unasked question, flagged: **bulk-flow octrees may share substrate with FF2b's volumetric summary octrees** (SVDAG/Aokana candidate). "Worth checking before either is built." | `water.md` § What survived the sweep | PROPOSAL |
| 4.14 | S3 measured a real cave-LOD cost: **~2/3 of far-mesh triangles were sealed cave surfaces** in the volumetric shell. Resolved for the worldgen far field only by making it a top-surface heightfield — i.e. **the current far field cannot represent caves by construction.** | ROADMAP Observed; `S3-results.md` § 241 | measured |
| 4.15 | **Sim light**: caves being lightless *in the simulation* is a named unlock; "optimistic sky" would light caves as if open to sky until data arrives — **the lighting spike must decide re-light-on-load** (S3 OQ 6). | `light.md` §§ 8–9; ROADMAP Sequenced | designed, spike owed |
| 4.16 | **Caves get their own ecology for free** — "no light, stable temperature, wet, fed from outside" is a set of conditions like any other, so the ecology sim diagnoses cave communities once light is an axis. **No cave-specific ecology system.** | `light.md` § 7; `visuals.md` L284–288 (user: "if it happens on earth we give it a…") | doc-assertion, user-adjacent |
| 4.17 | Cave acoustics fall out of per-voxel density/porosity — "cave acoustics from actual geometry, sound deadened by packed-earth walls." | `ideas.md` § Sound | sketch |
| 4.18 | Cave-ins / structural integrity from structure density/porosity — mining engineering, conservative rubble. Behind the realism-knob doctrine. | `ideas.md` § Structure & danger | sketch |
| 4.19 | Two `things-that-will-happen` lines are cave acceptance tests: *"You find a cave that used to be underwater: dissolved smooth below the old water table, dry now, growing flowstone from the drips still coming through."* and *"The sun low on the horizon shines into a cave mouth… and the simulation knows it does, not just the shader."* | `things-that-will-happen.md` | design target |
| 4.20 | **Acceptance test proposed for the first karst slice**: *"the encounter is a stream that vanishes into a sink and a spring that returns it downstream — make **that** read-quality the test of the slice, not void statistics."* Notes it is "the thing 460 m coarsening most threatens." | `water.md` § Session capture | PROPOSAL |
| 4.21 | **Which cave family ships first is an explicit open question** (#6): "Karst is orogeny-proven but **carbonate-gated**; erosional may be nearly free; littoral needs wave energy; glacial needs ice as an agent." | `water.md` § Open questions | open |
| 4.22 | Glacial is explicitly sequenced LAST — "cause 4" of dismal mountains, "we have no cryosphere", geology.md sequences glacial as "later". | ROADMAP Observed § dismal mountains; `geology.md` | doc-assertion |
| 4.23 | Littoral has a live blocker and a named heir: the wave magnitudes are **NOT ratified** (station 5 null: 0.68 m at the world's most-attacked coast); the retune was **STRUCK by the user 2026-07-21**: *"That whole mechanism changes after water machinery. that would be a bandaid… can revisit later."* Heir: **wave energy as a fact about the water body — fetch from S11's body graph × the 0037 wind field.** | ROADMAP Sequenced; `earth-processes.md` closing block; `journal/0049` | **DECIDED-by-user (struck)** — wave expression is now a *consumer of the water design pass* |

---

## Q5. Springs, seeps, and water exiting the ground

| # | Claim | Where | Status |
|---|---|---|---|
| 5.1 | **"A spring is bound→free"** — named as one of the four canonical regime transitions in the argument that carried the one-quantity ratification. | `water.md` § DECIDED 2026-07-20, rationale | **DECIDED-by-user** (as part of the rationale) |
| 5.2 | **"A spring is a body outlet where the table meets the surface (derived, not authored)."** | `water.md` § Session capture 2026-07-21, present tier | PROPOSAL |
| 5.3 | **The water table is pinned by the drainage network — "streams, springs and coastlines every few hundred metres."** This is not decorative: it is the boundary condition without which the S11 halo measurement is meaningless (corrections #15). The halo numbers cluster near `L/2` where `L` is drainage/seepage spacing. | `S11-results.md` § Q1 boundary condition; `corrections.md` #15 | **measured** — springs are load-bearing for the locality result |
| 5.4 | S11's Q1 harness models seepage columns literally: a lattice of seepage columns at spacing `L`; the perturbation is *"one new seepage column — a dug shaft."* | `S11-results.md` § Q1 method | measured (spike code, `water/sat.rs`) |
| 5.5 | **FLAG 7 (S11, unresolved):** *"`Pinned` bodies take their level from outside and the spike simply sets it. Where that level actually comes from (deep-time sea level, a river reach's stage, **a spring's discharge**) is unspecified and is a real design question the notebook has not reached."* | `S11-results.md` § Design choices where water.md was silent | **explicitly open** |
| 5.6 | Conduit capture "is what produces losing streams, dry valleys over cave systems, **springs at base level**; without the feedback you get rivers flowing intact over networks that should have swallowed them." | `water.md` § Session capture, deep time | PROPOSAL |
| 5.7 | **Dripping from a ceiling is bound→free at low rate into a void** — the other named exit path, and the vadose/speleothem mechanism. | `water.md` § DECIDED rationale; § consequence 2 | DECIDED-by-user (as rationale) |
| 5.8 | Ore-genesis narratives already assume seeps as a *readable* signature: bog iron's read is *"bogs, **rusty seeps**, orange-slicked still water"*; the mechanism is *"Groundwater moving through soil under vegetation picks up iron as reduced, soluble Fe²⁺. Where that water **surfaces** at a wetland margin…"* | `ores.md` § 1 bog iron; § 8 table | doc-assertion (ratified roster item, mechanism narrative) |
| 5.9 | Lode gold's narrative is a hydrothermal exit too: *"the expelled water carries dissolved silica and metals up through the fracture network; it precipitates as quartz veins where pressure and temperature drop."* | `ores.md` § 1 orogenic gold | doc-assertion |
| 5.10 | The karst acceptance test (4.20) IS a spring test: "a stream that vanishes into a sink and **a spring that returns it downstream**." | `water.md` § Session capture | PROPOSAL |

**Verdict on Q5: springs are already load-bearing in three separate places
(the regime-transition rationale, the S11 boundary condition, the karst acceptance
test) but there is no design for one. The single named open question is 5.5 —
where a pinned body's level comes from.**

---

## Q6. BUILT vs DESIGNED vs SKETCHED

### BUILT — code exists

| What | Where | On the production path? |
|---|---|---|
| **Two-plane erosion + mass-conserving stream-power transport** (uplift → priority-flood → D8 → drainage-area accumulation → entrain/incise/deposit → weathering → hillslope diffusion), mass ledger `Δ(ΣR+ΣH)=N·Σuplift` asserted | `deeptime/erosion.rs` | **YES**, always-on A tier since 3e-1 |
| **Drainage re-routed every iteration** (priority-flood + steepest descent) — rivers already migrate inside the sim | `deeptime/erosion.rs`; corrections #20 | **YES** |
| **Pregen hydrology**: priority-flood fill seeded from ocean, steepest-descent routing, discharge accumulation, river edges above threshold; "rivers reach the sea" proven by test | `pregen/hydrology.rs`; `S7-results.md` | **YES** |
| **Rivers carved at collapse from `RiverSeg` chords** — straight cell-to-cell chords, width ∝ √discharge, elevation blended down toward a water surface, `riverbed` bare-surface flag. **Computed on the PRE-erosion pregen surface** | `collapse.rs::carve_rivers`, `RiverSeg`, `RIVER_REACH=40+BANK` | **YES** — and this is the stale second hydrology (corrections #20) |
| **Drainage export on `DeepField`** (`recv`/`area`/`lake`/`exhum`/`t_crust`/chapter table); Σ area over sinks = n exactly; 100 416 exported lakes | S12, `journal/0044` | **YES** (populated); **consumed by nothing** |
| **Orographic precipitation march**, re-marched every ~20 iterations on current topography; normalized 0–1 | `deeptime/climate.rs` | **YES** |
| **Zonal circulation**: signed zonal wind magnitude through zero at band boundaries + subsidence aridity | `journal/0037` | **YES** |
| **Paleo-sea-level sinusoid** (amp 35 m, period 50) driving every unit's subsea/subaerial tag | `deeptime/grid.rs::sea_level_at` | **YES** — stubs.md § 9 |
| **Wave (littoral) erosion agent** — cutting at the current sea stand, mass-neutral | `erosion.rs::wave`, `wave_erosion` 0.05, `wave_band_m` 30.0 | **YES** since the `full_agents` flip; **magnitudes UNRATIFIED**, retune STRUCK |
| **Frost/ice agent** — temperature-gated weathering multiplier peaking near 0 °C | `erosion.rs::periglacial` | **YES**; magnitudes RATIFIED 2026-07-21 |
| **Eolian agent** — deflation + downwind loess/dune deposition, mass-neutral | `erosion.rs::wind` | **YES**; magnitudes RATIFIED |
| **Agent-specific lithic resistance incl. a populated-but-dormant `Dissolution` axis** | `deeptime/lithology.rs` | **YES** (dissolution dormant) |
| **`MaterialProps::solubility`** on the property sheet | `dc-core/materials/mod.rs` | **YES** — **all values are 0.0** |
| **S10 waterlogging proxy** (climate moisture + base-level + upslope-drainage bonuses) driving peat/coal | `deeptime/biotic.rs`; S10 | **YES** — retirement already decided |
| **Flood disturbance** in the biotic layer — drainage-area-gated succession reset, deliberately **no distinct tag** (the overbank mineral band is its signature) | S10 § process 6 | **YES** |
| **Pore/eighths occupancy model** (structure / debris / pore roles; fluids occupy empty eighths and pores) | `dc-core/materials/contents.rs` | **YES** |
| **Occupancy primitives naming fluid fill as consumer #1**: `free_eighths`, `open_pores`, `free_debris_eighths`, `loose_eighths`, `bound_eighths`, `is_occupancy_solid`, `SOLID_EIGHTHS=4` | `contents.rs` L235–290 | **YES** (journal/0052); *collision still reads `Block::is_solid`* |
| **`fits_in_pores` / `K_PORE`** packability helper | `dc-core/materials/packing.rs` | **YES** |
| **S11 water spike code**: `water/{mod,vox,sat,conn,body}.rs` — saturation relaxation, water table *read* not stored, per-chunk air-component labelling + coarse union-find connectivity index, body graph with `Finite`/`Pinned`, commutative-monotone events + deterministic fixpoint | `crates/dc-worldgen/src/water/` | **NO — additive and standalone; nothing in production calls it** |
| **S1 noise caves** (`CAVE_THRESHOLD` 0.58, 3D OpenSimplex) | `dc-client/src/worldgen.rs` | **NO — legacy S1 `TerrainGen`, keys 3/4 only** |

**Not built, notably: there is no `Block::Water`, no water `MaterialId`, no ice,
no limestone/carbonate, no `LOAM` registered in any class.** The block enum ends
at `CarbonaceousMudstone = 11`. The material roster (25 ids) has sand, gravel,
snow, leaf litter, clay, silt, potsherd, knapping debris, ash, loam, scree, bone,
mudstone, sandstone, granite, basalt, gold dust, siltstone, conglomerate, diorite,
andesite, olivine, peat, coal, carbonaceous mudstone. **No fluid exists in the
world at any tier.** Rivers are dry carved channels; the sea is a datum
(`SEA_LEVEL_M = 0`, genesis stub) below which `clastic_pass` returns early.

### DESIGNED — DECIDED / ratified

1. **One quantity, two regimes** (2026-07-20, user) + its five consequences:
   water table is READ not modelled; vadose/phreatic falls out (no cave-morphology
   system needed); aquifer/aquitard are material facts; S10's proxy has a defined
   retirement; conservation is the invariant.
2. **S11 ratification calls (2026-07-20, user)**:
   - **`Pinned` default ON, and TOGGLEABLE.** User: *"i guess pinned but we have to be able to turn it off and see what happens. simple as. default on."*
   - **The ~12-cell halo is a KNOB, not a constant.** User doctrine, stated generally: ***"where there's a cell range, there's a knob. we don't know what may be more perf in the future when other unbuilt systems are also running. knob now."***
   - Calls 3 (body identity across merges) and 4 (index persistence) **RECLASSIFIED as engineering, not user calls** — ride as-built. *(Process note recorded: they should never have been put to the user.)*
3. **Water-model design pass ratified 2026-07-19** (user): groundwater as "another dimension for the flow to go"; water table/aquifers; ponds and sub-resolution water; visible/flowing water; lakes/inland seas as deep-tier flooded basins with known spill levels; **Groundwater ↔ CAVES flagged**. Design doc before any code.
4. **Pore packability + `K_PORE`** (2026-07-20, user), with the genesis exemption.
5. **Caves/aquifers/lava at region scale, not per-column noise hacks** (ARCHITECTURE, 2026-07-18).
6. **The fill contract** (2026-07-21, user): contents authoritative, `classify(contents)->Block`, `block == classify(contents)`; a column is an ordered list of spans; solidity moves onto an occupancy threshold; **water fill is one of the four consumers that carried the decision**.
7. **Fractions come only from the ledger** (2026-07-21, user) — recorded quantity/variance, never cosmetic noise.
8. **Root-lattice soil model** (2026-07-21, user).
9. **Erosion-agent roster flipped ON** (2026-07-21, user) — wind + frost + wave in every world; the seven magnitudes were then partly ratified (wind/frost) and wave explicitly NOT.
10. **Wave retune STRUCK** (2026-07-21, user): wave expression is a consumer of the water design pass; heir is fetch from the S11 body graph × the wind field.
11. **Expression of the ledger** (2026-07-21, user): everything recorded must be expressed; unexpressed only where the expresser is unbuilt, loudly temporary; **"Material placement keyed on present-day slope/elevation/temperature is a mock, not a model, wherever the ledger holds the answer."**
12. **Enhancement doctrine** (2026-07-21, user): local procedural refinement is honest iff (a) deterministic function of the recorded field and (b) the local procedures reflect the deep-sim processes — *"in what sense they must agree is an explicitly open design question."*
13. **3e-2 decision 1** (2026-07-19, user): **drainage and contributing area are decided ONCE at the coarse A tier; C inherits them as fixed boundary fields and never re-routes.** Ratified with the river-conditioning mechanism as the excuse; hard constraints: descent-along-flow is a hard refinement constraint, and **the corridor never crosses a drainage divide (no catchment theft)**.
14. **Water rendering directive** (2026-07-21, user, at session start): (a) take full advantage of **partials and structure**; (b) **placeholder texture is appropriate**; (c) **leave the seams** *"for us to do more interesting things with rendering it such as textures showing direction of flow, waves, etc."*
15. **Water look** (visuals.md): semi-realistic MC-shader tradition — animated surface, depth fog, shore blending; pixelated dynamic detail, not smooth sprays; *"Water placement is physical (aquifers, porous stone) and the look should honor that."* PIPELINE reserves a **`water` stage (5)**: translucent pass, animated surface, depth fog, shore blend; water writes depth for `volumetrics`.
16. **Renderer light and sim light are separate**, neither derived from the other; **light attenuating with depth in water is a free unlock** (photic zone) if water is a material occupying pores and partials.
17. **Content-pack completeness bar**: user, listing water features — *"features: galciars, lakes, lagoons, oceans, inland seas, waterfalls, etc etc. **literally whatever you encounter on earth**."* The default pack's completeness bar is Earth itself.

### SKETCHED — ideas / unratified proposals / notebook captures

- **Bulk flow as octrees of continuous-flow volumes** (user): *"creates no new blocks as long as its directional outlet connects to receiving flow volumes"*; **our water is mobile**, and *"if continuously fed, immobile + a producer of new blocks or a sustainer of existing bulk flows"*; **event driven, not polled**. Explicitly NOT decided by S11.
- **Capillary action** — a water partial travelling along a ceiling or side face before stochastically dropping. User flags it "flashy" and names the cost honestly: *"would imply a whole other state for partial fluids."* Filed wanted-if-affordable, not scoped.
- **Sub-resolution water** (a trickle, a damp seam, a puddle) — open question 2.
- **Evaporation as an honest sink** (integrator PROPOSAL, unratified).
- **Erosional caves as the cheapest first family** (PROPOSAL).
- **Bulk-flow octrees sharing substrate with FF2b volumetric summaries** (PROPOSAL, unasked).
- **Event-driven flow invalidation = the existing dirty rail with a different payload** (PROPOSAL).
- **Two drainage opinions → deep drainage becomes the spine** (PROPOSAL, with a history-pass resequencing consequence).
- **Paleo-channel + per-chapter water-table recorder axis** (PROPOSAL, cost unmeasured).
- **Column-as-interval-log target contract** (PROPOSAL, **user-owned scope call, unmade**).
- **Water-table placement: likely BOTH** a coarse paleo table per chapter in the deep record AND a present-tier locale relaxation over the strata's permeability (PROPOSAL; "needs the spike, not an assumption").
- **S14 spike spec — "can drainage refine under coarse boundary conditions?"** — **DRAFTED 2026-07-21, NOT dispatched, NOT ratified.** Six measurement groups, decision rule stated in advance, sequencing note that it is downstream of the two-drainage-opinions decision.
- **Soil is loose but packable** (ideas.md, user) — upstream of groundwater.
- **Loose materials spread when dropped** (granularity + x + fall height).
- **Uncollapsed history frontier**, incl. the user's counter that even global flow can be frontier-hacked: *"we don't know where all this water came from at this edge, but it does imply there's higher terrain that way which hasn't been collapsed yet."*
- **Freeze–thaw** (water in pores + cold → cracking/spalling), "delicious, deferred".
- **Sinking-in-partials movement rules** (knee-deep snow), deliberate future step; collider stays binary (solid ≥ 4/8) meanwhile.
- **No swimming, buoyancy, drowning, or submerged-movement prior exists anywhere.** S6 explicitly: *"Buoyancy/fluids: nothing here models fluid volumes; rapier has no [buoyancy]."*

---

## Q7. The open questions the corpus itself names

### `water.md` § Open questions (carried, unanswered) — enumerated verbatim in substance

1. **What are the *encounters* — the moments a player meets water that no other game gives them?** *(Asked; the user's answer is PENDING, and the notebook says it should be driven by it.)*
2. **Sub-resolution water** (a trickle, a damp seam, a puddle) — pore occupancy covers water *in* rock, but a film or a rivulet in open space is not obviously an eighth.
3. **Bulk-flow octree vs the per-voxel pore/saturation field**: one system at two scales, or two systems with a boundary? *(Partially pre-answered: pores are settled material-tier machinery; the octree is the open half.)*
4. **Does the near-field flow network persist, or re-derive from the water table on load?** *(Same family as the owed far-field summary persistence.)*
5. **Where does the deep-time water field live relative to the A/C tiers** — given S9 says it *relaxes* and is therefore haloable? *(Sharpened in the Session capture to "likely BOTH".)*
6. **Which cave family ships first, and does it wait on carbonate?** (Karst orogeny-proven but carbonate-gated; erosional may be nearly free; littoral needs wave energy; glacial needs ice as an agent.)

### Explicitly NOT decided by the two-regimes ratification

> *"The **representation** of each regime (the bulk-flow octree, the saturation
> field), the event vocabulary, timescale ownership, sub-resolution water, and
> capillary action all remain open below."*

### S11's ten FLAGs (design choices where water.md was silent) — all still open

1. A body's container is its **whole air component**, not a per-basin depression analysis. Correct 3-D basin decomposition = priority-flood over air cells, out of scope. Consequence: a lake spilling over a lip into a lower part of the *same* cave is not two bodies today.
2. **Bodies in the same component always merge**, lowest id surviving → body identity is not stable across a breach. Named lakes need a separate identity layer.
3. Order-independence achieved by making the event vocabulary **commutative and monotone** — this constrains what any future event may do.
4. The connectivity index is derived and never persisted; **6 ms rebuild for a 1.77 M-voxel world**; at real scale this is a streaming problem, unmeasured.
5. The coarse union-find is **rebuilt wholesale per edit** (0.249 ms) — wants region-scoped rebuild before the coarse graph reaches ~10⁵ nodes.
6. A body's geometric anchor is a **single voxel**; if an edit makes the anchor solid the body is orphaned — not handled.
7. **Where a `Pinned` body's level comes from is unspecified** (deep-time sea level? a river reach's stage? a spring's discharge?) — "a real design question the notebook has not reached."
8. Bound water's head proxy is `H = y + sat`, per-layer — an unconfined approximation, **not a Darcy solver**.
9. **The Q1 halo depends on the drainage-network boundary condition.** If a production world has regions with no drainage within kilometres, the halo there is set by leakage alone — measured 4–5 cells for contrast ≤ 10, **unmeasured for a tight aquitard with no drains at all.**
10. The spike's rock properties are **a small local table, not the real property sheet**. Porosity/permeability are property-sheet facts; wiring them through was out of scope.

Also, per-round `resolve` cost is **O(bodies × links)** — ~2.5 ms at 1 000 bodies; wants a per-body link index before tens of thousands of nodes. **Flagged, not fixed.**

### ROADMAP items naming water

- **§ Sequenced — "S11 follow-through: the water model's four open calls"** (calls 1 and 2 answered 2026-07-20; 3 and 4 reclassified as engineering).
- **§ Sequenced — "Water-model design pass"** (ratified 2026-07-19).
- **§ Sequenced — FF2b** paired with the caves/underground thread.
- **§ Sequenced — wave-magnitude retune STRUCK**; wave is now a consumer of the water pass.
- **§ In flight — "Design passes queued (main-session conversations)": ecology · social sim · the water/caves thread — "water.md § Session capture 2026-07-21 — the two-drainage-opinions finding and the bounded-drainage-refinement spike question are the live items."**
- **§ Observed — "Caves ↔ hydrology integration thread captured"** (nothing decided; four work-shaped findings).
- **§ Observed — unlock ranking (assistant view, unratified)**: (1) persistence — *"three systems have now independently filed 'must persist' as an open question (far-field summaries, **S11's water body graph**, the `MixtureTable`)"*; (2) inventory/items; (3) loose materials; **(4) water present-tier — "S11 says GO."**
- **§ Observed — "Ours is not a deep world right now — you can keep digging into no-variety"** (user field report) — explicitly connects to *"§ 8 groundwater/karst (slated deep hydrology). The vertical dimension is a mostly-unbuilt frontier."*
- **§ Observed — "Rivers are straight cell-chords (S7); course refinement needs the 2-ring argument re-proved at finer levels."**
- **§ Observed — "The 5–460 m band has no process — only decayed noise"**: no gullies, ravines, outcrops, knickpoints, talus, or hillslope-scale stream incision. C-refinement designed for exactly this, unbuilt.
- **§ Observed — "The next arc after planet gen"** (user): *"deposits of loose partial mixes from grinding and **hydrology below surface**."*
- **§ Observed — the eolian record's +92.76 MB memory bill**, adjacent to any new recorded plane.

### Other named-and-open

- **Light § 9 Q3**: *"Whether one relaxation machinery serves light and bound water"* — a design question the light spike must answer or explain away. (Structural parallel: bounded local relaxation attenuated by a per-material property — opacity vs permeability.)
- **materials.md § Open questions**: pore size model (scalar vs per-material spectra); does active alluvial sorting get a fast path near water; freeze–thaw.
- **`ores.md` R-decisions awaiting the user** (several water-dependent): R1 lode gold; R2-B placer presence conditioned on upstream endowment; R3 BIF vs the Phanerozoic register; R4 evaporite members (salt only, or salt + gypsum); **§ 2.4 slice 1 — evaporite deposition needs a real recorded evaporation event in the deep sim** (*"no evaporite without a recorded evaporation event"*).
- **stubs.md § 9** paleo-sea-level sinusoid; **§ 11** placer presence is source-blind.

---

## PRIORS THAT CONTRADICT OR CONSTRAIN THE USER'S FRAMING

**Read this section first if reading nothing else.**

### C1. Drainage is the single ADVECTIVE wall — and it is RATIFIED as decided-once-and-coarse. This is the hardest constraint on "one unified transport system."

S9 measured (2026-07-19): *"**drainage/flow-area is the sole ADVECT**, everything
else in the catalog relaxes at 16–24 cells."* Hillslope decay 21 cells clean;
fluvial spikes to 26 **via drainage reroutes**, with isolated spikes *reappearing*
at d14 and d28. corrections #8 sharpens it: *"only **hillslope** erosion is bounded;
**fluvial** erosion inherits drainage's global reach… C is licensed **solely**
because drainage is decided coarse."*

Then **3e-2 decision 1, RATIFIED 2026-07-19 by the user**: *"drainage and
contributing area are decided ONCE at the coarse A tier; C inherits them as fixed
boundary fields and **never re-routes**."* Hard constraints attached: descent-along-
flow is a hard refinement constraint, and **the corridor never crosses a drainage
divide (no catchment theft)**.

**Why this constrains the framing:** a unified transport model in which underground
conduits capture surface flow (losing streams, dry valleys, conduit capture — the
exact phenomena the Session capture proposes) is a **re-routing of drainage by a
sub-coarse-cell process**. That is precisely what decision 1 forbids at the C tier.
The Session capture is aware and proposes the karst feedback *at the A tier*
("feeds back into flow routing"), which is legal — but it means **karst cannot be
a refinement-tier phenomenon**, and the 460 m cell is then the resolution at which
a cave system captures a river. The Session capture itself flags this: the karst
acceptance test (the sink and the spring) *"is the thing 460 m coarsening most
threatens."*

### C2. The S14 spike that would relax C1 is DRAFTED BUT NOT DISPATCHED, and is explicitly sequenced *behind* a decision nobody has made.

`water.md` § SPIKE SPEC: *"This is a specification awaiting the user's go-ahead,
not a decision and not scheduled work."* And: *"This spike is downstream of a
decision it should NOT prejudge: the two-drainage-opinions finding… **Settle the
spine question first or the spike measures the wrong field.**"*

Its stated decision rule, fixed in advance: a halo in the S9/S11 family plus a low
divide-instability rate ⇒ rivers become refined terrain and `RiverSeg` retires.
Otherwise ⇒ **the carve survives as a named refinement OPERATOR, never a persisted
primitive.** And the named failure mode to hunt: *"refinement moving a divide such
that the refined interior disagrees with the coarse boundary it was handed."*

### C3. `corrections.md` #14 already falsified the graph shape the user's "carry on as a surface spring" mental model implies.

**Claim falsified:** free water is a graph of bodies carrying volume, level,
inlets, outlets, with links between them. **Measured:** links were **0 or 1** in
every one of seven scenarios. *"In a voxel world **air connectivity already carries
the relation**. Two bodies in the same air component are not linked, they are the
same body… A kilometre-long channel joined to a river is not 'a link on the river's
body' as the notebook sketched; **it is the river**."*

**Consequence for the framing:** "water exiting underground and carrying on as a
surface spring" is, in the S11 model, **not a link** — if the conduit and the
hillside are in the same air component it is one body; if they are not, it is one
of the rare genuine links. The transport-continuity intuition is right physically
and *architecturally invisible* — which is a feature, not a problem, but it means
the spring is a **derived outlet condition**, not an authored graph edge.

### C4. `corrections.md` #15 constrains any claim about groundwater locality: the number only exists because of the drainage-network boundary condition.

*"A relaxation measurement is only meaningful against the boundary condition the
real system has — get that wrong and the method still produces a clean,
reproducible, entirely fictional decay curve."* And S11 FLAG 9: the halo is
**unmeasured for a tight aquitard with no drains at all**. So "bound water is
local" is conditional on springs/streams existing every few hundred metres — which
is exactly the thing the design has not built.

### C5. `corrections.md` #16 and #17 constrain how a dissolution agent must be wired.

#17: **the rate-limiting term on hillslopes is bedrock→regolith weathering, not
incision or entrainment.** *"Karst arrives by adding a term to the phase this fix
already couples."* So dissolution belongs in `weather_cell`, composed with the
lithic and biotic multipliers — **not** as a new incision term. #16: a
plausible-sounding modifier can be **anti-correlated** with the contrast you want;
measure direction before adding it.

### C6. The one-number-erodibility trap is a *closed* decision that karst design must not reopen.

`lithology.rs`: *"There is no value of one number that gives both, so a one-number
model does not merely approximate karst badly — **it forecloses it**."* And the
`Agent` enum is deliberately not `non_exhaustive` so a new agent is a compile error
at every site. **This is the corpus's explicit protection of the karst path; any
unified-transport design must arrive as `Agent::Dissolution` reading
`LithoResistance::dissolution`, not as a bypass.**

### C7. Karst is HARD-GATED on carbonate, which does not exist.

Every material in the roster has `solubility = 0.0` (code comment says so
explicitly). There is no limestone, no carbonate class member, and the carbonate
milestone is sequenced but unscheduled (`geology.md`: "Carbonate follows";
ROADMAP 3d). **Open question 6 names this as the reason karst may not ship first.**

### C8. There is no water in the world at any tier — the unification has nothing to unify yet.

No `Block::Water`, no water/ice `MaterialId`, no fluid role in `VoxelContents`
beyond capacity accounting. Rivers are **dry carved channels** (elevation blended
down, a `riverbed` bare-surface flag). The sea is a datum below which `clastic_pass`
returns early. S11's code is *"additive and standalone… nothing in the production
world-generation path calls it, deep time is untouched, and there is no renderer
work."*

### C9. Two drainage opinions already exist, and resolving them has a pipeline-surgery consequence.

corrections #20: *"the rivers **carved into the world** at collapse come from
**pregen** `Cell.river`/`flow_to`/`discharge`, computed on the PRE-erosion coarse
surface — a second, older hydrology painted over terrain the deep sim has since
reshaped."* The Session capture escalates it: *"the same disease as two water
tables, one level up."* And the consequence with teeth: **the history pass sites
settlements against pregen rivers** (order: tectonics → climate → hydrology →
history → deep-time), so making deep drainage the spine *"would need resequencing
after deep time."* Flagged as **pipeline surgery — decide deliberately, not
mid-karst-slice.**

### C10. The user has already pre-conceded lossiness — so an evaporation/mass-cycle argument must not be sold as a correctness requirement.

3.4 verbatim. The corpus stance is: honest mechanism first, compromise last, every
perf hack sanctioned. The evaporation-as-sink idea is an *unratified integrator
proposal*; presenting it as settled would be re-litigating a user stance.

### C11. Wave/littoral is deliberately BLOCKED on the water pass — a bandaid was struck.

User 2026-07-21: *"That whole mechanism changes after water machinery. that would
be a bandaid, against our standing rule against bandaids."* So the water design
pass now **owes** littoral its heir (fetch from body size/shape/depth × the wind
field). This is a *dependency the pass has acquired*, not an optional extra.

### C12. Any new per-epoch recorded plane has a measured memory precedent against it.

The eolian record cost **+92.76 MB at Medium** (DeepField 52.8 → 145.5 MB) and is
filed as an undiagnosed bill. Paleo-channels and a per-chapter water table are
exactly that shape — the Session capture itself says *"cost unmeasured… measure
first."*

### C13. The far field cannot represent caves, and the near/far boundary is already visibly inconsistent.

The worldgen far field is a **top-surface heightfield** by construction (S3 found
~2/3 of far-mesh triangles were sealed cave surfaces in the old volumetric shell).
Two live Observed items — "the far field is now BOXIER than the near field" and
"the far field is block-tier and carries no contents" — mean **any world with
overhangs/caves lands on an LOD system that has no answer for them.** FF2b is the
named heir and is unbuilt.

### C14. The 0.9 m voxel sieve is a hard constraint on cave and conduit expression.

The charcoal precedent: mean bed 3.5 cm, **0 of 158 310 survived**. The
distribution-first rule (journal/0055) fixed *quantization order* and moved the
drop threshold to ~5.6 cm — but a conduit narrower than a voxel still cannot be a
void. `ores.md` § 5's sieve table is the worked precedent for how a sub-voxel
feature must change **form** rather than be deleted.

### C15. Order-independence is a hard, structural constraint on any new water mechanism.

Three independent proofs in the corpus (S9b's scatter→gather, S11's commutative-
monotone events + fixpoint, light's max-plus relaxation) plus the orogeny hygiene
rule *"judge the whole volume at the same scrambled position."* S11 FLAG 3 states
it as a live constraint: **any new event must be expressible as a monotone
mutation, or the property is lost.** Classic error diffusion is explicitly
forbidden. All entropy must be addressed (seed + world position).

### C16. "No bandaid tuning" and "loudly temporary" bind this pass.

`geology.md` § Expression of the ledger: everything the ledger holds must be
expressed; inexpressed is permitted **only** when the expresser is unbuilt, and
then **loudly**. Combined with the enhancement doctrine's open question — *"in what
sense [local and deep processes] must agree is an explicitly open design question"*
— any hydrology enhancement (a spring, a puddle, a drip) must be a deterministic
function of the recorded field, not field-blind detail.

---

## WHAT THE CORPUS DOES NOT CONTAIN

Genuine blank space. Nothing below has a prior of any kind — no decision, no
sketch, no measurement, no code.

1. **Any fluid in the world.** No water block, no water material, no ice, no
   snowpack-as-water, no fluid role beyond capacity accounting.
2. **Any solute / dissolved-load quantity.** `solubility` exists as a rock
   property; there is no representation of *how much of what* is in solution, no
   saturation-index, no concentration field, no transport of dissolved mass.
3. **Cementation as a process.** No cement supply, no precipitation-in-pores
   mechanism, no diagenetic pore-filling event, no porosity-reduction-over-time
   term. (2.1's compaction loop is the closest, and it is a loose→structure phase
   change with no cementing substance, designed 2026-07-18 and never built.)
4. **Any water mass, volume, or flux with physical units.** `precip` is 0–1;
   `discharge` is a flow-accumulation count; there is no mm/yr, no m³/s, no
   storage term.
5. **Evaporation, transpiration, condensation, humidity as a state variable, or
   any closed cycle.** (Evapotranspiration is *implicit* in S10's moisture
   tolerances; that is the whole of it.)
6. **Snow/ice as water.** Snow is a loose material with a property sheet; there is
   no melt, no snowpack water equivalent, no glacier, no permafrost, no cryosphere.
   Frost exists only as a weathering *multiplier*.
7. **Swimming, buoyancy, drowning, submerged movement, or fluid drag.** S6:
   *"nothing here models fluid volumes."*
8. **Any water renderer.** PIPELINE reserves a `water` stage at format ≥ 1; the
   hook does not exist. PBR-2 (which owns "water") is user-DEFERRED.
9. **Speleothem / flowstone / travertine / tufa as a material or a process.**
   Named twice in prose (things-that-will-happen; the Session capture) and nowhere
   else.
10. **Sinkholes, dolines, dry valleys, losing streams, resurgences** — named only
    inside the Session capture PROPOSAL.
11. **Aquifer confinement (confined vs unconfined), artesian pressure, or Darcy
    flow.** S11 FLAG 8 explicitly disclaims: *"an unconfined-aquifer approximation
    adequate for a locality measurement; it is not a Darcy solver and should not be
    quoted as one."*
12. **Water temperature, thermal springs, or hydrothermal fluid as a modelled
    thing.** earth-processes § 8 names hydrothermal; ores.md narrates it; nothing
    models it.
13. **Water chemistry.** No pH, no redox as a field (redbed copper and bog iron use
    *record adjacency* as a proxy for redox), no brine concentration (evaporites
    need a deep-sim slice that does not exist).
14. **Erosion or deposition BY groundwater.** The five erosion agents are abrasion,
    dissolution (dormant), frost/ice, wave, eolian. **There is no subsurface
    transport agent of any kind** — the user's "carry the eroded material to
    wherever, maybe exiting underground" has no existing agent slot.
15. **Sub-resolution water in open space** (film, rivulet, damp seam, puddle) —
    named as open question 2, never designed.
16. **Any answer to "what are the encounters"** — open question 1, and the user's
    answer is explicitly PENDING and the notebook says it should drive the pass.
17. **Ocean currents, tides, waves as water motion** (wave *energy* is an erosion
    constant with a named heir; there is no wave as a moving surface).
18. **Ice as an erosion agent / glacial caves.** Explicitly last in every sequence.
19. **Where a `Pinned` body's level comes from** (S11 FLAG 7).
20. **Persistence of anything water-related.** The body graph is one of three
    systems that independently filed "must persist" against a persistence layer
    that does not exist.
