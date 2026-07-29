# ROADMAP staleness sweep — the 2026-07-25 FLOW / identify / residency reconciliation

> ## ⚠ A DATED SNAPSHOT — named as such by `journal/corrections.md` #55 (2026-07-26).
>
> This sweep's verdicts describe the board **as of 2026-07-25**, before the deep-time
> scale finding (`journal/0111`, corrections #56), the erosion recalibration (`journal/0114`)
> and the bootstrap-content removal (`journal/0121`). **Its "STALE" and "UNAFFECTED" columns
> are not current.** Later staleness sweeps supersede it item-by-item; this one is kept so a
> reader can see what was true on the day and what the next sweep did not have to re-walk
> (§ *Checked and found genuinely current*).
>
> *Banner added 2026-07-29 (baseline sweep S8/B7).*

**Read-only audit, 2026-07-25.** Sweeps the whole board — Shipped `OWED` clauses,
In flight, Sequenced (~2482–3604), Observed (~3605–5444) and the still-live
**2026-07-24 BUILD-DAY close block** (~5445–5510) — against what shipped today:

- **FLOW** — slice 1 (journal/0096, flux on faces), continuation (a) (journal/0098,
  `dc:field/head`), design doc **`docs/design/flow.md`** ratified with §§ 10–11.
- **`identify(pos)`** UNTIERED, `UNRECORDED` first-class (journal/0101, corrections #49).
- **Residency** — `FactLedger` → CSR (0100) → one grid-wide record (0102); probes in
  the gate (0103); the front's mass claim settled as noise (corrections #50).
- **`reads_prev` is a mechanism** (0104) and `dc:field/head` declares its terrain
  revision **and** the writer that supersedes it (0107).
- **corrections #51 — the shipped world has ZERO coal**; R3 measured (0106): per-depth
  weathering is cheap in gen time and **973 MiB resident**.
- **The weathering front is a PROFILE** (0099), walk-PASSED by the user.

**No ROADMAP, journal, stubs, spines or source edits made — the integrator applies.**
Line numbers are as of commit `2f3803d` (ROADMAP unchanged since); headings given as a
fallback. Verdicts carry a line, a test name, or a journal citation; nothing here rests
on "feels stale".

---

## The three that would most mislead the next session

1. **`3e-2 — C refinement` still says "Nothing open — implementable" while the same
   Sequenced section says its expression half is superseded** (row C-1). Two Sequenced
   entries, ~1000 lines apart, give opposite dispatch advice about the same work.
2. **Movement 2b's ratified mechanism walks the structure FLOW is retiring** (row R-1),
   and Movement 2b exists **only inside a close block** — it has no Sequenced entry, so
   the wrap rewrite can silently drop the arc the whole session pointed at.
3. **The coal chain** (rows S-6/S-7): a Shipped entry claims coal is "still a diggable
   seam on Medium", and the only test defending *"a player can find and dig a coal seam"*
   runs on a seed the client cannot open — corrections #51's exact defect, one file over,
   in a file journal/0106's sibling audit did not enumerate.

Runner-up worth reading immediately after: **flow.md § 11.1's user-RATIFIED third
scheduler axis (`ORDER × RATE × WINDOW`) says "Sequenced, not built" and appears in no
sequence** (row D-3).

---

## SUBSUMED

| item | ROADMAP line | evidence | ready-to-paste correction |
|---|---|---|---|
| **B-1. Observed — "Caves ↔ hydrology integration thread captured"** | **4310–4321** | All four of its "work-shaped findings" now have owners: **two drainage opinions** → flow.md § 6 retires `pregen/hydrology.rs` outright (*"wrong resolution, wrong time, wrong topology"*); **"deep drainage is computed and discarded — per-chapter table recorder axis wanted for erosional caves"** → *that is `DeepField::flux`*, shipped (journal/0096: 2,590,372 entries, per chapter, 40.66 MiB, `per_chapter_history_is_retained_not_just_the_final_epoch`); **the bounded-drainage-refinement spike question that gates RiverSeg retirement** → S14 is **"superseded as posed"** (flow.md § 9.6; water.md banner l.12) and RiverSeg retirement is FLOW continuation **(e)** (ROADMAP 2589); **column-as-interval-log** → flow.md § 7 promotes it from *proposed* to **"becomes necessary"**. | Strike the entry and replace with a pointer: *"SUBSUMED 2026-07-25 by the FLOW arc — the two-drainage-opinions subsumption is flow.md § 6's retirement list, the discarded-per-chapter-drainage finding is `DeepField::flux` (journal/0096), the bounded-refinement spike is S14-superseded-as-posed (flow.md § 9.6), and the interval-log contract is now **necessary** (flow.md § 7). Cave-specific residue rides continuation (c)."* |
| **B-2. Sequenced — the field-pass migration's `burial_temp_c` clause** | **2641–2643** (inside 2638–2648) | stubs.md #14 is headed **"RETIRED 2026-07-24 (journal/0093, the geotherm field pass)"**; ROADMAP Shipped 433 says the geotherm *"Retires the degenerate `burial_temp_c` stub (#14) **ENTIRELY**"*. | Delete the `burial_temp_c` clause from the migration list (see row S-1 for the rest of that entry). |

---

## CONTRADICTED — named, **not resolved**; the user's call

| item | ROADMAP line | side A | side B |
|---|---|---|---|
| **C-1. `3e-2 — C refinement`** | **3508–3518**, the verdict at **3517–3518** | *"**Nothing open — implementable.**"* — DECIDED 2026-07-19: drainage coarse-at-A, river-conditioning, corridor wander, **descent-along-flow + no-divide-crossing as a hard constraint**. | The FLOW arc header, **2491–2494**: *"Supersedes the river half of `water.md` and the **expression half** of `earth-processes.md` § 3e-2; § 3e-2's constraint half survives **qualified** — the no-divide-crossing rule binds the FREE regime only, or regional groundwater is foreclosed."* flow.md § 6 lists § 3e-2's expression half under **"What this retires"**. A third stamp is also live: the 2026-07-24 sweep marked 3e-2 **VALIDATED** (*"drainage is decided-once-coarse and is the sole advect"*). **Not resolved here.** The narrow question: *does anything of 3e-2 remain dispatchable, or is what survives only the qualified constraint?* |
| **C-2. Who owns clastic facies** | **3005–3011** (genesis-passes "WHAT") vs **2597–2601** (FLOW acceptance) | GENESIS-PASSES: *"Rock **distribution + physical facies** come from **deeptime genesis passes**… Output: where materials are · how distributed · **the facies**."* | flow.md § 8 / ROADMAP 2597–2601 makes facies the FLOW arc's **un-gameable acceptance test** (*"channel gravel with a placer streak → floodplain silt → an unconformity…"*), and material-behavior.md § 13.7 says transport *"**IS** clastic sedimentary genesis… delivers most of Movement-4 genesis for clastics as a consequence of the erosion loop."* Three ratified things claim one output. Probably complementary (genesis = *which rock*; flow/transport = *where the clastics went*) — **but nobody has said so**, and the genesis arc's first slice converts `deep_class`/`dithered_member`, the same seam transport would move. |
| **C-3. The close block's "Owed / carried" geotherm-coal walk** | **5503–5504** | *"Appearance walks (S18 band once M3; **geotherm coal** — low-priority placeholder)"* — listed as owed. | corrections #51 / ROADMAP **3630–3637**: **0 coal units across 297,025 cells** on the shipped world; the Sequenced walk tracker at **2676–2684** already reclassifies it — *"**THE WALK IS A NULL AND NEEDS NO GAME TIME** … **Do not spend a walk on it**."* Mechanical: the wrap rewrite must not carry it forward as a walk. |

---

## STALE — intent survives, the stated mechanism / numbers / blockers are wrong

| item | ROADMAP line | evidence | ready-to-paste correction |
|---|---|---|---|
| **S-1. "MIGRATE EVERY 'NOT REAL' FIELD INTO A REAL DECLARED FIELD PASS"** | **2638–2648**, target list **2641–2645** | Three of four named targets moved. (i) `burial_temp_c` → **retired** (B-2). (ii) **drainage `recv`/`area`/`lake` is not a migration target — it is a DELETION target**: spines § 3 marks the row *"**SUPERSEDED 2026-07-25 (journal/0096, FLOW slice 1)**, retirement sequenced"*, heir = `DeepField::flux`, disposal = continuation **(e)** (2589). (iii) `dc:deep/drainage` has been **a declared pass since Movement 1** (journal/0090; the 17-pass order at Shipped 45–47 names it) — the unconsumed thing is the exported *plane*, not the pass. Only `exhum`/`t_crust` survives as written. Two field passes have landed since the entry was filed. | Replace the target list with: *"`exhum` + `t_crust` exported planes (spines § 3, still unconsumed — heir is metamorphic grade, stubs #4). **Struck: `burial_temp_c`** (stubs #14 retired 2026-07-24, journal/0093) and **drainage `recv`/`area`/`lake`** — those planes are **superseded, not migratable**; their heir is `DeepField::flux` and their disposal is FLOW continuation (e). `dc:deep/drainage` is already a declared pass (Movement 1). Vocabulary now: `dc:field/temperature` (0093), `dc:field/head` (0098) — two rows this migration did not have to open."* |
| **S-2. "WEATHERING IS ONE PROCESS" — the gating sentence** | header **2726–2727**; requisites **2754–2766**; new blocker **2782–2793** | Header still reads *"Not scheduled; **gated on the requisites below, and it enters Sequenced the moment they are met**."* R1 ✅, R2 ✅, R3 ✅ (marked in-entry, 2759–2767); commit `160858b`'s own message: *"the gate is open, and the blocker moved."* The replacement blocker is **residency: `LedgerField` 17.45 MiB → 973 MiB (55.8×), resident**, with three named axis-levers (drop chapter ÷4.8; drop agent ÷3; per-slot scalar). | *"**Requisites MET 2026-07-25 (R1/R2/R3). The gate is open and the blocker moved: this is now a DESIGN question, not a prerequisite question.** Before it can be built the arc must answer **what is persisted** — chapter axis, agent axis, or a per-slot scalar (§ R3's levers) — which is the *'a summary must be derived from the authority, never become it'* question this project already has a doctrine for. Gen time is affordable (25.7 s → 46.4 s); residency is not (973 MiB)."* |
| **S-3. "STRUCTURE-AWARE FINE EXPRESSION" — the structuring signal** | **2624** and **2626–2629** | Both name **`recv`** (*"the flow field `recv` + local topography for placement"*; *"the cell's composition **and** its flow vector (`recv`)"*). `recv` is the receiver tree the FLOW arc retires; the arc's own UNIFIES clause (2515–2516) already says the flux record is *"the rich thing others bias to"*. The flow half of the input **exists today**; only *composition* still waits on Movement 2b. | 2624 → *"the **flux record** (`DeepField::flux` — per-chapter directed flux on faces, journal/0096) plus local topography for placement"*. INPUTS paragraph → *"**The flow half of the input already exists** (the flux record shipped 2026-07-25 and carries direction *and* magnitude per chapter, which `recv` never could — one out-edge cannot express a fan). The composition half still rides Movement 2b."* |
| **S-4. Sequenced "Water-model design pass"** | **3554–3562** | No pointer to `flow.md`, which now opens water.md itself with a supersession banner (water.md l.2–13). Its groundwater half is **partly delivered at the deep tier**: `dc:field/head` derives transmissivity / vertical conductivity / **confinement** from the strata record's own permeabilities (*"a marine mud over a fluvial sand **is** a confined aquifer — no landform code path"*, 2557–2562), and artesian **occurs naturally** (60 columns, max excess 2.94 m) — recharge open as stubs #19. | Prepend: *"**⚠ Read `flow.md` first (ratified 2026-07-25).** The river / drainage / channel-expression half is superseded; the **groundwater** half is partly delivered at the **deep** tier by `dc:field/head` (journal/0098 — confinement derived from the record, artesian occurring, recharge open as stubs #19). What this pass still owes is the **present/runtime** tier: visible water, ponds and sub-resolution water, speleogenesis, the free-water body-graph coupling. Caves ride FLOW continuation (c)."* |
| **S-5. Sequenced "Coal rank when burial deepens"** | **3491–3498** | Stated blocker: *"rank transitions live at ~1–2 km of burial and our deepest recorded overburden is ~100 m"*. Coalification is now **temperature**-gated (`COAL_ONSET_C`, journal/0093), not burial-gated; and the shipped world grows **zero coal** (corrections #51). | Append: *"**Re-blocked 2026-07-25.** Coalification moved onto a real temperature axis (the geotherm, journal/0093), so 'the depth axis IS the rank axis' is no longer the whole rule — and the shipped world (1337/Medium) carries **0 coal** (corrections #51), so a rank ladder has nothing to discriminate on any world a player can open. Revisit after the coal-content call (a)–(d)."* |
| **S-6. Shipped — the geotherm entry's coal claim** | **436–439** | *"**Coal recalibrated** (`COAL_ONSET_C 8→22 °C`): 12% → 60% of peat candidates, relocated to warm crust, **still a diggable seam on Medium** (`the_geotherm_coal_shift_is_plausible_not_degenerate`)."* Measured on the **warm reference** world; on the shipped world: **0 coal units / 297,025 cells**, hottest candidate 15.4 °C vs a 22 °C onset (3633–3637). The cited test name no longer exists — `tests/geotherm.rs` now carries `the_geotherm_rule_governs_coalification_on_the_production_world` (1337/Medium, **requires no coal**) and `coal_follows_the_warm_crust_on_the_warm_reference_world`. | Append: *"**CORRECTED 2026-07-25 (corrections #51):** 'still a diggable seam on Medium' was measured on `0x0D5EED572026`, **not** the shipped world. On seed 1337 / Medium the recalibration produces **zero coal** — the hottest candidate is 6.6 °C short of onset. The guard has been split and re-seeded (journal/0106); the content question (a)–(d) is open in Observed."* |
| **S-7. `tests/organic.rs` — the "a player can dig coal" claim runs on a world the client cannot open** *(the gap is that there IS no ROADMAP line)* | related: Observed **4756–4767**; journal/0106's sibling audit at **3669–3675** | `crates/dc-worldgen/tests/organic.rs:31` — `const SEED: u64 = 0x0D5E_ED57_2026;`; `the_measured_coal_seam_is_coal_a_player_can_dig` (`:264`) builds `medium()` from it. `MIN_DIGGABLE_COAL_VOX` (`:78`) has been re-baselined **15 → 10 → 6**, twice flagged **NEEDS RATIFICATION**, every time on that seed. Observed 4756–4767 records that `dc-client` can only ever open **1337** (`BENCH_SEED`, an `i32`) and that `0x0D5EED572026` *does not fit* it. journal/0106's sibling audit enumerates `providers_common`, `rh_unification`, `s18_first_behavior_weathering`, `deeptime::production_config`, `water::coarse::production()` — **`organic.rs` is not on the list**, and unlike those its claim is **not** seed-independent: it is a claim about what a player finds. | New Observed entry: *"**🔴 The only test defending 'a player can find and dig a coal seam' runs on a world no player can open.** `tests/organic.rs::the_measured_coal_seam_is_coal_a_player_can_dig` builds `SEED = 0x0D5E_ED57_2026` (`organic.rs:31`); the client boots `BENCH_SEED = 1337` and the reference seed does not fit its `i32`. `MIN_DIGGABLE_COAL_VOX` was re-baselined 15 → 10 → 6 on that world, twice under a NEEDS-RATIFICATION flag. This is **corrections #51 one file over**, missed by journal/0106's sibling audit because that audit asked 'is the claim seed-independent?' and this claim emphatically is not. **Two honest options, the user's:** re-seed to 1337 and watch it fail (the true statement about the shipped world), or rename it `..._on_the_warm_reference_world` and file the shipped-world diggability claim as **unguarded**. Do not leave it named for a player."* |
| **S-8. S10 ratification call 3 — the abundance census** | **3451–3460** | *"coal 0.55 % of columns, paleosols 22.9 %, charcoal 20.2 %, retrogression 23.5 %"* — measured 2026-07-20, before the susceptibility blend (0072) and the geotherm (0093), on the reference seed. Coal on the shipped world is **0 %**. | Append: *"**Numbers are historical (2026-07-20).** Coal has moved twice since — the susceptibility blend made near-surface coal recessive (journal/0072) and the geotherm re-sited it onto warm crust (0093) — and on the shipped world it is now **0 %** (corrections #51). Re-census before treating any of these as an appearance call."* |
| **S-9. In flight — "async-offload slice is now the sequenced NEXT"** | **2097–2099** | Shipped 2026-07-23: journal/0083 + journal/0084, Shipped entry at **532–559**. Status drift only. | Strike-through and point at Shipped 0083/0084; the live successors are the throughput-ceiling and per-thread-attribution Observed items (**4115–4131**). |

---

## UNBLOCKED / TRIGGERED

| item | ROADMAP line | what met the prerequisite | should it move? |
|---|---|---|---|
| **U-1. "S2 checkpoint facts (deep-time re-derivation cost) — design owed **before ledgers densify**"** | **5129–5130** | **The ledgers densified today.** `LedgerField` holds **1,033,189 facts** across 72,006 slots (0102); `DeepField::flux` adds **2,590,372 entries / 40.66 MiB** (0096); `dc:field/head` adds 6.96 MiB (0098); R3 projects **973 MiB** for a per-depth weathering ledger (2782). Its own trigger has fired. | **Yes.** Promote from the one-line Observed tail to a real entry: *"**TRIGGERED 2026-07-25.** The condition this line waited on has occurred — the deep record now carries three sparse per-cell records and a fourth is projected at 973 MiB. The re-derive-vs-persist question is live, and flow.md § 11.3 gives it a standing constraint: **any mode that changes what an absent entry means must be carried in the record.**"* |
| **U-2. "STRUCTURE-AWARE FINE EXPRESSION" — the within-CELL half** | **2626–2629** | Its stated wait was Movement 2b (*"transport already produces the inputs"*). **Half the input landed early** — the flow vector is now the flux record (S-3). User status unchanged: *"interesting-but-unsettled… NOT a committed direction"* (2630–2636). | **No move** — an input change, not a promotion. Fix the input sentence per S-3 so nobody re-derives "we must wait for 2b" for the flow half. |
| **U-3. Metamorphism** | **5497–5498** (close block), **3117**, **3125–3127** | Unblocked 2026-07-24 by the geotherm (`exhum` = P, `dc:field/temperature` = T); spines § 3 lists **both** planes with metamorphic grade as intended consumer; stubs #4 is the same absence. Nothing today changed its status — but the wrap is about to overwrite its only "sequenced" home. | **Yes** — see D-1: give it one Sequenced bullet before the close block is rewritten. |

---

## RESHAPED

| item | ROADMAP line | how the shape changed |
|---|---|---|
| **R-1. Movement 2b — material-aware transport** | close block **5495–5496**; spec = material-behavior.md § 13 | **Its ratified mechanism walks the structure FLOW retires.** § 13.1: *"Transport is a **cellular pass** that carries a transient load cell-to-cell **in downstream order along the pinned receiver**."* The pinned receiver is `recv` / `Cell::flow_to` — the spanning tree flow.md § 2.1 kills (*"cannot represent divergence at all"*) and continuation (e) deletes. Two further reshapes: the driving field is **potential**, not elevation (flow.md § 2.4 — and `dc:field/head` now exists), and **MFD is the nearest-term continuation** (2578–2579), which is precisely what turns one receiver into several. § 13.2's wind/ice/gravity family is the same as FLOW's `Cause` = **the mover** (flow.md § 7); stub #18 already names Movement 2b as heir of the flux atom's constant `cause`. The **load-exchange half is explicitly claimed by the FLOW arc** (2517–2518). |
| **R-2. In flight — the hydrology tail** (coarse-capacity void axis; re-run S15's natural-sill falsifier) | **2402–2412** | The void axis says *"decide it once **in water.md** rather than twice"*. Its home moved: flow.md § 7 makes the **interval-log fill contract** *necessary* (not proposed), and voids / conduits / springs are FLOW continuation **(c)** — which as of today ships **three** obligations, not two, because a karst conduit pairs by **void connectivity**, a third mode flow.md § 11.5's two-mode green does not cover (2582–2587). The natural-sill falsifier is gated on the same (c). |

**Ready-to-paste for R-1** — as a new Sequenced bullet, so the arc survives the close-block rewrite:

> - **MOVEMENT 2b — MATERIAL-AWARE TRANSPORT** (material-behavior.md § 13, ratified
>   2026-07-24; **RESHAPED 2026-07-25 by the FLOW arc — read `flow.md` first**). The load
>   multiset, Hjulström entrainment and settling deposition stand. **What changed:**
>   § 13.1 routes transport *"in downstream order along the **pinned receiver**"* — that
>   receiver is the spanning tree FLOW retires (flow.md § 2.1; deletion is continuation
>   (e)). Transport must walk the **face-flux record** (`DeepField::flux`), which can
>   diverge, and descend **potential** (`dc:field/head`), not elevation (flow.md § 2.4).
>   **MFD is the nearest-term FLOW continuation** and is what makes several receivers
>   representable at all (§ 2.6) — so 2b either **follows MFD** or ships knowing it cannot
>   fan. It also discharges stub #18's constant `cause` field. **Sequencing question for
>   the user:** 2b after MFD, or 2b on the faces as they stand today?

---

## UNAFFECTED-BUT-ADJACENT — cross-references so the next reader finds them

| item | ROADMAP line | why cross-reference |
|---|---|---|
| **A-1. Observed — `DeepField::strata` is `Vec<DeepStrata>`** (9.06 MiB of structs over an 84.47 MiB heap; 11.3 % of cells hold an empty record) | **3787–3798** | Now on the **weathering arc's critical path**: R3 named **residency**, not gen time, as that arc's blocker (2782–2793). The two entries should point at each other — the strata collapse is the next lever on the same budget the weathering arc must fit inside. |
| **A-2. Observed — coal renders as pure black in the lit pass** | **4712–4743** | Still a valid lighting/tonemap question, and now **un-walkable on the shipped world** — there is no coal to photograph (corrections #51). One line so a future session does not launch for it. |
| **A-3. In flight — "an absent chunk must read **UNKNOWN**, never solid"** (corrections #31, the S11 connectivity index) | **2408–2411** | `Identity::Unrecorded` (journal/0101) is the **in-tree precedent** for a first-class "no record here" answer, distinct in the *type* from an empty value. Cite it so the connectivity index copies the shape rather than inventing a second one (A-4). |
| **A-4. Observed — "A front's parent alternates diorite/granite down a single column"** | **3874–3880** | New today, already correctly assigned to the genesis-passes arc. It is the member-dither family's first *measured* within-column instance — worth linking from "Mixed voxels carry no member dither" (**4255–4266**), still the untested-either-way twin. |
| **A-5. Observed — the palette-quant STATION + the far-LOD material split** | **3994–4055**, **4072–4113** | Untouched today; the mixture-representation arc they converge into is still held. One thing moved under them: `identify(pos)` is **untiered** now (2876–2904), so the *"the distance pyramid — near/mid/far **are** the `identify` tiers"* framing at **2982–2986** no longer holds. The pyramid is a **render** concern only. |

---

## DUPLICATES and ORPHANS

| kind | what | lines | note |
|---|---|---|---|
| **D-1. DUPLICATE (3×) + ORPHAN** | **Metamorphism** | close block **5497–5498**; layer-cake redemption **(b)** at **3117**; consume-the-ledger **(b)** at **3125–3127**; plus stubs #4 and two spines § 3 rows | One job, three ROADMAP homes, and its only *sequenced* home is a **close block the wrap is about to overwrite**. Give it one Sequenced bullet — `exhum` = P + `dc:field/temperature` = T → grade, retiring stub #4 and emptying the `exhum`/`t_crust` § 3 row — and make 3117 / 3125–3127 point at it. |
| **D-2. ORPHAN** | The **`production_* → golden_*` rename**, annotated *"left **sequenced**"* | **3671–3672** | There is **no Sequenced entry for it**. `providers_golden.rs` plus comments in `flux_record.rs`/`head_field.rs` still name a non-shipped world `production_*`. Either sequence it or drop the word "sequenced" — the same doctrine that would have made S-7 findable. |
| **D-3. ORPHAN (the big one)** | **flow.md § 11.1 — the aggregation window as a third scheduler axis** (`ORDER × RATE × **WINDOW**`), **RATIFIED by the user 2026-07-25**, ending *"**Sequenced, not built.**"* | **absent from ROADMAP entirely** | Grep-verified absent, and **material-behavior.md § 5 "Cadence: order × rate" (l. 315) is unamended** — the spec of record for the scheduler does not carry the new axis. A ratified architectural decision living in one paragraph of one design doc. Same exposure for **§ 11.3** (*"the record must SELF-DESCRIBE its completeness"* — a **standing contract** on every future record), in neither ROADMAP nor spines. **Ready-to-paste Sequenced bullet:** *"**THE AGGREGATION WINDOW IS A DECLARED AXIS** (flow.md § 11.1, RATIFIED 2026-07-25). § 5's cadence grows a third axis — **ORDER × RATE × WINDOW**. A window that decides an acceptance number must be declared, not assumed: slice 1's divergence count was produced by an implicit 25-epoch chapter. Consequence (§ 11.2): chapter-vs-epoch resolution becomes a shipped **default**, not an engine property — default cheap, expose the knob. Rider (§ 11.3): **the self-describing-record contract** — any mode that changes what an ABSENT entry means must be carried in the record; the marine-sink lever (2541–2543) is its first customer."* |
| **D-4. DUPLICATE (within one entry)** | `identify(pos)`'s **"THE FAR TIER'S PAYLOAD IS A MIXTURE, NOT A WINNER"** bullet | **2955–2967** | Still written in tier language the **UNTIERED** decision (2876–2904) retired *the same day*, and its surviving content is already restated at **2901–2904** (*"payload is uniformly a mixture… there is no far tier left to special-case"*). Its **unique** residue is the render-side speckle direction and the LOD fix (b) cross-ref. Fold: keep the last two sentences, strike the tier framing — or mark it *"preserved for the render-side query"* as its two siblings already are (2929, 2941). |
| **D-5. ORPHAN (walk tracking)** | The `pore_rider_share` correlation walk — *"a fullbright walk along a strong front looking for banding correlated with the parent's eighth"* | proposed at **3869–3872** | Not listed in **APPEARANCE WALKS OWED** (2650–2685), which is the tracker. Add it as item (5) or say it is desk-deferred. It is also the user's call by its own text — re-addressing the rider's draw moves every contact voxel in the world. |

---

## Checked and found genuinely current (so the next sweep does not re-walk them)

- **stubs.md #16 / #18 / #19 / #20** — all four correctly statused as of today; #17 discharged.
  #20's heir sentence (*"the deep tier carrying the front as a **depth-resolved term**…
  deleted rather than tuned"*) is exactly the WEATHERING-IS-ONE-PROCESS arc, and the arc
  says the same from the other side (2804–2807). No drift.
- **spines § 3** — rows for `flux`, `head`, `chapters`, `geotherm`, `exhum`/`t_crust` and
  `recv`/`area`/`lake` are current, re-confirmed 2026-07-25 by the post-FLOW sweep. The
  `recv` row is the evidence behind **S-1**.
- **APPEARANCE WALKS OWED** (2650–2685) — (1)(2)(4) done and walk-verdicted; (3) correctly
  reclassified as a desk null. Only **D-5** is missing from it.
- **The `identify(pos)` continuation slot** (2993–2999) — storage/wire migration, runtime
  edit-fact overlay, far-span `Block`→material, legacy-S1 retire: **all four untouched by
  today's work** and correctly stated. journal/0101 says explicitly that nothing was
  drained, deleted or migrated and the path stays edit-blind for composition. No correction
  owed — only **D-4**'s stale tier wording inside the same entry.
- **The FLOW arc entry itself** (2489–2605) — carries slice 1, continuation (a), the conduit
  third mode and the three open riders honestly. What it does *not* carry is flow.md
  §§ 11.1 / 11.3 (**D-3**).

---

## Genuinely unsure — flagged rather than asserted

1. **C-1's third position.** The 2026-07-24 sweep stamped 3e-2 **VALIDATED** ("decided-once-
   coarse, the sole advect") as a claim about pass **shape**; flow.md supersedes its
   **expression** and qualifies its **constraint**. These may all be compatible (shape
   survives, expression dies, constraint narrows) — but three stamps on one item, none
   referencing the others, is how a dispatch goes wrong.
2. **R-1's sequencing.** I am confident the *mechanism* reshaped; I am **not** confident
   whether 2b should wait on MFD. Walking today's faces is defensible (the record already
   carries divergence from avulsion); MFD-first is defensible (simultaneous divergence). The
   entry states the fork rather than picking it.
3. **S-7's disposition.** The defect is real and unrecorded. Whether the right response is
   re-seeding (and accepting a red test that tells the truth) or renaming (and filing the
   shipped-world claim as unguarded) is a user call about what the world is supposed to
   contain — the same (a)–(d) call corrections #51 already put in front of them.
