# ROADMAP staleness sweep — the 2026-07-23/24 material-model reconciliation

**Read-only audit, 2026-07-24.** Sweeps every **Sequenced** item (ROADMAP
~1951–2490) and the model-touched **Observed** items (~2491–3571) against the
new decisions of this session:

- `docs/design/material-behavior.md` (ratified 2026-07-23/24) — deep-cell
  inventory = fill contract one tier up; forms = closed machine set; edges =
  machine-complete process catalog; agents = content rate-terms on edges;
  passes = **cellular** vs **field**; commit-semantics = append facts;
  provenance = base+facts.
- `north-star.md` § Refinement 2026-07-23 — field-solvers reframed core→content
  (core keeps solver primitives + ledger); passes cellular/field;
  deeptime-compiles / present-executes; tiering deferred; agents/actors a
  deferred second substrate.
- `materials.md` DECIDED 2026-07-22 — block IS material, one namespace.
- journal/0086 (substrate design pass), journal/0087 (A1 collapse merged).

**No ROADMAP or design-doc edits made — the integrator applies.** Line numbers
are as of the read (commit 1a16e67 vintage); headings are given as a fallback.

---

## SUPERSEDED

| Item | Location | Supersedes it | Proposed edit |
|---|---|---|---|
| "Data-driven block registry (API.md `define block_type`) supersedes the appended enum here or soon after" | Sequenced **3c-2**, L2460–2461 | **block-is-material** (materials.md DECIDED 2026-07-22): one namespace, `Block = {Air, Material(MaterialId)}`, no separate block registry. There is no `define block_type`; a material's own identity *is* the block. | Strike the sentence. Replace with a pointer: "block identity now collapses into MaterialId (block-is-material, In-flight); the appended enum is retired by the Crux 1 storage-atom redefinition, not by a block registry." The rest of 3c-2 (sidecar dither, partial-height loose render, threshold collider) has largely shipped — mark it STATUS-DRIFT/shipped separately. |
| **The forms/partials design pass** ("partials-first world-gen emission… generalizing to forms across all systems") | Sequenced, L2080–2085 | **material-behavior.md §§ 2–3** IS this design pass, now ratified (journal/0086). Forms are a closed machine set (structure/loose/pore-fill/fluid; void=complement) with occupancy orthogonal; the transition graph is machine-complete. | Mark the design-pass DONE → material-behavior.md. Keep only the *implementation* residue (worldgen loose emission, mesher partial-height) as a rider under the substrate build (Crux 1). The "design pass first" framing is spent. |

---

## STALE-REFRAME

| Item | Location | Proposed reword |
|---|---|---|
| **The distance pyramid — consume the mixture LOD by band, drop the dither with range** | Sequenced, L1953–1986 | Still real. Reframe as a **rider under Crux 1 (far-field span/`Block`-token migration)** + the palette-quant diagnosis. The `MixtureDownsampleRule`/`DominantClassDebrisAware` machinery it names is exactly the reduce path implicated in the far-field material-identity split (Observed L2538) and the palette-quant station (Observed L2493). Note the far band's "winning material" is now the **dominant material identity** post-block-collapse (journal/0087), not a `classify`-to-block step. Sequence *after* the palette-quant diagnosis + contents inspector, which share its locus (the >4→4 winner reduction). |
| **Sim light — what the voxels know** | Sequenced, L2223–2300 | Reframe sim light as a **field pass** (material-behavior § 5): it computes a per-cell light-level *field* via bounded max-plus relaxation, plants it as a pinned coarse field (ctx channel 2, § 7), and cellular biology passes *read* it. The "structural parallel to bound-water saturation" note (L2268) is now **formalized** in material-behavior §§ 6/10 (both are bounded local relaxations attenuated by a per-material property). Design-pass/spike sequence still owed; the reword is terminological + slots it into the two-pass-shape taxonomy. Keep the max-plus determinism note — it is exactly the field-pass order-independence argument. |
| **The sim must know about light** (Observed twin of the above) | Observed, L3126–3133 | Same reframe — a field pass computing a per-cell light field. The bound-water parallel it flags is now ratified in material-behavior § 6. Point it at the Sequenced sim-light item and material-behavior § 5. |
| Tectonic expression at the collapse tier — **layer-cake redemption**, esp. piece **(c) drainage export (`recv`/`area`/`lake`) → macro drainage consumers** | Sequenced, L2006–2020 | This is **field-pass output consumption**. Reframe (c): `recv`/`area`/`lake` are the **pinned coarse fields (ctx channel 2)** the *cellular transport pass* reads — material-behavior § 7 and § 11 name them explicitly as "built, populated, consumed by nothing (spines § 3)." Pieces (a) dip/fold and (b) `exhum`/`t_crust` are likewise the **tectonic/thermal field-pass state** the collapse reads. The whole item is legitimate native-core field-output plumbing (see VALIDATED). |
| **The world is a LAYER CAKE — no dip/fold/tilt** | Observed, L3087–3103 | Same substance as the layer-cake-redemption Sequenced item. Reframe as consuming the **tectonic field pass's** deformation output (uplift/deformation field, § 5) at collapse — the fold-phase `f(uplift)` relax term is the field the cellular/collapse tier reads. Cross-link to the Sequenced item so a reader sees mechanism + symptom together. |
| **Loose materials do not exist in the world yet** (spread-on-drop / angle of repose) | Observed, L3115–3123 | **Loose is now a machine-owned form** (material-behavior § 2, "obeys gravity") and spread-on-drop is the `void→loose` / gravity-march behavior over the form inventory. Reframe from "half-built content/sim gap" to "rider under the substrate: the loose form + its gravity edge; the partials renderer is the waiting present-tier half." |
| **3a. Form archetypes + drop distributions** | Sequenced, L2452–2453 | The "form archetypes" half is subsumed by the machine form set (material-behavior § 2 — forms are structure/loose/pore-fill/fluid, machine-owned, NOT the old open "archetypes"). Reword to keep only "**drop distributions** at the first inventory/interaction milestone"; point "forms" at material-behavior § 2. (Overlaps NOW-A-RIDER.) |

