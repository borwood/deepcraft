# S3 — Materials & Content (baseline doc-topology sweep, 2026-07-28)

> ## 📋 DISPOSITION — applied 2026-07-29.
>
> | # | state |
> |---|---|
> | **1** biotic shipped-ON vs `worldgen.md` sequencing | **✅ ALREADY FIXED before this pass** — `ecology.md` now carries the reconciliation and credits this sweep: *"It is a seam with a named heir, not a system under development… the missing word was **building**."* No action taken. |
> | **2** `materials.md` § AS BUILT describes `block_twin` in the present tense | **✅ APPLIED** — pointer added: the mechanism was deleted 2026-07-23 (journal/0087); read the paragraph for its **argument**, not its API. |
> | **3** RATE bullet still says "composed with topo-sort" | **✅ APPLIED** — struck (the last un-struck `topo-sort` in that section, 23 lines below its own SUPERSEDED banner, reported two days earlier and unfixed). `flow.md:459`'s *"ORDER (topo-sort)"*, flagged here for another slice, is **also fixed**. corrections #65's site list amended. |
> | **4** `S16` names two spikes | **⚠ ESCALATED** — the finding says *"User/integrator call; I am not resolving it."* Renaming the water-side reference (id-free) vs renumbering a live id cited ~1,130 times is a real trade-off. |
> | **5** lode gold's heir 2 = a calibration `stubs.md` records as blocked | **❌ FINDING OVERTAKEN — see the note below the table.** |
> | **6** formation predicates: plain data (DECIDED) vs functions (open edge) | **✅ PARTIAL** — the **reciprocity gap is closed** (§ 14 now points at the notebook's § 2.4 and states the available reconciliation, marked *not settled*). **The recommended `🔖 OPEN EDGE` banner on a DECIDED clause is ⚠ ESCALATED** — changing a decided section's status is not a sweeper's call. |
> | **7** ecology.md's 25 s ratification | **✅ ALREADY FIXED before this pass** — `ecology.md` carries *"⚠ And the cost quoted below is the withdrawn number… measured at 13.79 s."* No action taken. |
> | **8** `ores.md` § 8 reports a doc-hygiene defect since fixed | **✅ APPLIED** — struck; `geology.md`'s annotation verified present. |
> | **9** roster tables vs R8's member naming | **✅ PARTIAL** — the **one-directional pointer is fixed**: § 2.1's table now carries a ⚠ naming R8, the substance-vs-deposit conflict, and the redone texture pass. **The amendment itself remains ⚠ ESCALATED — it is explicitly the user's call.** |
>
> **On finding 5 — the audit was overtaken by a user ruling it could not have seen.** It reads
> the metre-scale-exhumation caveat as a live blocker whose shape changed. **On 2026-07-28 the
> user rejected the caveat outright** (`journal/corrections.md` **#70**): *"We do not need to
> have ore 'exposed' — the default plugin pack will ship a voxel game **with digging**…
> Absolutely no reason to treat it like everything needs to be discoverable on the surface."*
> `ores.md` gained a top-of-file 🔴 banner the same day. **What this pass did:** struck the
> four caveat sites the banner named and left unstruck (§ 1 flagship warning, § 2.1 heir 2,
> § 6 R1's *"probe 3 measures this first"*, § 8.1's collision), each pointing at #70. **The
> same correction supersedes S8's contradiction C1**, which recommended surfacing the caveat
> to the user as a blocked ratification — it is not blocked; the premise is dead.

**Watermark commit: `f652b60`.** Every `file:line` below was read at `f652b60`; a quotation
without a revision is not a quotation (skill rule, corrections #67).

**Read in full (3,338 lines):** `docs/design/material-behavior.md` (1036) ·
`materials.md` (528) · `ores.md` (618) · `ecology.md` (375) ·
`material-genesis-notebook.md` (337) · `octree-substrate.md` (233) · `worldgen.md` (211).
Plus, as the other side of pairs: `CLAUDE.md`, `north-star.md` in full; targeted whole
sections of `stubs.md` (§§ 24–27), `ARCHITECTURE.md` (fill contract), `geology.md`
(§ v1 content), `S10-results.md` (banner), `water.md:756`, and the
`2026-07-26-doc-topology-sweep.md` findings table (to separate repeats from new).

**Not opened:** `flow.md`, `spines.md` § 3, `ROADMAP.md`, `journal/*`, `visuals.md`,
`bodies.md`, `biomes-ecology-agenda.md`, `earth-processes.md`. Several pairs below have a
plausible third side in those files; where that is so, it is said.

**I did not resolve any pair.** Which side wins is a user call in at least #1, #5 and #6.

---

## 1. Findings — 9 pairs, ranked by blast radius

### 🔴 1. Is the biotic layer SHIPPED AND ON, or is ecology gated behind work that has not begun?

| | |
|---|---|
| **A** | `docs/design/ecology.md:262-263` — *"**GO on the biotic layer.** `production_config`'s `biotic` flag is flipped ON; biology is a **shipped part of world generation, not an experiment**."* (DECIDED 2026-07-20, user) — reinforced at `ecology.md:290` *"**`production_config`'s `biotic` is ON.** Every new world runs the six processes over deep time"*, and at `ecology.md:249-250` *"**Biology is measurably an erosion term**… adding an organism pack would change terrain."* |
| **B** | `docs/design/worldgen.md:189-191` (DECIDED 2026-07-28, user) — *"**The ordering is firm:** engine + non-bio earth science, in the ratified plugin shape → **ecology** → **social concepts**… Ecology precedes anything social; nothing social is considered before it."* preceded by `worldgen.md:184-187` — *"We have a lot of engine work and non-bio default-modpack work to do **prior to considering ecology**… much work and reflection will be done before the **USER** decides it is time."* |
| **C** | `CLAUDE.md` § Conventions — *"we have not moved on to bio/evo/socia/civ… just open edge gestures so far"*, and *"we do NOT have any form of evo/socia/civ modeling **even at the design stage**"*. |

**Both are user-originated — this is not a provenance tie-break, it is two ratifications
that have never been put side by side.** A is 2026-07-20 and is **confirmed live in code**:
`crates/dc-worldgen/src/deeptime/field.rs:288` reads `biotic: true` inside
`production_config`, with six processes writing coal/peat/soil/charcoal facies that reach
material selection. B is 2026-07-28, eight days newer, and reads as *ecology has not been
taken up*.

**Why this is the top finding and not a quibble.** A reader of `worldgen.md` § *Sequencing*
today concludes there is no biology in the world; a reader of `ecology.md` concludes biology
ships, is on by default, and **moves rock**. The reconciliation that is probably intended —
*"the S10 biotic layer is a deep-time earth-process pass; the ECOLOGY DESIGN PASS (species,
proliferation patterns, evolution) is what is gated"* — **is not written anywhere in either
doc.** `ecology.md`'s own dormancy banner (`ecology.md:3-15`) says the design is dormant and
must be revisited; it does **not** say the shipped biotic layer is exempt from the gate, nor
that it is covered by it.

**The narrowest place the ambiguity bites:** `ecology.md:282-284` and `:328-330` record § 5
fork 2 (**pack-addition blast radius**) as *"Still open, still the user's"* and now shipping
*"in a world where biology really does move rock"* — an open user decision that is live
today under A and unreachable under B.

*Recommendation (labelled as one): a single sentence, ratified, in `worldgen.md`
§ Sequencing saying whether the shipped biotic deep-time pass is inside or outside the
gate. Not a doc edit by a sweeper — B is eight days old and A is in production.*

---

### 🔴 2. `materials.md` § *The fill contract AS BUILT* still describes `block_twin` as the implemented shape

| | |
|---|---|
| **A** | `docs/design/materials.md:310-312` — *"Then **`block_twin(material) -> Block`**, a **material→block table**, not a class→block table. **That relocation is the load-bearing bit**…"* (present tense, under the heading *"### `classify` — implemented"*) |
| **B** | `docs/design/materials.md:498-500` (DECIDED 2026-07-22, user, 188 lines later in the same file) — *"The **fifteen-name `block_twin` match and its `_ => Stone` arm die**; no twin field is ever built (the interim registry-field plan is superseded…)"* |
| **C** | `crates/dc-core/src/classify.rs:24` — *"There is **no `block_twin` re-translation any more**"*, and `docs/spines.md:213` records the deletion as shipped compliance work (journal/0087). |

**Shape 1 + shape 6.** The § AS BUILT section is an engineering record of 2026-07-21 and its
own header says *"Nothing here is a new user ratification"* — but it is written in the
present tense, describes a mechanism that has since been **deleted**, and carries **no
back-pointer**. B names A's mechanism as dying; A holds no link to B. This is the
one-directional-pointer signature the skill measures at 8-of-15 corpus-wide.

**B and C are better supported** (a user decision plus the code). The live question A still
answers, and which does not survive B, is *which paragraph now explains why two members of
one content class summarize to one name* — that argument is only in A.

*Note the same section's neighbours are being maintained:* `materials.md:344-347` was
updated 2026-07-28 for the ruin-post deletion and its reciprocal at
`docs/ARCHITECTURE.md:256-259` was stamped in the same commit. **The block_twin paragraph
forty lines above it was not.**

---

### 🟠 3. `material-behavior.md` § 5's RATE bullet still asserts topo-sorted ORDER — **REPEAT, unfixed since 2026-07-26**

| | |
|---|---|
| **A** | `docs/design/material-behavior.md:369-370` — *"the temporal-resolution knob **topo-sort alone does not give**. This is `ideas.md`'s fractional-phase sketch, reconciled: it is the RATE axis, ***composed with* topo-sort**, never replaced by it."* |
| **B** | `docs/design/material-behavior.md:346-348`, **twenty-three lines above A** — *"**🔴 SUPERSEDED 2026-07-26 (user) — ORDER IS AUTHORED, PER WORLD.** See `ARCHITECTURE.md` § *The engine is plugin-agnostic, and pass ORDER is authored* (DECIDED) and **corrections #65**."* |
| **B′** | `docs/ARCHITECTURE.md:536` — names `material-behavior.md` §5 by name as a site of the falsified claim. |

**Verified as a repeat, not a new find:** this is finding **15** of
`docs/audits/2026-07-26-doc-topology-sweep.md:400-410`, filed two days ago against these
exact lines. The sweep's sibling findings in the same section **were** fixed —
`material-behavior.md:277` now reads *"~~topo-sorted~~ **authored order, validated**"* — so
the ORDER and WINDOW bullets are clean and **only the RATE bullet still carries it**.

`flow.md:459` and `flow.md:718` were the other half of that finding; `flow.md:718` now
carries a strikethrough and `flow.md:459` still reads *"ORDER (topo-sort)"* — outside my
slice, flagged for whichever slice owns `flow.md`.

**B is user-originated and DECIDED. A is an assistant reconciliation** — and it is the
*same* assistant reconciliation corrections #65 was written about. Its survival two days
after being reported is itself the datum.

---

### 🟠 4. `S16` names two different spikes, and `octree-substrate.md` sends a water reader to the weathering one

| | |
|---|---|
| **A** | `docs/design/octree-substrate.md:36-38` — *"**S16 is unwritten** and drafts against an octree **node**, not a per-deep-cell summary (`water.md` § consequence for S16)."* (§ 0 Priors, presented as current state) — reciprocated at `docs/design/water.md:756` *"**Consequence for S16 (unwritten):** draft it against an **octree node**…"* |
| **B** | `docs/spikes/S16-weathering-behavior-shape-results.md` **exists**, and `docs/design/material-behavior.md:537` cites it as current: *"first cellular pass (agent-fold, pure behavior + apply) — `WeatheringPass` (`dc-worldgen/src/deeptime/weather_behavior.rs`, **S16**)"*, with five further live uses at `material-behavior.md:45, 244, 246, 336, 547`. |

**Shape 4, in its stable-id form.** The spike number was taken by a different spike; the
water design's reservation was never renumbered or annotated. A grep for `S16` returns two
unrelated bodies of work, and `docs/spikes/` (the read-first item-5 lane, *"measured
numbers, don't re-guess them"*) resolves it to the weathering one.

**B is better supported** — the artifact exists on disk. A is a reservation that lost its
number. *The pathspec for the absence, stated per the skill's corollary:* `ls docs/spikes/`
at `f652b60` lists `S15-results.md`, `S16-weathering-behavior-shape-results.md`,
`S17-deep-cell-inventory-results.md` — **no water/hydrology spike doc exists at any
number.**

---

### 🟠 5. `ores.md`'s named heir for lode gold is a calibration `stubs.md` records as measured-and-blocked

| | |
|---|---|
| **A** | `docs/design/ores.md:288` — lode gold's heir row: *"**heir 2: erosion-supply calibration** — gives `exhum` **legible dynamic range** (§ 8.1)"*, with `ores.md:143-147` *"**exhumation comes out metre-scale at shipped erosion rates**, so exhumed-core signals are illegible at any amplitude **until the erosion-supply calibration (Sequenced) lands**"* and `ores.md:513-515` repeating it as an unresolved collision. R1's whole recommendation (`ores.md:389-391`) is conditioned on it. |
| **B** | `docs/design/stubs.md:818-823` — *"**(b) The calibration was BUILT, MEASURED and left OFF** — and that half is not discharged, it is **superseded by § 27 and § 29**. The entry said the world should land in **1–10 m/Myr**. **It does not, at `EROSION_CALIBRATION = 45` or at any value**… and **no multiplier reaches the craton band with a world left in it**."* — with `stubs.md:832-834` *"**The incision clamp leaves deep closed depressions at any multiplier above 1×** — 0 pits at 1×, **44 at 5×**… That is the blocker on the flag."* |

**Not a falsification — a heir whose shape changed underneath the doc that names it.**
A's premise (*metre-scale exhumation at shipped erosion rates*) is **still true**, because
`calibrated_rates` shipped **off** and the world still runs at 0.0110 m/Myr. What has
changed is that A promises *"the calibration lands, `exhum` gets range"* as a scheduled
event, and B records that the calibration was run, produced 0.4142 m/Myr (a floor band, not
the craton band), and is held behind a measured defect. **Neither file names the other.**

**B is better supported** (measured, journal/0114, with the probe attached). The user-owned
part is what A's R1 fork should now say — its *"probe 3 measures this first"* gate may
already have its answer.

*Third side likely in `ROADMAP.md`, which still carries* **Erosion-supply calibration**
*as Sequenced at `ROADMAP.md:2438-2443` with the 2026-07-20 framing — not opened in full;
flagged for the ROADMAP slice.*

---

### 🟡 6. Formation predicates: **plain data** (DECIDED) vs **functions** (the live open edge)

| | |
|---|---|
| **A** | `docs/design/material-behavior.md:1014-1017` (§ 14, **DECIDED 2026-07-24**) — *"**Formation predicates are plain data over field-ids**… A predicate is a **conjunction of `(field_id, comparator, range)` conditions** the engine evaluates against local field values — **no code crosses the SDK; the fields are the interface** (crossing constraint met by construction)."* |
| **B** | `docs/design/material-genesis-notebook.md:155-157` (§ 2.4, **PROPOSED**, 2026-07-26) — *"**Terms should stay FUNCTIONS, not a closed formula vocabulary.** A liquidus temperature is pressure-dependent, so the "key" is **a function evaluated at the cell**, then sorted… **avoids committing to a `linear\|exponential` formula enum**."* — resting on `material-genesis-notebook.md:144` *"**ordering key** — scalar, **comparable across edges the engine has never seen**"*. |

**A is DECIDED; B is explicitly PROPOSED and assistant-originated** (the notebook's own
tier legend, `material-genesis-notebook.md:9-12`). So this is not a superseding — it is a
**live edge that contests a decided clause without naming it.** The notebook's § 6
*Pointers in* (`:333-337`) lists `material-behavior.md` **§12** as its inbound link; **§14
is not listed, and §14 carries no pointer to the notebook** (only §12 does, at
`material-behavior.md:604`).

The reconciling reading — *predicates are declarations (data), terms inside a pass body are
code, per `north-star.md:430-431`'s "plain data + opaque ids for **declarations**; pass
**bodies** are backend-compiled code"* — is available but **written in neither doc**.

*Recommendation (labelled): §14 gains the same 🔖 OPEN EDGE banner §12 already has. Cheap,
and it is the doc a formation-predicate implementer opens.*

---

### 🟡 7. `ecology.md`'s ratification still stands on the withdrawn 25 s number

| | |
|---|---|
| **A** | `docs/design/ecology.md:267-270` (§ DECIDED 2026-07-20, unstruck) — *"**The 25 s ritual is acceptable** — explicitly, with headroom: the user's frame is that Dwarf Fortress takes minutes… **World-generation time is not a design constraint we optimize against by default.**"* — set up by `ecology.md:239-240` *"**Cost**: ritual 15.2 s → 25.2 s (1.66×)"*. |
| **B** | `docs/design/ecology.md:325-327`, 58 lines later — *"**Cost correction**: the ratified 25 s ritual was **the spike harness's scalar path**. Shipped it is **13.79 s**, biology's marginal cost **+2.6 s** — see journal/corrections.md #12."* |

**The corrections #12 family, at its third site.** `docs/spikes/S10-results.md:3-30` was
stamped with a full superseded banner on **2026-07-28** under the immutable-body/mutable-
header policy. **`ecology.md` — which is where the ratification lives, and which
`ecology.md:231-232` points readers to S10-results *from* — was not stamped in that
sweep**, and its § *Sequenced next* still frames the spike against *"the ~14 s ritual"*
(`ecology.md:230`).

**B is better supported** and A's *decision* is unaffected (the user ratified a cost that
turned out to be cheaper). The exposure is that A is the DECIDED heading a cold reader
quotes.

---

### 🟡 8. `ores.md` § 8 reports a doc-hygiene defect that has since been fixed

| | |
|---|---|
| **A** | `docs/design/ores.md:536-538` — *"**Doc hygiene**: geology.md § v1 content (DECIDED 2026-07-18) **still says** v1 has "one ore vector (placer)"; the 2026-07-20 ore decision supersedes it but **the older line was never annotated**."* |
| **B** | `docs/design/geology.md:48-50` — *"*(Superseded on the ore axis by § Ore — DECIDED 2026-07-20: the full v1 ore roster; engineering pass in ores.md. This paragraph's "one ore vector" was the geology-backbone scope, not the ore roster's.)*"* |

**B is better supported: the annotation exists.** A is a 2026-07-21 observation still
asserted in the present tense. Small, but it is the exact shape this sweep exists to catch
— *and it is a fix that closed with no back-pointer to the report that asked for it.*

---

### 🟡 9. `ores.md`'s roster tables still name members R8 says contradict the ratified representation

| | |
|---|---|
| **A** | `docs/design/ores.md:177-181` (§ 2.1 roster table) and `:234-243` (§ 2.3 class contracts) — members `dc:geo/gold-quartz`, `dc:geo/redbed-copper`, `dc:geo/bog-iron`, carried through the whole doc including `:288-292` and `:330-334`. |
| **B** | `docs/design/ores.md:605-614` (§ R8, added 2026-07-21 by the integrator) — *"**MEMBER NAMING CONTRADICTS THE RATIFIED REPRESENTATION**… this doc's members are named as **DEPOSITS** while geology.md § Ore (DECIDED 2026-07-20) ratifies ore as a **SUBSTANCE inside a HOST**… The first texture pass generated deposit portraits from these names and **had to be redone** (journal/0048). **Proposed amendment (user call):** rename members to substances — `dc:geo/native-gold`, `dc:geo/malachite`, `dc:geo/limonite`."* |

**Self-declared, and honest** — R8 is a *proposed* amendment awaiting the user, so this is
not a silent contradiction. It is listed because **the pointer runs one way**: R8 is the
last section of a 618-line draft, and the five tables an implementer actually reads carry
no marker. It has already cost one redone texture pass.

*The user-facing half (B is an integrator proposal, so `geology.md` § Ore is the
user-originated side) makes B's premise the stronger claim, but the amendment itself is
assistant-originated and undecided.*

---

## 2. Claim inventory — load-bearing claims asserted as CURRENT

*Terse; `file:line` at `f652b60`. Ordering follows each doc. Claims already covered by § 1
are marked ⚠.*

### `material-behavior.md` (ratified 2026-07-23; the content-layer spec)
- `:39-56` A deep cell's inventory is a **stack of `VoxelContents`-shaped spans indexed by depth**; present voxel / deep cell / caves / block↔material are one shape at four resolutions. (user)
- `:71-74` Deep-tier spans carry **finer-than-eighth** fractions; quantize to eighths **only at collapse**. DECIDED 2026-07-23.
- `:76-97` Commit semantics: in-place transformation → **append a FACT**; depositional arrival → **append a UNIT**; **the applied-edge LOG is the fact source, not a post-hoc diff**; empty log ⇒ record byte-identical. DECIDED 2026-07-24 (user).
- `:98-115` Fact shape `(chapter, cause, edge, from→to, fraction)`; **8 B, zero padding**, `EdgeId`; f32 narrowing once at persist, max relative error 5.766e-8. In tree since journal/0108.
- `:116-124` The fact is the seed; pass/epoch/driver are **derived at read time, never stored**. Per-voxel provenance falls out.
- `:141-180` **Forms are a closed set the machine owns** — Structure/Loose/PoreFill/Fluid/Void; Void is the **complement**, not a stored role; occupancy is an orthogonal axis; **forms are NOT SDK-registrable**.
- `:184-228` **The 20-edge transition graph is complete by construction and machine-provided**; `is_declared_edge` / `EdgeId::declared` make a fact structurally unable to name an undeclared transition; only refusals are `from == to` and `Void→Void`.
- `:206-211` **Weathering shrinks the reserved shape; inherent porosity does not** — soft-boundary caves fall out.
- `:232-262` **Agents SUM, not multiply** — DECIDED 2026-07-24 (user); byte-identity with the legacy product world retired; **one fact per agent**; acceptance instrument is a **walk**, not goldens.
- `:264-271` Rates read **live cell + halo state**, not just the sheet.
- `:274-312` **Two pass shapes**: cellular runs edges (writes material, has agents), field computes fields (writes API state, no agents). Processes that seem both are **split**.
- ⚠ `:342-364` **ORDER IS AUTHORED PER WORLD**; `{reads,writes}` is the **validator, not the generator**. SUPERSEDED-of-topo-sort, 2026-07-26 (user), corrections #65. — *and* `:369-370` still says RATE is *"composed with topo-sort"* (finding 3).
- `:365-370` **RATE** = fractional-phase length + the `dt` scaling transformations.
- `:371-407` **WINDOW** = epochs summed into one record entry; *"a window that decides an acceptance number must be declared, not assumed"*; slice-1's 175,320 / 7.378 % divergence was a product of an implicit 25-epoch window (zero at a one-epoch window). RATIFIED (flow.md §11.1). **Declared here, NOT BUILT** (`:405-407`).
- `:398-403` **RIDER, standing on every future record:** any mode changing what an *absent* entry means must be carried **in the record**.
- `:429-446` A cellular behavior is **read → fold agent terms → write** over a `(MaterialId, Form)` fraction; bounded local relaxation; synchronous update ⇒ order-independent.
- `:449-468` The `ctx`'s **three channels** (mutate in-cell spans · read pinned fields · query the free-water body graph). **Free water and transport load stay OUT of the inventory.**
- `:471-486` **Deeptime compiles; the present executes** — one declaration, two executors. *(Validated at the first runtime-process milestone; none exists yet.)*
- `:490-503` **Ownership:** forms + edges = machine; materials + agents + passes = content. *"Acid-rain weathering is a new AGENT, not a new pass."*
- `:509-528` Open: fluid-as-stored-vs-derived · deep-cell span granularity · **dissolution is carbonate-gated, every material's `solubility = 0.0`, the agent exists and is dormant**.
- `:543-559` **Continuation slot:** the R/H↔inventory unification (`H = Σ Loose`, `R = Σ Structure` derived) is the reserved next arc.
- `:563-600` § 12 is a **STRONG LEANING, NOT RATIFIED**; transformation = input-owned, formation = output-owned, recipes = registry-owned; provenance marked per claim.
- `:602-623` **THE DISCRIMINATOR IS RESOLUTION, NOT PHASE** — DECIDED 2026-07-25 (user); the phase test survives only as a heuristic.
- `:632-647` **The four-way test**; **category 2 (transform with a SEAMED driver) is where a project-in-progress lives**.
- `:649-661` **The pathology: filing a (2) as a (4) is irreversible** — A-1 in this domain. *Declare the edge, seam the driver.*
- `:663-681` **The floor moves**; drivers already declared intended (solute, melt, biomass) are **presumed seamed**; a category-4 pass declares its floor + conversion condition.
- `:683-695` Only the **t=0 temporal dropoff** is permanently legitimate genesis.
- `:697-703` **Genesis reads fields; a proxy reads tags.**
- `:705-711` **THE COMPLETENESS TEST** — RATIFIED 2026-07-25 (user). Traceability audited first, four-way classification later.
- `:713-718` **A correctly-classified transformation in the wrong shape is a MIGRATION; a misclassified one is a DEFECT.** (user)
- `:725-755` Scope now = geo, **no registry needed for geo** · distribution emerges from formation predicates · **substance-kind is derived, not tagged** (argmax over property space) · **materials-are-minerals** (flagged as the least settled claim in the doc).
- `:763-777` § 13 status: **R/H unification + material-aware transport RATIFIED**; entrainment/deposition mechanics are assistant framing, user-endorsed.
- `:779-786` **Flow is a FIELD; transport is a CELLULAR pass that walks it** — locality is per-cell, the global structure is the field.
- `:788-796` **Transport is a FAMILY** (water/wind/ice/gravity), differing only in field + competence curve; **tectonic drift is NOT in it**.
- `:803-821` Hjulström entrainment (**only LOOSE entrains**; removing structure is incision) · deposition = **capacity + competence + a falling ceiling**; *"we write the ceiling, not the sort."*
- `:823-834` **R = Σ Structure, H = surface Loose (a positional query, not a column sum)**; cave fill is loose but NOT H; scratch-first reconcile, `H` materialized at pass boundaries.
- `:836-854` **Transport owns clastics; genesis owns the bedrock the sand came off** — DECIDED 2026-07-25 (user). Genesis inherits stubs #16.
- `:856-907` § 13.8b BUILT (journal/0110), on by default; load = multiset of 7 `Litho`; `COMPETENCE_SCALE = 420` **anchored, not fitted**; **⚠ the outcome is a NULL** — 0.000006 %, fluvial is 0.109 % of routing, creep moves 918× more.
- `:909-983` § 13.8c BUILT (journal/0112), on by default; **creep has no competence curve and that is the point** (colluvium is poorly sorted); one `outcrop_shares` seam, three consumers; conservation proved by **bit-exact antisymmetry**; **✅ 65.206 % of recorded mass now disagrees with its own environment**, 71.3 % in the lower five drainage deciles; **nothing was tuned**; surfaced corrections #57.
- `:1001-1019` **Condition-fields** are named per-cell quantities with opaque ids; **a general engine primitive**, expected to serve ecology and civilization too (user).
- ⚠ `:1014-1017` Formation predicates are **plain data over field-ids; no code crosses the SDK** (finding 6).
- `:1021-1023` **NO capability tiers** — a mod authors field passes exactly as the defaults do (north-star § Deviations #2).
- `:1025-1036` **The geotherm is the FIRST field pass** — DECIDED; subsumes stubs #14's degenerate `burial_temp_c`, recalibrates `COAL_ONSET_C` in the same slice, **NOT byte-identical**; unblocks metamorphism.

### `materials.md` (material definitions)
- `:11-34` **A voxel is 8 eighths, three occupancy roles** (structure / debris / pore-fill); loose contents are an **unordered multiset** — deposit order deliberately not tracked.
- `:36-45` Two paths into pores: deliberate packing, and overflow (fines before spill-up).
- `:55-61` **Extraction is typed damage**, yielding in ascending resistance order; *archaeology lives in the extraction mechanics, not a minigame.*
- `:63-79` **Stratification is derived, not ticked** — a pure function of (contents, time-since-disturbance, agitation).
- `:81-97` **S8 RESOLVED (2026-07-18): GO on free-form mixtures.** `C(k+8,8)−1` cap; 2-material processes saturate at **45** states, adversarial 4-material at **495**; real deposit chunks **0.14–0.38 B/m³**; debris-free chunks pay zero.
- `:99-102` Still owed by later work: sparse sidecar encoding, region-table lifecycle, angle-of-repose settling, render-blend prototype, pore-packing mechanics.
- `:119-126` **Breaking is a sampled conservative distribution** — mass that does not become drops stays as debris; *"nothing is deleted; middens happen by physics."*
- `:128-132` Terminology: the design term is **loose**, the storage keyword stays `debris`.
- `:148-158` **Inventory = mass/volume with encumbrance behind a realism knob** · **fracture = per-(material, damage-type) outcome weights**, not one brittleness scalar. Both DECIDED 2026-07-18.
- `:172-189` **Pore packability: `filler_grain ≤ K_PORE × host_grain`, `K_PORE ≈ 0.25`** — derived from ideal-packing interstices + the geotechnical filter criterion. DECIDED 2026-07-20 (user). **Genesis exemption** for formation-context emplacement. **Sieve resistance ≡ grain size** (registry invariant).
- `:191-202` **Sand first, emitted as partials**; the direction generalizes to *"everything should be spawned in partials"*. DECIDED 2026-07-21 (user).
- `:206-217` **Fractions come only from the ledger** — RATIFIED; partial occupancy expresses *recorded* quantity and variance, never cosmetic noise.
- `:219-227` **Sand is a FORM of an existing clastic material, not a new identity** — DECIDED (user); within-identity grain gradation is not modelled. Accepted consequence: loose clastic is textured identically to structural.
- `:229-236` **The soil model (user):** anything with roots carries a **root material in STRUCTURE form** — an anti-erosion model for free.
- `:238-250` **Grass is suspended — do not express it.** DECIDED. Appearance change **pre-ratified**: *"it'll be a mostly brown world for a bit."*
- `:252-287` The fill contract + the **135-site / 31-file consumer audit**; four future systems (water, loose gravity, compaction, sim light) want **one** primitive — free capacity in eighths — which is the argument that carried the decision.
- ⚠ `:296-321` `classify` implemented: structural → debris → pore-fill speaks; most abundant wins, ties to lowest id; **`block_twin` material→block table** (finding 2).
- `:325-333` Occupancy primitives implemented; **`SOLID_EIGHTHS = 4`**; *"the client's collision still reads `Block::is_solid`"* — rewiring is its own slice.
- `:335-349` **The absent-contents rule as built:** the invariant is scoped to **non-empty** contents; a voxel with no record is **unclassified, not Air**; the exception list is enumerated, test-pinned, and **can only shrink** (ruin posts left it 2026-07-28 by deletion).
- `:353-361` Still open: **the fractional top** (deferred behind the veneer's retirement), collision/water/light reading `is_occupancy_solid`, form-dependent texture variants.
- `:363-395` **Distribution-first expression** DECIDED 2026-07-21 (user): the defect is **quantization ORDER**, not resolution — `Σ round(tᵢ/0.9)` where honesty requires `round(Σ tᵢ/0.9)`; the dune field's 379 units summing to 7.99 m all rounded to zero. **Supersedes the stubs § 12 heir.**
- `:396-405` **Quantization happens ONCE, at contents construction.** Presentation (banded vs speckle) is **non-blocking** (user).
- `:407-428` **Dithered eviction, not deterministic truncation** (user) → integrator generalization to **unbiased addressed stochastic rounding** (flagged PROPOSED at `:414`). **The draw is addressed; Floyd–Steinberg is forbidden.**
- `:430-441` Expected consequences to be **measured not assumed**: sieve loss ~75 % → ~0, threshold 0.9 m → ~5.6 cm; **the mixture table grows** (the slice's main risk); the veneer thickness budget should **self-retire** — *verify rather than surgically delete*.
- `:443-465` **ONE slice, the surface folded in** — user overruled the integrator's split. **Where a record exists, the record decides what the world is skinned with.**
- `:467-475` **The one-world-answer constraint:** the record-derived block belongs **inside `surface_sample`**, shared by `column` and `coarse_surface`, or the LOD boundary becomes a visible lie.
- `:477-482` Legitimate fallbacks: border wilds · subaqueous columns · columns whose record rounds to nothing (read as basement, not painted Dirt).
- `:489-510` **ONE NAMESPACE: BLOCK IS MATERIAL** — DECIDED 2026-07-22 (user). *"There are no edge cases where block != material in my mind."* Cost accepted: **art per material**. **Categories stay and become registrable — packs add classes, not only members.**
- `:512-528` **Transformation axes live on the material definition; both sims read the same rule** — DECIDED 2026-07-22 (user). Inheritable / patchable / deletable. *The fires pass's `soil × FIRE_CHAR_FRAC` is the standing counterexample, riding as-built with a three-link heir chain.*

### `ores.md` (**DRAFT — NOTHING RATIFIED**, `:3-12`)
- `:16-85` The six reality-first chains (placer · orogenic lode · bog iron · BIF · redbed copper · evaporite), plus coal as the shipped proof.
- `:92-96` **The roster is ratified** (geology.md § Ore, 2026-07-20, user); tin/tungsten, porphyry, hydrothermal veins **deferred with their engines** — *"no ore before its process."*
- `:97-103` **Representation is ratified:** ore is a MATERIAL inside a HOST; three forms; **grade IS the eighths count**; **ore is subtle** (no glint).
- `:104-107` **Endowment is ratified** — soft guarantee at Medium+, honestly uneven.
- `:108-111` Genuinely distinct raw forms are **DISTINCT materials refining to the same metal** — *identification knowledge is real knowledge.*
- `:118-123` **The methodological template is journal/0026** — roster decided by MEASUREMENT; no rank ladder the data cannot back.
- `:133-142` The recorded context axes as of `recorder.rs`; **U8 flipped 2026-07-21** — production worlds populate `exhum`/`t_crust`/chapters/drainage export, and **the expression slices that consume them are unbuilt**.
- ⚠ `:143-147`, `:288`, `:508-515` metre-scale exhumation gates the flagship (finding 5).
- `:148-152` **Class satisfiability** — a class with no member refuses the world build.
- `:159-165` The one-sentence roster shape: two ride built vectors, two need a small honest deep-sim addition, the flagship splits, coal is done.
- ⚠ `:174-181` The roster table (finding 9).
- `:249-273` **The two process additions asked for:** evaporite deposition (a real recorded facies, **not** painted at collapse) and BIF gating (selection over existing axes, no new process).
- `:284-294` **The genesis-vector honesty table** — BUILT / PARTIAL / STUB per ore, each with a named heir. **`:296-300`: v1 paints no appearance where no cause exists.**
- `:304-318` **AMENDED 2026-07-22: "fails the sieve" is a FORM condition, not a kill condition** — charcoal shipped at 0.394 % of recorded spans after stochastic rounding; re-read every needs-measurement row accordingly.
- `:326-334` Form verdicts per ore from literature thickness; **`:336-343`** grade = eighths (1/8 a show, 3/8 workable, 8/8 a story); extraction rides S8 with **zero new mechanics**.
- `:345-370` **Six named censuses, each with a stated kill condition**, before the corresponding member ships.
- `:372-489` **R1–R7 are the user-owned calls** — each with options, a recommendation, and a plain-language in-game picture.
- `:493-499` Rides as-built: source-blind placer presence (**owes a stubs.md entry**), coal exactly as shipped, the ore-subtlety defaults.
- `:506-542` § 8 collisions, reported not resolved: flagship vs measured exhumation · things-that-will-happen vs the source-blind placer · **BIF vs the ratified Phanerozoic register** (both user-ratified) · S10's arid coal swamps as an accidental gift to redbed copper (*"flagged so nobody later 'fixes' the aridity tag"*) · ⚠ doc hygiene (finding 8).
- `:544-565` **Scope fence** — no smelting/economy, no marker beds, no prospecting mechanics, **no new property-sheet axes**, no display naming, no deferred-engine ores.
- ⚠ `:605-618` **R8** (finding 9).

### `ecology.md` (**substrate RATIFIED 2026-07-19; § 4 evolution is the USER'S DESIGN**)
- `:3-15` **⚠ DORMANT SINCE AUTHORING** (2026-07-26, user): still a ratified user design; *existence-is-not-standing* **has no authority here**; but dormant since 2026-07-19 and **the architecture has moved underneath it**. Four `stubs.md` entries name ecology as their heir and inherit the caveat.
- `:29-58` The reality-first account: **tolerance × limitation (Liebig's `min`, never a weighted mean) × dispersal × interaction × history**; adopted vocabulary — niche, Connell & Slatyer succession, niche construction, CLORPT, **Walker & Syers** (P depletes, N plateaus, retrogression), disturbance regime, ~10 % trophic transfer.
- `:60-77` **Flora → geology is why this earns its cost:** vegetation invented meandering rivers; plants accelerated chemical weathering; **coal, chalk and limestone are corpses**; paleosols and bioturbation write the record.
- `:79-90` **BIOME IS A DIAGNOSIS, NOT A PRIMITIVE** — DECIDED 2026-07-19. **A biome pack is an ORGANISM pack**, symmetric with the geology pack.
- `:92-123` **The bounded substrate, RATIFIED:** per-species niche contract · per-cell ~10 scalars + a **top-K community vector** (*K bounds co-occurrence, never roster size* — the S8 mixture-cap lesson) · **six processes per epoch** · biotic tags join the strata record.
- `:125-129` **Derivation chain:** A-tier millennial propagation → C refines on approach → individual plants by addressed hashing.
- `:131-135` **Lagged coupling is required and is a stated architectural rule** — biology reads *last* epoch's terrain; the pass graph will correctly refuse the cycle. (Claude, flagged.)
- `:137-152` **Evolution as a two-point boundary-value problem — the USER'S DESIGN.** Ancient defs opine on t₀; present-epoch defs **weight the sim**; an **ahistorical origin override flag**; implies an **org body-plan builder**, and NPCs eventually get the same treatment.
- `:164-169` Mechanism (Claude, proposed): the target is a **prior over mutation**, not a thumb on selection — so **the world can falsify the target**.
- `:171-197` Consequences: **mid-horizon pins are fossils** · the target says WHAT, the world says WHERE · **allopatric speciation is free** · bodies.md's plan-params are the mutation substrate · ahistorical orgs have no lineage and that is diegetically perfect.
- `:208-224` **Open forks, user-owned:** fauna in v1 · **pack-addition blast radius** · evolution epoch resolution · educational posture.
- `:226-232` S10 was to be run **before any commitment**; evolution explicitly NOT in S10.
- ⚠ `:234-258` S10 measured: all four signals legible (coal 24 m, paleosols 22.9 %, charcoal 20.2 %, retrogression 23.5 % *with the geography right*); **the lagged-coupling rule worked and was the least troublesome part**; **biology is measurably an erosion term**; the top-K cap **was not stressed**; the doc is **silent on world-genesis colonization** (the spike needed a background propagule term).
- ⚠ `:260-284` DECIDED 2026-07-20 (user): **GO, biology ships** (finding 1); the ritual is acceptable (finding 7); **world-generation time is not a design constraint we optimize against by default**; coal and every missing organic material built immediately.
- `:286-330` SHIPPED: `deep_class` consults `Biofacies` **first and lets it win where inhabited**; four organic classes; **why "wins" and not "blends"** (flow energy describes grains, not plant matter outrunning decay — an arid tag on a coal swamp is correct); **the record's resolution limit bounds what ecology can express as material** (coal 89.6 %, Soil 48.5 %, Retro 6 %, Peat 2.3 %, **Charcoal 0 of 158,310**) — *a facies the A tier can record is not automatically a facies the collapse tier can show.*
- `:332-363` DECIDED 2026-07-21 (user): **vegetation joins deep time**; runtime gen reads the latest ecological state and expresses it **per member** via proliferation patterns, not a painted biome lookup. **The surface veneer is a placeholder — DO NOT BANDAID**; reasoning about grass is suspended until the ecology design pass, which **starts from `biomes-ecology-agenda.md`**.
- `:365-375` **Charcoal/coal rest on placeholder vegetation** — real outputs on a placeholder input; the ecology pass **inherits the obligation** to make fire/peat/coal consume real members.

### `material-genesis-notebook.md` (**OPEN EDGE, not a decision record**, `:3-12`)
- `:22-52` § 1 restates what is RATIFIED elsewhere (authoritative copy: `material-behavior.md` §12) — resolution-not-phase · **the ontology is a SIEVE** (*the thread's founding insight*, user) · the four-way test · the (2)-as-(4) pathology · the moving floor · t=0 only · the completeness test · genesis reads fields · who owns clastic facies.
- `:61-71` **Pass purity — passes are material-agnostic** (user-originated); this turns out to be **already the ratified north-star shape**.
- `:72-83` **And the running engine violates it. Measured 2026-07-26:** `biotic.rs:177 COAL_ONSET_C = 22.0` · `inventory.rs:1294 BEDROCK_SEAM_MATERIAL = GRANITE` · `inventory.rs:1369 basement: GRANITE`. **PROPOSED: a pass-purity audit. Not yet sequenced.**
- `:86-117` **TERM-KEYED EDGES — the load-bearing idea** (user): a pass selects edges by the **terms** they carry, not a shared name. Two payoffs: it **solves competitive ordering** (a scalar comparison topo-sort cannot express) and **dissolves the read-set-union topo-sort problem** (a pass reads *fields*, so adding a material never changes any read-set). Consequence: **genesis stops being a pass type and becomes an edge whose source is a field**.
- `:119-137` **Per-parameter shadowing with parent-value access** (user) — strictly better than edge-level shadowing, **needs no new rule**. Creates a requirement: parameters need **three** states (inherit / override / disable).
- `:139-164` Six candidate primitives; **the ordering key carries the architecture** (the only one that must be comparable across edges the engine has never seen). **Terms should stay FUNCTIONS** ⚠ (finding 6). PROPOSED boundary: engine owns primitive terms, **a pack may declare derived terms with the passes that read them**.
- `:166-186` **(a) The identity default is INERT, never ERROR** · **(b) the plug-and-play mechanism IS parent inheritance, so our default pack's PARENT materials are a PRODUCT SURFACE** — *"nothing in the corpus says this, and it is the most actionable finding in the thread."*
- `:188-194` **(c) Interop and interference are the same mechanism.** OPEN.
- `:198-237` **Fluid is a FORM, not a substance — DECIDED** (`flow.md` §2.5, RATIFIED 2026-07-25); the flow carries a **fluid material id**; rheology is material properties. **What is open is narrower:** the inventory still treats `InvForm::Fluid` as derived-never-stored, so the record cannot hold *"3 m of brine-in-fluid-form."*
- `:239-247` **🔴 A CORRECTION — an assistant assumption caught by the user before it was recorded:** §4's `from: <fluid: brine>` bindings assume a fluid-identity model that does not exist. **The examples are kept with the correction attached; do not read them as a proposed API.**
- `:256-303` Worked examples at the intended fidelity altitude: charcoal as **pyrolysis discriminated by burial-for-oxygen** (an honest coarsening **visible in the declaration**) · hydrothermal quartz · **the evaporite sequence** (*a modder adds `mymod:borax` with one number and it slots between halite and sylvite; the pass never learns what borax is*) · emerald (**the sieve sits lower than expected**).
- `:307-329` Five ranked open questions; **q5 records that the three-way split of "genesis" was explicitly NOT ratified** — the user agreed only that *pre-loop is not a pass*, was *"NOT sold"* on the rest.

### `octree-substrate.md` (D1–D3 DECIDED 2026-07-22; node contract RATIFIED v0.1)
- `:14-23` **The octree mostly exists, in pieces, under different names** — the S-2 story at the structural level. The S3 chunk pyramid **is** an octree; `MixtureDownsampleRule` is built, tested and **unrendered**.
- `:40-45` Binding ratified constraints: one shared terrain material · Bevy's GPU-driven mesh path · **no simulation-resolution edge may reach the eye** (S-4) · **rendering is never an observer**.
- `:47-55` **D1 — one substrate, and it is the existing chunk pyramid.** A node is `(level L, ChunkPos)`; edge = `0.9 · 2^L` m. **The first two consumers define the payload.**
- `:57-66` **D2 — first slice is FF2b-minimal**; the v0.1 rider requires it exercise **at least one synthesized-node path**.
- `:68-74` **D3 — stepped all the way**, at every level.
- `:84-124` **The node contract v0.1:** three strata (contents *(exists)* · occupancy/sky *(partial)* · **water summary — requirements only**), each with a declared reduction rule and source. **`CoarseField<T>` is the sampling vocabulary, adoption deferred.**
- `:126-138` **Derivation is TWO-SIDED** — reduce upward where children exist, **synthesize top-down** where they do not (most of the far field, forever), and **the two must agree statistically where they meet** — *the acceptance test for every synthesis path*. This is also the legal home of the far-field summarization that leaked into `collapse.rs::surface_sample`.
- `:139-143` **Ungenerated is not empty** — `derive_material_lod_chunk` reading a missing child as empty is an A-5-shaped defect over a lazily generated world.
- `:144-161` Strata are independent and extensible · **synthesis is seeded** · committed/fluid at node granularity · rendering is never an observer · **agreement tests are the acceptance tests** · every consumer declares its halo bound · headless split.
- `:167-173` Intended extensions, **recorded not designed**: statistical/social tier; a relation graph *over* the index.
- `:176-209` D3's record of considered alternatives — imposters **rejected**; **Aokana ray-marched SVDAG is OPEN, not rejected** (an earlier draft's "effectively rejected" was **user-corrected as scope-inference**), orthogonal to D3 and **spike-gated**; the mesh commitment is deliberate architecture, and if the march route is taken it is **far-field-only**.
- `:211-233` Open questions, none blocking D2; **q3 resolved 2026-07-22** (span stacks, ~3.5 ms/tile); **q5 is the Aokana tripwire** with FF2a's headroom checkpoint (~10 km ≈ 376 tiles / ~43 MiB) saying the mesh path is nowhere near the wire.
- ⚠ `:36-38` "S16 is unwritten" (finding 4).

### `worldgen.md` (draft 2026-07-18; **history layer ON HOLD 2026-07-28**)
- `:6-32` **⏸ THE HISTORY LAYER IS ON HOLD** (user). *History is coming, eventually, for the reasons this document states* — and *"not exhaustive"*. **Recorded ambition is not fabrication**; the *implementation* was unratified bootstrap content and was removed 2026-07-28 (journal/0121, 102 wood voxels, no golden moved). Six sections listed as affected.
- `:34-48` **The core decision:** bounded in extent, lazy in detail; real forward-simulated history above region scale (⏸); collapse-on-approach below; unbounded wilds at the borders.
- `:50-62` **⚠ WHICH LEG CARRIES BOUNDEDNESS:** history-needs-a-finite-world is on hold; **CLOSURE is pure earth science, live, and load-bearing**. *So bounded-in-extent stands on its own* — recorded so nobody "discovers" a collapsed justification (spines A-2).
- `:64-77` **The level pyramid** — ~5–6 levels, one rule: a cell's collapsed state = f(its base state, 1-ring neighbours' **base** states, the **parent**'s collapsed state). One hop per level, but influence rides the coarse levels.
- `:79-104` Above region scale: tectonics · climate/hydrology · **deep-time geology via the materials model** are the **three live stages**; **item 4 (history) is ON HOLD and NOT A PIPELINE STAGE TODAY** — the S2 ledger machinery has **no production caller** and is **kept deliberately as a shape reference**.
- `:106-114` Below region scale: nothing pregenerated; **extreme depth costs nothing until descended into**; voxel detail is never bounded by the extent decision.
- `:116-126` ⏸ After year zero — **the no-seam property is the requirement worth preserving** and the first thing to re-derive.
- `:128-140` **Borders: unbounded wilds — DECIDED**, and *unaffected by the hold; it was never contingent on history.*
- `:142-160` **Extent is a player knob — DECIDED**; **⏸ two of its four sizing criteria are on hold**, so *the extent default is currently being chosen on half its intended criteria.*
- `:162-175` Open: **topology PROVISIONALLY DECIDED by S7 — continent-disc in a world-ocean** (a wrap would force a seam column through every pyramid level) · coarse cell resolution · legends UI scope · savegame determinism policy.
- ⚠ `:177-211` **§ Sequencing — DECIDED 2026-07-28 (user)** (finding 1): the ordering is firm; the engine shape that must hold first (**engine owns primitives + runner; plugins declare all content**); **that list is explicitly NOT exhaustive**; **there is no checkable gate and that is deliberate** — *"Ask; do not infer."*

---

## 3. What I could NOT attribute

- **`worldgen.md:64-77`'s level pyramid (~5–6 levels, ~10–100 km coarse cells).** Every other doc in the slice speaks in **~460 m deep cells** and **0.9 m voxels**. I could not determine, from the docs I read, whether the 5–6-level pyramid is still the live shape, was collapsed into the deep-cell/collapse two-tier split, or is a third thing. **I cannot attribute this** — it needs `ARCHITECTURE.md` § chunk shape and `S7-results.md`, which I did not open in full.
- **Whether `materials.md:325-333`'s *"the client's collision still reads `Block::is_solid`"* is still true.** That is docs-vs-code and belongs to `spine-audit`; I did not verify it and am not reporting it as a finding.
- **`ores.md`'s six censuses (§ 5) — run or not?** The doc lists them as gates before members ship. I found no census results in the docs I read and did not search `journal/` or `examples/`. **I cannot attribute their status**, so I did not rank the several ores whose shipping decision depends on them.
- **Whether finding 1's reconciliation exists in `biomes-ecology-agenda.md`.** That doc is named by `ecology.md:360` as where the ecology pass starts and I did not open it. If the "S10 substrate ≠ the ecology pass" distinction is written anywhere, that is the most likely place.
- **`material-behavior.md:405-407`'s WINDOW: "declared here, not built".** Whether a `Pass` has since acquired a window field is a code question I did not ask.

## 4. A null I am reporting honestly

I looked specifically for **user-originated designs reconciled away** (shape 3) across the
slice — the corrections #65 shape — by reading every *"(user"* / *"user's strong leaning"* /
*"RATIFIED"* block in all seven docs. **I found none new.** The two candidates I checked and
cleared: `material-behavior.md` §12's leanings are provenance-marked per claim
(`:568-572`), and `material-genesis-notebook.md:239-247` is an assistant assumption the
**user caught and the notebook recorded as a correction** — which is the rule working, and
worth saying out loud in a report otherwise full of failures.

Finding 3 is the *residue* of the original #65 reconciliation, not a new one.