---

## NOW-A-RIDER (subsumed under a new keystone/spec)

| Item | Location | Parent it now rides under | Proposed edit |
|---|---|---|---|
| Consume-the-ledger piece **(c) — derive material FORM from provenance** | Sequenced, L2028–2037 | **The deep-cell inventory keystone (Crux 1)** + forms machine set + commit-as-facts. Form is now a machine axis on each `(MaterialId, Form)` span; "derive form from provenance" = read `base + facts` (material-behavior §§ 1–3). | Retag as the flagship consumer of the deep-cell working-inventory. It is no longer a standalone "derive a form byte" slice; it falls out of the inventory + commit-as-facts. Keep stubs.md §12 (amalgamate sub-voxel units) as its enabling piece. |
| Consume-the-ledger piece **(d) — per-voxel provenance query** | Sequenced, L2038–2041 | **commit-as-facts** (material-behavior § 1). | **RESOLVED** — the doc states verbatim "Per-voxel provenance falls out (Sequenced item d, resolved)." The "needs a design pass first / PROPOSED not ratified" caveat is discharged: provenance = read `base + facts`; a move is itself a fact carrying lineage. Mark (d) resolved-by-design, retain only the *implementation* (surface it in the contents inspector). |
| **Organics render as solid boxes because form is keyed on CLASS** | Observed, L2622–2634 | Forms machine set + FORM-derivation (= the deep-cell inventory). | Already correctly attributed ("already the sequenced work: consume-the-ledger (c)"). Update the attribution target: (c) is now the deep-cell inventory keystone, and `is_loose`-by-class is replaced by the `(MaterialId, Form)` span carrying an explicit form (soil/peat = loose per § 2). It is the first visible symptom. |
| **Charcoal as an inclusion, not a band** (sub-voxel units dropped by `deposit_deep_history`) | Observed, L2366–2377 | **The deep-cell inventory** — material-behavior § 1 granularity: deep-tier spans carry **finer-than-eighth fractional quantities**, quantize to eighths only at collapse. | The "redistribute-into-host / inclusion channel" is now the general collapse-quantization rule of the deep-cell inventory (+ stubs.md §12 amalgamation), not a bespoke charcoal slice. Retag as a rider. |
| **Far-field LOD reconstructs a coarse box's MATERIAL IDENTITY differently cold vs warm** | Observed, L2538–2553 | **Crux 1 far-field span migration** + `classify`/`MixtureDownsampleRule` (already stated in the entry). | Already correctly framed ("belongs with the far-field span migration under Crux 1… a live violation of spines S-9"). No reword needed beyond confirming it is the same locus as the distance-pyramid rider; note both wait on the far-span `Block`-token migration. Keep "diagnose before touching." |
| **91.4 % of land skins to ONE block** — the *form* half ("organics formed as structure would render as cubes even once a soil material exists") | Observed, L2740–2749 | Forms machine set + FORM-derivation (deep-cell inventory). | The *content-set* half (no LOAM/soil substance registered) is a real STILL-VALID content gap — leave it. The *form* half is the same rider as "organics render as boxes." Split the note so the form half points at the deep-cell inventory / forms, not at a standalone fix. |

---

## VALIDATED (blessed by the new model — state it so nobody "fixes" it toward cellular/declarative)

| Item | Location | Why it is legitimate as-is |
|---|---|---|
| **Tectonics SPIKE** (uplift/deformation, chaptered plate state) | Sequenced, L2086–2094 | Tectonics is a **field pass** (material-behavior § 5: "field: uplift/deformation" + "tectonics + deposition/accretion split field/cellular"). It is native-core solver work — NOT a candidate for the cellular/declarative agent treatment. The spike stands; add a one-line "this is a field pass" affirmation so nobody tries to recast it as edges/agents. |
| **Erosion-supply calibration** (denudation vs orogen rates) | Sequenced, L2103–2109 | Erosion = the **drainage(field) + transport(cellular)** split (§ 5); calibration is legitimate field-pass tuning. Note the cause-3 reframe already recorded (journal/0079, corrections #41): relief is generation-side, `--erosion-budget` is a dev tool. Bless the calibration as field-pass work. |
| **3e-2 — C refinement** (drainage coarse-at-A, sole advect, no divide-crossing) | Sequenced, L2396–2406 | Explicitly blessed: material-behavior § 5 cites "**3e-2 decision 1: drainage is decided-once-coarse and is the sole advect**" as the exemplar of the field/cellular split (advective non-locality lives in the field; cellular transport follows the pinned receiver and cannot re-route). State "VALIDATED by the two-pass model." |
| **S11 water — the free-water body graph / connectivity index** (four open calls) | Sequenced, L2408–2440 | The body graph is **ctx channel 3** (material-behavior § 7 — "behaviors QUERY, never store an edge"); free water and transport load are the two things the doc explicitly keeps **out of the inventory** (§ 7, S-2). The open calls (Finite/Pinned, halo, persistence) are legitimately pending the water design pass (§ 10 defers fluid-storage). Bless, don't retrofit. |
| **Water-model design pass** | Sequenced, L2442–2450 | Blessed as a field pass + body-graph channel; material-behavior § 10 explicitly defers the fluid-storage question to it. STILL owed, and correctly so. |
| **Roughness recalibration** (three costed candidates) / **Dismal mountains** relief thread | Sequenced L2042–2064; Observed L3051–3084 | Deep-field **relief generation** — untouched by the material model; a field/terrain axis. journal/0079 already reframed cause 3 as generation-side. State "unaffected by the material-behavior model" so it is not conflated with the palette/form threads. |

---

## STATUS-DRIFT (already shipped/closed this arc — note, but the conceptual staleness above is the real work)

| Item | Location | Note |
|---|---|---|
| Consume-the-ledger **(a) carry `H`** | Sequenced, L2023–2024 | Already SHIPPED (journal/0053); marked. No action. |
| **Tectonic uplift-plane redesign — DESIGN PASS** | Sequenced, L2110–2116 | Already marked "(superseded — done)". No action. |
| **Erodibility coupling / NEEDS-RATIFICATION erosion block** | Sequenced, L2138–2212 | Largely ratified + answered (journal/0079, corrections #41). Only item 3 (four rate coefficients, plausible-not-tuned) rides open. No conceptual staleness. |
| **A1 present-tier collapse** (referenced across the block-collapse In-flight) | In-flight, L1490–1497 | MERGED `de984eb` (journal/0087); the Observed "surface material quantized per chunk: FIXED" (L2687) and related already reflect it. Contents-absent far path still block-tier — feeds the distance-pyramid rider. |

---

## STILL-VALID (unaffected — leave as-is; listed for completeness of the Sequenced sweep)

| Item | Location |
|---|---|
| Edited chunks accumulate — save-layer heir (persistence) | Sequenced, L1987–1994 |
| Collapse-cache `evict()` unreachable from far-field sampling | Sequenced, L1996–2004 (note: owner reassigned to the `collapse.rs` rewrite, now under the substrate build — a soft reframe, not staleness) |
| Consume-the-ledger **(b) consume `exhum`/`t_crust`** | Sequenced, L2025–2026 (borderline VALIDATED — these are the metamorphic-grade *field* axes read at collapse; field-pass output plumbing) |
| A 5th/6th far LOD level (horizons past ~10 km) | Sequenced, L2068–2072 |
| Wave-magnitude retune — STRUCK (rides as-built) | Sequenced, L2073–2079 |
| Retire `client_player_pose_set` into `dc:character/pose` | Sequenced, L2118–2123 |
| FF2b — coarse volumetric summaries | Sequenced, L2129–2135 (pairs with the distance-pyramid rider) |
| Far-field range knobs / ranges-as-player-config | Sequenced, L2214–2221 |
| PBR-2 — lit-world completion (renderer; explicitly NOT sim light) | Sequenced, L2302–2322 |
| S10 biotic-layer five open calls | Sequenced, L2324–2358 (item 5 "organism pack" aligns with agents=content ownership; item 3 soil-abundance couples to the no-soil-substance Observed gap) |
| Coal rank when burial deepens | Sequenced, L2379–2386 |
| 3d — geology post-v1 roster (accessory inclusions as pore partials aligns with the pore-fill form) | Sequenced, L2462–2468 |
| 3e — epoch-indexed formation context | Sequenced, L2469–2472 |
| 4/5 — ecology + organisms (biology-as-erosion-term = agents on edges; aligns) | Sequenced, L2473–2486 |
| 6 — body-plans staircase (entity substrate is the deferred *second* substrate, § 10) | Sequenced, L2487–2489 |
| **STATION — per-chunk palette-quantization checkerboard** (fresh 2026-07-24; live diagnostic) | Observed, L2493–2524 (couples to the distance-pyramid rider + contents inspector; the >4→4 winner reduction is the shared locus) |
| **Dev slice — look-at-voxel contents inspector** (fresh 2026-07-24) | Observed, L2526–2536 (highly relevant: it is also the instrument for inspecting the deep-cell inventory / provenance-as-facts — worth cross-linking, but the item itself is valid as written) |
| Far field boxier than near / far carries no contents | Observed, L2611–2621, L2657–2659 (geometry rider under the distance-pyramid + far-span migration; distinct from the material-identity split) |
| Mixed voxels carry no member dither / member-selection on chunk lines | Observed, L2695–2706 |
| Razor-straight grass/dirt frontier in far field | Observed, L3209–3239 |
| Unlock ranking (persistence / inventory / loose / water) | Observed, L2958–2971 |
| Thick units render as flawless monoliths (collapse-tier procedural detail) | Observed, L3105–3113 (loosely pairs with charcoal-inclusion rider) |
| Perf threads (throughput ceiling, per-thread perf instrument, async-offload edit-corner hook, pooling, coal-black tonemap, heightmap-shape legibility, albedo-at-range) | Observed, L2555–2609, L3136–3150, L3152–3183, L3581–3630 — perf/render, unaffected by the material model |
| S2 checkpoint-facts (deep-time re-derivation cost) | Observed, L3569 (loosely: commit-as-facts § 1 — "facts persist, working inventory transient" — begins to answer the re-derivation-cost question; worth a cross-link, not stale) |

---

## Genuinely unsure

1. **Sim light: field vs cellular.** The task says reframe it as a **field pass**, and I followed that. But material-behavior § 6 lists light *with weathering* as "a bounded local relaxation… structurally for light" — a description that reads cellular. The clean split: light **computes a per-cell field** (output = environment planted as cell state, consumed by biology) → **field pass by output type**, even though its *body* is a local relaxation rather than a global solve. The doc's § 5 field-pass definition ("compute a value per cell… never run edges, don't touch the form inventory") fits light exactly. I'm ~80% on field-pass; flagging because light's local-relaxation body is unlike the global drainage/tectonics solves that motivate the field category. Pending the sim-light design pass to settle it.

2. **Consume-the-ledger (b) `exhum`/`t_crust`** — filed STILL-VALID but it is arguably VALIDATED (field-pass output). Left in STILL-VALID because it is a straightforward "consume a produced field" plumbing slice with no risk of being mis-fixed toward cellular. Integrator may promote.

3. **Collapse-cache `evict()` item** (L1996) — its stated owner is "the forms/partials `collapse.rs` rewrite," which is now subsumed by the substrate build. Not stale in substance (still a real unbounded-growth gap), but its *owner reference* drifted. Left in STILL-VALID with a note rather than STALE-REFRAME because the memory fact is unchanged.
