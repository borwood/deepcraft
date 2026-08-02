# P11 — MEMBERS INTO DEEP HISTORY: representation options, residency envelope, ripple map

**Arc anchor:** `ROADMAP.md:359-398` § Sequenced *"MEMBERS INTO DEEP HISTORY"* — **DECIDED
2026-08-01 (user), TOP PRIORITY**. Companion: `docs/design/materials.md:522-541`;
`docs/dependency-graph.md:79` (P11 row).

**Produced by a read-only design-pass agent. It DECIDES NOTHING.** Every option below is
priced and its trade-offs named; the pick is the user's. Anything the agent originated is
marked **(assistant-proposed)**. No cargo was invoked; every number is arithmetic over
already-measured quantities, cited to source, or a `size_of` computed by hand from the
field list and flagged as such.

> **RULINGS (mutable header; body is the dated design record):**
> 1. **The record names `MaterialId`** (user, 2026-08-01, *"certainly"* — the Q1 upstream
>    fork). Rationale as presented and accepted: after deposition-time fitness has run,
>    the member has done its whole job — the fitness envelope is *selection machinery*,
>    and history records **the rock, not the road to it**. Two members sharing a material
>    are two paths to one rock. Zero-byte swap in `DepUnit`; openness arrives via the
>    material registry (the SDK arc), never via a second identity system. `GeoMemberIdx`
>    stays what it is: the selector fitness speaks, spent at deposition.
> 2. **Representation = OPTION A, CLEAN** (user, 2026-08-01: *"i agree with this direction
>    generally"* after killing D's premise): identity per unit, fitness at deposition, and
>    **NO class view survives in storage or physics** — Option D died on the user's
>    challenge *"are they just places the old shape is entrenched?"*, which verified: the
>    far `ShareVec<6>` site's semantics are already rejected-interim, the transport planes
>    need per-cell support not a global roster, and `LithoResistance` is already
>    per-material. Fixed-N sites re-shape (sparse/top-k) rather than inherit a blessed
>    class view. A future coarse view, if ever wanted, is derived against a real consumer
>    and gets the proxy question asked THEN. Options B/C/E stay gated as written (B behind
>    the closure audit; C/E behind a deliberate P5 opening — corrections #65).
> 3. **Transport planes go SPARSE DAY ONE** (user, 2026-08-01, verbatim). Dense scales as
>    cells × registry-size — the wrong asymptote for a plugin-first engine with an open
>    material registry; sparse scales as cells × local presence (a cell's in-transit load
>    only holds its catchment's species). CSR precedent in-tree (`FluxRecord`). Budget
>    arithmetic stays f64 unless a store-f32 split re-proves Law-3 closure.
> 4. **U5 (pack-members-move-terrain) is DISSOLVED, not ruled — corrections #84.** The
>    user's 2026-07-19 world-identity rule (`ideas.md:189`, *"patch set joins pack set in
>    world identity"*) makes the question vacuous: a different pack set IS a different
>    world; and P11's record-bake kills the one real hazard (retroactive change to an
>    existing world's expression). What remains is owed work, not a decision: rewrite the
>    `lithology.rs:58-62` comment in the P11 slice (its reason expires — A-2), and note
>    the identity FIELD is E7's manifest (the rule is written; the field is not — the
>    2026-07-22 seam inventory). Caught by the user's memory before ruling; second
>    instance of corrections #81's shape in three days.
> 6. **IDENTITY PROPAGATES; THE DEPOSITION DRAW IS NAMED SCAFFOLDING** (user,
>    2026-08-02, at slice 1's harvest): *"the interim does not matter in any way shape
>    or form... we are marching toward slice 2 ASAP. this is not a real fork."*
>    MaterialId — not membership of a group — is the basic unit of deeptime: identity
>    is a conserved quantity flowing through the mass arithmetic (erosion releases →
>    transport carries by id → deposition records what settled). The fitness draw at
>    deposition is INTERIM SCAFFOLDING with two named heirs: **slice 2** retires it for
>    transported deposits (identity from the arriving composition term); **FS-A**
>    retires it for weathered material (release spectra, behind U1). What survives is
>    the genuine-degeneracy remainder (primary formation, transformation edges under
>    declared conditions), drawn coherently. The interim got the cheap engineering fix
>    — (cell, chapter) addressing — with no ratification weight attached.
> 5. **MM-3 / the near-path restructure FOLDS INTO P11's build sequence** (user,
>    2026-08-01). One pass over the shared files instead of two overlapping surgeries;
>    the restructure lands on the member-grade record it will serve; its cell-membership
>    dither (spatial) survives P11 untouched while the expression-time member-fitness
>    machinery it would have preserved is deleted instead. The ~460 m near tile rides
>    until that slice; the octaves walk remains sequencing input. **The design phase of
>    P11 is CLOSED** — remaining opens (Litho-dissolution residue sites, record-terms
>    ruling 3 re-entry) are build-sequencing content, not design questions.

**Status legend:** **DECIDED/RATIFIED** (user) · **BUILT** (code exists, verified here) ·
**PROPOSED** (recorded, not decided) · ⚠ **FLAGGED** (could not verify / stale input).

**Immutable body, mutable header** (CLAUDE.md read-first item 5). Read at commit
`9b5971a` + this file. Anything that later refutes or re-scopes this audit gets a banner
**here**, stamped by the author of the correction.

**Read with:** `docs/audits/2026-07-29-fluvial-record-terms-priors.md` (its § 1.6 and § 4
are the direct parents of §§ 2–3 below) · `docs/spikes/S19-flow-record-cost-results.md` ·
`docs/spikes/S20-fact-ledger-residency-results.md` · `docs/design/refinement.md` § 5 (the
three Laws) · `docs/design/material-genesis-notebook.md` § 2 · `docs/design/stubs.md`
#16 / #21 / #23 / #25 / #31.

---

## 0. The one finding that reorganises the whole question

**The expression tier is ALREADY member-grade end to end. The deep record is the only
class-grade link in the chain, and it is exactly one byte wide.**

The pipeline as built (every hop verified in code):

```
DepUnit.species : Litho            (7 variants, 1 byte)        recorder.rs:273
   → deep_class_of_species(species) → &'static str class       geology.rs:431-440
   → GeologySet::select(class, FormationContext, u)            dc-core/materials/geology.rs:381-419
        = fitness(window, ctx) × normalised abundance × draw
   → GeoMemberIdx(u16)              (14 in vanilla)            dc-core/materials/geology.rs:225
   → GeoMemberDef.material : MaterialId (26 registry entries)  dc-core/materials/geology.rs:193
   → VoxelContents.slots : [MaterialId; 8]                     dc-core/materials/contents.rs:111
```

Three consequences the design must be argued from, not around:

1. **`MaterialId` is one byte** — a `#[repr(u8)]` fieldless enum with 26 contiguous
   variants (`dc-core/src/materials/mod.rs:39-70`), niche-bearing on purpose. `Litho` is
   also one byte. **Swapping `DepUnit::species: Litho` for `DepUnit::species: MaterialId`
   is a ZERO-BYTE change** — `size_of::<DepUnit>()` stays 16 (§ 3.1). That is the single
   most consequential arithmetic in this document.
2. **The deep-cell inventory is already `MaterialId`-grade and already up-converts.**
   `inventory.rs:1144-1146`:
   ```rust
   pub fn derive_base(unit: &DepUnit) -> Vec<Portion> {
       material: unit.species.reference_material(),
   ```
   `Portion { material: MaterialId, … }` (`inventory.rs:427-428`) and `EdgeId` over
   `(MaterialId, InvForm)` (`inventory.rs:242`) are the *keystone* the north star named
   (Crux 1) — and they are member-grade today. The class-grade record is up-converted into
   them **through a fixed reference table**, `Litho::reference_material()`
   (`lithology.rs:301-311`). P11 does not build a new bridge; it **replaces one lookup with
   a recorded fact.**
3. **The class layer's only irreplaceable job is FITNESS** — `GeologySet::select` is where
   `(temp_c, precip, depth_m)` become an identity. P11's real question is therefore not
   "what shape is the species field" but **"when does fitness run: at deposition, or at
   expression?"** Every representation option below is a different answer to that.

---

## 1. Code inventory — what speaks `Litho` today

**The count, both ways, because they differ and the difference matters:**

- `git grep -n "Litho" -- crates/ plugins/ tools/ | wc -l` → **333** across **28 files**
  (substring — includes `LithoResistance`, `REFERENCE_LITHO`, `lithology`).
- `git grep -n "\bLitho\b" -- crates/` → **308** across **26 files**, **all in
  `dc-worldgen`**. No `Litho` in `dc-core`, `dc-sim`, `dc-client`, `dc-api`, `dc-physics`.
- `deep_class_of_species|dithered_member` → **31** sites.
- **Of the 308, ~250 would have to change if the roster stopped being a fixed 7**; ~58 are
  doc-comment or incidental mentions.

**That `Litho` is confined to one crate is a real result:** the class-grade abstraction
never leaked into the engine crates. `dc-core` speaks `MaterialId` and `GeoMemberIdx`
only. P11's blast radius is `dc-worldgen`, not the workspace.

Breakdown by file (substring `git grep -c`, so slightly high; the word-boundary counts are
in the change table below):

| file | hits | what it is |
|---|---:|---|
| `dc-worldgen/src/deeptime/lithology.rs` | 110 | **the roster itself** — the enum, `COUNT`, `ALL`, `index()`, `code()`, `reference_material()`, `resistance()`, `litho_of_tag`, `as_deposited`, `window_walk`, `exposed_shares`, `settling_table`, `susceptibility_table`, `WindowShares` |
| `dc-worldgen/src/deeptime/erosion.rs` | 12 | `SPECIES = Litho::COUNT` (`:377`) and the four `n × SPECIES` planes |
| `dc-worldgen/src/collapse.rs` | 15 | `DEEP_CLASSES = 6`, `DEEP_CLASS_ORDER`, `deep_class_slot`, `top_voxel_shares` (the **far field**) |
| `dc-worldgen/src/geology.rs` | 11 | `deep_class_of_species` (`:431`), the collapse-tier bridge |
| `dc-worldgen/src/deeptime/head.rs` | 9 | per-`Litho` reads on the head/flow path |
| `dc-worldgen/src/deeptime/providers/mod.rs` | 7 | the `outcrop_at` / `outcrop_shares` provider slots |
| `dc-worldgen/src/deeptime/providers/outcrop_shares.rs` | 4 | the `WindowShares` identity |
| `dc-worldgen/src/deeptime/recorder.rs` | 3 | **`DepUnit::species: Litho`** (`:273`), `deposit_as`, `promote_coal` |
| `dc-worldgen/src/deeptime/{mod,weather_behavior,weather_inventory}.rs` | 3 | re-exports / incidental |
| tests: `providers_common`, `providers_outcrop_at`, `outcrop_blend`, `erodibility`, `full_agents`, `material_transport`, `material_creep`, `contents_contract` | 66 | the mirror-invariant suite (`litho_routing_matches_the_collapse_tier`) and the blend laws |
| examples: `entry_species_probe`(42), `facies_probe`(14), `erodibility_probe`(9), `outcrop_dominance_probe`(9), `tour_map_0071`(7), `colluvium_probe`(5), `amplitude_tour`(3), `outcrop_blend_probe`(2), `walk_tour_0115`(2) | 93 | measurement instruments; several are **gate-carrying** (`[[example]] test = true`) |

### 1.1 The sites that CHANGE if the roster stops being a fixed 7

These are the load-bearing ones — everything else is a rename.

| # | site | shape | why it breaks |
|---|---|---|---|
| C1 | `lithology.rs:258` `Litho::COUNT = 7` | `const` | the width every table below is generic over |
| C2 | `lithology.rs:565` `pub type WindowShares = ShareVec<{ Litho::COUNT }>` | **const generic** `ShareVec<const N: usize> { shares: [f64; N] }` (`dc-core/src/coarse.rs:151-153`) | a const generic **cannot be open-ended**. An open roster forces either a compile-time max, a sparse `(id, share)` list, or keeping a fixed *parent* alphabet here |
| C3 | `lithology.rs:707` `susceptibility_table(agent, …) -> [f64; Litho::COUNT]` | fixed array, built once/epoch, read per cell | member-grade widens it 2–4× and turns "one table per agent per epoch" into "one table per agent per epoch per **registered member**" |
| C4 | `lithology.rs:647` `settling_table() -> [f64; Litho::COUNT]` | ditto | same |
| C5 | `lithology.rs:467` `window_walk` — `acc`/`order`/`seen` are `[_; Litho::COUNT]` | stack arrays in a hot per-cell walk | member-grade at 26 makes these 26-wide stack arrays per cell |
| C6 | `erosion.rs:377` `const SPECIES: usize = Litho::COUNT` | **four `n × SPECIES` `Vec<f64>` planes** — `qs_sp` `:1465`, `dep_sp` `:1474`, `creep_sp` `:1490`, `shares` `:2169` | the biggest *memory* consequence in the tree (§ 3.3), and `split_by_shares` (`:482`) plus the sorted-settling loop (`:2951`) are O(SPECIES) inner loops |
| C7 | `collapse.rs:1581` `DEEP_CLASSES = 6` + `:1593` `DEEP_CLASS_ORDER` + `:1603` `deep_class_slot` | `CoarseField<ShareVec<6>>` — the **far field's** surface class draw | same const-generic problem as C2, at the tier that was walked and ratified 2026-07-29 |
| C8 | `lithology.rs:382` `litho_of_tag` / `geology.rs:442` `deep_class` / `geology.rs:431` `deep_class_of_species` / `collapse.rs:1603` `deep_class_slot` | four **exhaustive matches over the 7 variants**, pinned pairwise by tests (`erodibility.rs::litho_routing_matches_the_collapse_tier`, `collapse.rs::deep_class_slot_matches_deep_class_of_species`) | these are the *class routing*. Under member grade they either dissolve (identity is recorded) or become the parent aggregation (§ 4) |
| C9 | `lithology.rs:695` `Litho::as_deposited` | a match encoding "basement quarried and carried is coarse detritus; transported organics are carbonaceous mud" | **this is real physics, not scaffolding** (journal/0112, corrections #57). Under member grade it becomes a *per-material* declared transition — which is exactly `materials.md`'s 2026-07-22 transformations-on-material-definitions rule |
| C10 | `lithology.rs:301` `reference_material()` | the class→member table | **retired by P11 by construction** — it exists only because the record cannot name a member |
| C11 | `inventory.rs:250-255` the `EdgeId` compile-assert | `MATERIAL_COUNT * FORM_COUNT <= 256` → **hard ceiling of 51 materials**, STUB #21 | a member-grade record does not itself trip this, but P11's *motivation* (packs register members) walks straight at it. Heir already named: the interned per-world edge dictionary (S20 § 3.2) |
| C12 | **three DUPLICATE exhaustive `Litho`→class matches** outside production: `tests/erodibility.rs:176` `class_of`, `tests/providers_common/mod.rs:419` `species_code`, `examples/entry_species_probe.rs:123` `class_of_litho` | hand-written copies of `geology.rs:432` / `lithology.rs:274` | **S-3 parallel rules** already in the tree. Each is a site that must be updated in lockstep and *cannot* be caught by the compiler if the roster becomes data rather than an enum. Worth naming in the ripple as a hidden cost of any option that opens the roster |
| C13 | `erosion.rs:1386` `litho: Vec<u8>` — the per-cell exposure plane, decoded `Litho::ALL[b as usize]` at `:1952` | `n × 1 B` = 290 KiB | **safe at ≤ 256 members** — a byte-wide identity plane survives member grade untouched. Listed because it looks like a table and is not one |

### 1.1a Sites that are ALREADY registry-shaped and survive any option

Recorded because they are the shape the rest should aim at, and because assuming they
break would inflate the ripple:

| site | why it survives |
|---|---|
| `head.rs:214` `permeability_of(litho) -> f64` | **derived from the property sheet, not a table** — call it with a `MaterialId` and it works. Call sites `:279`, `:295` read `u.species` and pass it straight through |
| `flux.rs:399-408` `FluxEntry::load` | **deliberately species-blind** — the doc names the discarded per-species split and its heir (the fluvial record-terms slice). Nothing to widen |
| `inventory.rs` | one point lookup, no per-species stock (§ 0) |
| `dc-core/src/coarse.rs:151` `ShareVec<const N>` | the generic itself is width-agnostic; only the **instantiations** (C2, C7) are pinned |
| `lithology.rs:333` `resistance_of_material(m: MaterialId)` | already takes a `MaterialId`. `LithoResistance` is a per-**(material, agent)** model; only the `Litho`-indexed table around it (C3) is class-grade |

### 1.2 `species` as a field — where identity actually flows

| site | what |
|---|---|
| `recorder.rs:273` | the field. `DepUnit { tag, thickness_m, unconformity, chapter, species }` |
| `recorder.rs:326-344` `deposit_as` | the **only** writer that states an identity rather than deriving it; `species` **joins the merge key** (`:331`) — two runs of the same environment that delivered different rock are two units |
| `recorder.rs:303-305` `deposit` | every other depositor (wind, wave, biotic, tests) takes `litho_of_tag(tag)` — the tag-derived default |
| `recorder.rs:386` `overprint_top` | pedogenesis **overwrites** the transported identity with the tag's own species (in-place alteration) |
| `recorder.rs:530` `promote_coal` | burial diagenesis rewrites species when peat→coal |
| `erosion.rs:1160` `arriving_species` | **the argmax** — one `Litho` out of the whole `dep_sp + creep_sp` mixture. STUB #25 |
| `lithology.rs:489` `window_walk` | reads `u.species` (not the tag) for the outcrop window |
| `collapse.rs:1664` `top_voxel_shares` | reads `u.species` → `deep_class_slot` → far-field share vector |
| `geology.rs:701` `deposit_deep_history` | reads `u.species` → `deep_class_of_species` → `select` → the expressed member |
| `inventory.rs:1146` `derive_base` | reads `u.species.reference_material()` → the ledger's base composition |
| `head.rs:279`, `:295` | `permeability_of(u.species)` → the harmonic-mean vertical `k` and the transmissivity/aquitard cap. **Property-sheet derived — survives member grade unchanged (§ 1.1a)** |
| `erosion.rs:1115` `record_cell(.., species)` → `:1120` `deposit_as` | the writer's caller |
| `erosion.rs:3490-3511` `species_at` closure | the scalar and parallel record paths |

**`DepUnit::species` is read at 8 production sites and written at 4** (`recorder.rs:341`
new unit, `:373` tag-derived default, `:386` pedogenic overprint, `:530` coal promotion),
plus 2 coalescing-guard comparisons (`recorder.rs:331`, `:392`) and one test fixture
(`geology.rs:1017`). **It is the only field of type `Litho` anywhere in `crates/`.** That
is the whole surface P11 re-grades at the record; everything else in the 308 is the *class
tables* (C1–C7) and their duplicates (C12).

---

## 2. Design question 1 — REPRESENTATION (options, not a pick)

The fork underneath all of them: **which identity does the record name?**

- **`MaterialId`** (26, closed compile-time enum, 1 byte) — what the ROADMAP says
  (`:361`, *"registry `MaterialId`s — mudstone, siltstone, sandstone…"*) and what the
  inventory/contents tiers already speak.
- **`GeoMemberIdx`** (u16, 14 in vanilla, **open per `GeologySet`**) — what the fitness
  machinery produces and what `StrataEvent::member` records at the collapse tier.

They are not equivalent. `GeoMemberDef.material` is a *function* member → material
(`geology.rs:193`); the inverse is not a function in general (two members of different
classes may share a material; today none do — ⚠ unverified as an invariant, no test
forbids it). **Recording the `MaterialId` loses which class/member produced it; recording
the `GeoMemberIdx` couples the deep record to pack content ordering.** This choice is
upstream of every option below and is a **user call** (it is the engine/pack partition line
the P11 graph row says the design pass must draw).

### Option A — **identity per unit** (`DepUnit::species: MaterialId`)

The minimal, literal reading of the ruling. `arriving_species`'s argmax now produces a
member identity instead of a class; fitness runs **once, at deposition**, under the
formation context of the epoch that laid the bed.

- **What the fitness machinery becomes.** `GeologySet::select` moves from `collapse.rs` /
  `geology.rs::deposit_deep_history` into the deep tier, called at deposition time from
  `arriving_species` (or from `deposit_as`). Its `FormationContext` becomes the *actual*
  at-deposition state the deep sim already holds — the marched `temp_c`, the aridity that
  produced `DepTag::aridity`, the burial depth at that instant.
- **⚠ This structurally fixes stub #31** (`stubs.md:1393-1401`, the formation context
  frozen at the chunk centre). #31 exists because `collapse.rs::column` samples
  `temp_c`/`precip`/`depth_m` once per 28.8 m chunk and the fitness *thresholds* step on
  that grid. If fitness runs at deposition, there is no chunk to freeze it at — the deep
  cell's own measured context is the context. **The heir #31 names ("per-column formation
  context, naturally part of the near-path record restructure") is superseded by a
  different and stronger mechanism.** That is a disposition the ripple map must carry
  (§ 7), and it is the strongest argument for Option A over the alternatives.
- **What expression still does.** Everything refinement Law 1 says it may
  (`refinement.md:189-198`): the *dither* survives as a **sub-cell membership dither**, not
  as an identity invention. Today `dithered_member` (`geology.rs:954`) re-runs `select`
  per voxel column; under Option A it would instead interpolate/dither between the
  **recorded** identities of neighbouring deep cells — which is exactly move B of the
  `CoarseField` contract, and exactly what the far field already does at
  `collapse.rs::surface_class`. Expression expresses; it stops inventing.
- **Cost:** **0 bytes** (§ 3.1). Deep-tier `select` calls go from ~1 per (chunk, event) to
  1 per (deep cell, deposition event) — gen time, which is not a constraint by doctrine.
- **What it does NOT give:** a unit is still **one identity**. A bed that is 60 % sandstone
  / 40 % siltstone records as sandstone. The composition the record-terms slice wants
  (ruling 1) is a *face* quantity, not a unit quantity — see Option B.
- **Loss:** `Litho::Basement` has no `MaterialId` equivalent that is honest. Today
  `as_deposited` guarantees no unit carries `Basement`, so the unit alphabet needs no
  basement value — but `exposed_shares`'s deficit **does** (`lithology.rs:500-509`). So
  the *unit* alphabet and the *window-share* alphabet diverge under Option A. **This is a
  design detail with a known good shape available: `Identity::Unrecorded` (journal/0101) —
  a "no record here" answer distinct in the TYPE, not a sentinel in the value.**

### Option B — **member DISTRIBUTION per unit** (shares over the roster)

`DepUnit` carries a share vector instead of an argmax. Retires stub #25's collapse
(*"7-vector → 1 byte"*) at the unit tier.

- **What the fitness machinery becomes.** Nothing selects at all at the unit tier —
  the transport solve's `dep_sp`/`creep_sp` mixture (which is already a per-species vector,
  `erosion.rs:1471-1490`) is *recorded* rather than argmax'd. Fitness survives only where a
  *class* still has to be filled (i.e. nowhere, if the load is already member-grade — which
  is Option E).
- **Cost:** the most expensive family. **+42.23 MiB** at 7 × u8 and **+168.94 MiB** at
  26 × u8 (§ 3.2), against a whole-field residency of 108.55 MiB. **This is the option
  the arithmetic is hostile to.**
- **Sparse variants worth pricing (assistant-proposed):** argmax + one minor + a share
  byte = 3 bytes → still +8 B/unit after alignment → the same 42.23 MiB. **There is no
  free lunch inside `DepUnit`: it has zero padding left** (`stubs.md:914-919`, verified
  here by field arithmetic — `tag` 5 + `f64` 8 + `bool` 1 + `u8` 1 + `species` 1 = 16
  exactly at align 8).
- **The honest alternative shape:** put composition on the **face** (`FluxEntry`), where
  the record-terms slice already wants it and where a parallel CSR costs 4.43–13.39 MiB
  (§ 3.4). *"What settled here"* vs *"what moved through here"* are different questions
  (priors § 1.3/§ 6.5) and P11 need not answer both at the same tier.

### Option C — **member-grade species roster replacing `Litho` throughout** (transport too)

`SPECIES` stops being `Litho::COUNT` and becomes the member/material count. The transport
solve carries member-grade loads; `settling_table` and `susceptibility_table` are per
member; the record's species field is the argmax of a member-grade mixture.

- **What it buys:** the only option under which *sorting* can discriminate members —
  `settle_energy` reads a real property sheet per member, so mudstone and siltstone finally
  differ in transport (today they are one `ClasticFine`), and the `as_deposited` /
  `reference_material` proxies all die. It is also the only option that makes
  `materials.md`'s **transitions-on-material-definitions** rule consultable by the deep sim
  — which is reason (1) in the arc's own WHY (`ROADMAP.md:369-371`).
- **Cost:** § 3.3. Four `n × SPECIES` f64 planes go **63.45 MiB → 126.9 MiB (14 members)
  → 235.7 MiB (26 materials)** of gen-time working set, plus O(SPECIES) inner loops in the
  hottest deep-time path. Gen *time* is free by doctrine; gen *memory* is not free on this
  machine (CLAUDE.md § Build rules — parallel heavy work has hung it).
- **Cost mitigation available (assistant-proposed):** the load is naturally sparse — a
  cell rarely carries more than 2–3 species. A sparse per-cell multiset (the § 13.3 shape
  `material-behavior.md:842-846` actually specifies) would decouple residency from roster
  width entirely. **But S19 § 6's warning is explicit and measured: do NOT build it as a
  `Vec` per (cell, slot)** — *"89 % of its 150 MiB heap is empty headers"*. The vindicated
  shape is CSR, which is a real slice of work.
- ⚠ **This option's calibration is not free.** `COMPETENCE_SCALE = 420` is anchored on
  *"coarse clastic's `settle_energy` at the shipped facies rule's Low/Medium boundary"*
  (`erosion.rs:379-424`) — a **class** quantity. Under member grade there is no "coarse
  clastic"; the anchor has to be re-derived against a named member, and that is a
  measure-against-the-literature obligation (§ 6), not a rename.

### Option D — **record the identity, keep `Litho` as a derived parent view**

Option A's record plus a **parent function** `parent_of(MaterialId) -> Litho` (or a
declared parent edge on the material). The deep tier's *arithmetic* — susceptibility,
settling, the window shares, the far field's `ShareVec<6>` — keeps aggregating by parent
where member grade is unaffordable; the *record* is member-grade.

- **This is the inheritance sketch the arc names** (`ROADMAP.md:381-383`: *"the
  parent-inheritance sketch … becomes parent-edge approximation with a legible error
  term — or dissolves"*), and it is the same shape as
  `material-genesis-notebook.md` § 2.3's per-parameter shadowing with parent-value access
  (**user-originated**).
- **What it buys:** every fixed-N site (C2, C3, C4, C5, C7) survives untouched, because
  the *parent* alphabet stays closed while the *record* goes open. It is the cheapest path
  to a member-grade record by a wide margin, and the one that lets P11 ship without
  re-opening the far-field register that was walked and ratified two days ago.
- **What it costs — and this is the sharpest trade-off in the document:** it re-creates a
  **summary beside an authority** (CLAUDE.md § *A summary is not an authority*). The
  discipline the doctrine names applies verbatim: *"if this consumer disappeared tomorrow,
  would this code still exist in this shape?"* A parent aggregation whose only justification
  is that member grade is expensive **is a stub**, and owes a `stubs.md` entry naming its
  heir plus a test asserting it AGREES with the member-grade authority.
- ⚠ **CONTESTS-adjacent, flag rather than resolve:** the user's 2026-07-22 ruling reads
  *"the properties of the materials themselves is what matters, **no proxy**"*
  (`materials.md:519`). A derived parent view is defensible as an *aggregation* (S-3
  sanctions approximating a declared rule) and indefensible as a *proxy identity*. **Which
  it is depends on whether any consumer can still read the parent where the member is
  available.** That distinction is the user's to draw; the audit will not draw it.

### Option E — **the load goes member-grade, the record follows** (C + A composed)

The full reading: transport carries members, the record names members, fitness runs at
deposition only where a *class contract* still needs filling (which, at the limit, is
nowhere — P5's *"retire the class-member-fitness abstraction … derive fitness purely from
properties stored on each material"*, `ROADMAP.md:2003`).

- **This is the option that unifies P11 with P5** and with the north star's *"one authoring
  shape"*. It is also the largest, and it inherits P5's **`🔖 OPEN EDGE`** gate
  (`ROADMAP.md:1986` — *read the genesis notebook before dispatching*), which is not P11's
  to open.
- Cost = Option C's memory + Option A's zero + P5's whole design surface.

### Summary of the representation fork

| option | record names | fitness runs | Δ residency | fixed-N sites survive? | fixes #31? | retires #25? |
|---|---|---|---:|---|---|---|
| **A** identity/unit | `MaterialId` (or `GeoMemberIdx`) | **deposition** | **0 MiB** | only via a parent view (→ D) | **yes, structurally** | no (still an argmax) |
| **B** distribution/unit | shares over roster | none at unit tier | +42.2 … +168.9 MiB | no | yes | **yes** |
| **C** member-grade transport | — | — | +63.4 … +172.2 MiB gen scratch | no | — | partially |
| **D** A + parent view | `MaterialId` + `parent_of` | deposition | **0 MiB** | **yes** | yes | no |
| **E** C + A | member | deposition / nowhere | C + 0 | no | yes | yes |

---

## 3. Design question 2 — the RESIDENCY ENVELOPE

### 3.0 ⚠ The counts, and which are stale

| quantity | value | source | staleness |
|---|---:|---|---|
| deep cells `n` | **297,025** (545², 460 m) | S19 § 1 | stable (grid geometry) |
| chapters | 8 | S19 § 1 | stable |
| **live `DepUnit`s** | **5,535,837** | S19 § 3(a); journal/0096:270-271 | ⚠ **measured 2026-07-25 — PREDATES journal/0111's ~1000× denudation recalibration and journal/0112's material creep.** Both change how much material moves and therefore how many units the run-length merge produces. **Direction of error unknown**; more erosion means more strips *and* more deposition events. **This is the weakest number in the document and every per-unit figure below inherits it.** |
| `size_of::<DepUnit>()` | **16 B, zero padding** | `stubs.md:914-919`; re-derived here from the field list | field arithmetic, not `offset_of!` — but the code comment at `recorder.rs:266-269` states it independently |
| `FluxEntry` count | **2,590,372** | journal/0096:256 | ⚠ **a lower bound** — FLOW continuation (a) populated vertical faces (priors § 4.1) |
| entries with `load > 0` | **494,296** (19.08 %) | journal/0096:296-298 | same caveat |
| `DeepField` w/o record | **108.55 MiB** (merged main) | journal/0096:266-267 | 2026-07-25 |
| flow record | **40.66 MiB** | journal/0096:255-258 | 2026-07-25 |
| fact ledger | **9.57 MiB** (1,033,189 facts) | S20 header, re-measured post-0108 | 2026-07-25 |

**Recommendation (assistant-proposed): re-run `flow_cost_probe` / `flux_record_probe` /
the strata itemisation BEFORE committing to any per-unit shape.** They are gate-carrying
examples (`[[example]] test = true`) and the cost is one probe run, against a decision that
multiplies by 5.5 million.

### 3.1 The per-unit species field — the zero-cost result

`DepUnit` field arithmetic (align 8, `f64` forces it):

```
tag: DepTag        5 B  (5 fieldless enums × 1)
thickness_m: f64   8 B
unconformity: bool 1 B
chapter: u8        1 B
species: Litho     1 B
                  ---
                  16 B, 0 padding
```

| species field | width | `size_of::<DepUnit>()` | Δ/unit | added residency @ 5.54 M units |
|---|---:|---:|---:|---:|
| `Litho` (today) | 1 B | 16 | — | — |
| **`MaterialId`** | **1 B** | **16** | **0** | **0 MiB** |
| a packed member byte (≤256 members) | 1 B | 16 | 0 | **0 MiB** |
| `GeoMemberIdx` (u16) | 2 B | **24** | +8 | **42.23 MiB** (+38.9 % of the 108.55 MiB field) |
| `(species: MaterialId, mover: u8)` — stub #25's heir, unpacked | 2 B | 24 | +8 | 42.23 MiB |
| `(species, mover)` **packed into one byte** — #25's own named free alternative | 1 B | 16 | 0 | **0 MiB**, but only ≤ 5 bits of species (32 members) |

**The headline: a `MaterialId`-grade unit identity is free, and a `GeoMemberIdx`-grade one
costs 42 MiB.** That is a direct argument for the ROADMAP's own wording (registry
`MaterialId`s) over the geology-set index — **on residency grounds only**; the partition
argument runs the other way (§ 2) and is the user's.

⚠ **One byte caps the roster at 256** — and STUB #21's `EdgeId` packing already caps the
*material registry* at **51** (`inventory.rs:250-255`, compile-asserted). So the binding
constraint on "how many members can deep history name" is **not** `DepUnit`; it is the
inventory's edge encoding, which already has a named heir (interned per-world dictionary,
S20 § 3.2). Worth stating in the ratification: **P11 at `MaterialId` grade does not trip
#21, but the pack-registration ambition that motivates P11 does.**

### 3.2 Per-unit DISTRIBUTION (Option B)

| shares over | payload | `size_of::<DepUnit>()` | Δ/unit | added residency |
|---|---:|---:|---:|---:|
| 7 (`Litho`, u8 quantised) | 7 B | 24 | +8 | **42.23 MiB** |
| 14 (vanilla members, u8) | 14 B | 32 | +16 | **84.47 MiB** |
| 26 (`MATERIAL_COUNT`, u8) | 26 B | 48 | +32 | **168.94 MiB** |
| 7 × f32 exact | 28 B | 48 | +32 | 168.94 MiB |
| argmax + minor + share (3 B) | 3 B | 24 | +8 | 42.23 MiB |

Arithmetic: 5,535,837 × 8 = 44,286,696 B = **42.23 MiB**; × 16 = **84.47 MiB**;
× 32 = **168.94 MiB**. (The 42.23 figure reproduces the priors audit § 4.5 and stubs #25
independently, which is the check.)

**Against the 108.55 MiB merged-main field, the cheapest distribution is +38.9 % and the
member-grade one is +155.6 %.** Option B prices itself out at any roster width unless it
displaces something.

### 3.3 Transport budgets (Option C) — the largest number in the document

`erosion.rs` holds four `n × SPECIES` `Vec<f64>` planes: `qs_sp` (`:1465`), `dep_sp`
(`:1474`), `creep_sp` (`:1490`), `shares` (`:2169`). At `n = 297,025`:

| SPECIES | per plane | four planes | Δ vs today |
|---:|---:|---:|---:|
| **7** (today, `Litho::COUNT`) | 15.86 MiB | **63.45 MiB** | — |
| 14 (vanilla members) | 31.73 MiB | **126.90 MiB** | **+63.45 MiB** |
| 26 (`MATERIAL_COUNT`) | 58.92 MiB | **235.68 MiB** | **+172.23 MiB** |
| 51 (STUB #21's ceiling) | 115.57 MiB | 462.28 MiB | +398.83 MiB |

Arithmetic: 297,025 × 7 × 8 = 16,633,400 B = 15.86 MiB; × 14 = 31.73; × 26 = 58.92.

**These are GEN-TIME working set, not shipped residency** — the planes live in the erosion
solver, not in `DeepField`. That matters two ways: (a) doctrine says gen time is free but
says nothing about gen *memory*, and this machine's build rules exist because memory
pressure has hung it; (b) they are the honest cost of Option C and they do not appear in
any `DeepField` itemisation, so a probe that measures shipped residency will report **zero
change** and be wrong about the machine. ⚠ **Flag for whoever builds this: the residency
probe cannot see Option C's cost.**

Also `[f64; SPECIES]` stack arrays: `w_settle`/`ws_order`/`bedrock_sp` (`:1443-1453`),
`split_by_shares`'s return (`:482`), `arriving_species`'s `mix` (`:1161`),
`window_walk`'s `acc`/`order`/`seen` (`lithology.rs:468-473`). At 26 these are still small
per call but they are in **per-cell** loops.

### 3.4 The face composition (the record-terms slice, ruling 1) re-derived at member grade

Priors § 4.3/§ 4.4 priced this at 7 species. Re-derived:

| shape | 7 species | 14 members | 26 materials |
|---|---:|---:|---:|
| into `FluxEntry`'s 2 padding bytes | ✗ (needs 7 B) | ✗ | ✗ |
| widen `FluxEntry` (u8 shares) | 24 B, **19.76 MiB** | 32 B, **39.53 MiB** | 44 B, **69.17 MiB** |
| parallel CSR over loaded entries (u8) | **4.43 MiB** | **7.73 MiB** | **13.39 MiB** |
| sparse side array + `u32` index (u8) | 5.66 MiB (12 B rows) | 9.43 MiB (20 B rows) | 15.08 MiB (32 B rows) |

Arithmetic: 494,296 × 14 = 6,920,144 B = 6.60 MiB + 1.13 CSR index = **7.73 MiB**;
× 26 = 12,851,696 B = 12.26 + 1.13 = **13.39 MiB**. Widening: 2,590,372 × 16 = **39.53
MiB**; × 28 = **69.17 MiB**. Side-array rows are `shares + u32` rounded to align 4:
494,296 × 20 = **9.43 MiB**, × 32 = **15.08 MiB**.

**The parallel-CSR ranking survives the re-grade** — it is still 3–5× cheaper than widening
the entry, at every roster width. Ruling 3 (face-vs-unit) is HELD on P11; the arithmetic
says the fork's *shape* answer does not change, only its magnitude. **A sparse
`(member, share)` list would decouple this from roster width entirely** (assistant-proposed;
the load's true sparsity per face is **unmeasured** — ⚠ flagged, and it is the one cheap
measurement that would settle the whole cost family).

### 3.5 What is FREE

| consumer | change | Δ residency |
|---|---|---:|
| the fact ledger / deep-cell inventory | `derive_base` stops calling `reference_material()` and reads the recorded identity (`inventory.rs:1146`) | **0** — `Portion` is already `MaterialId` |
| `VoxelContents` | nothing; already `[MaterialId; 8]` | **0** |
| `StrataEvent::member` | nothing at the collapse tier; it may *lose* its `select` call | **0** (possibly negative) |
| `EdgeId` / `Fact` | nothing at ≤ 51 materials | **0** |
| `head.rs::permeability_of` | nothing — property-sheet derived (§ 1.1a) | **0** |
| `FluxEntry` | nothing — deliberately species-blind (`flux.rs:399-408`) | **0** |
| `erosion.rs:1386` `litho: Vec<u8>` exposure plane | a byte is a byte at ≤ 256 members | **0** |

### 3.6 Fixed-N shapes and the wire discipline

- **`ShareVec<const N: usize>`** (`dc-core/src/coarse.rs:151`) is a `[f64; N]`. `N` must be
  a compile-time constant. Open-ended member grade is **incompatible with it as written**.
  Three answers, all with costs: (i) keep a fixed *parent* alphabet at these sites
  (Option D); (ii) pick a compile-time max and pay `N × 8` bytes per sample —
  at 26 that is 208 B per `ShareVec`, and `build_class_window` builds 16 of them per chunk
  (`collapse.rs:1678-1684`); (iii) a sparse share type, which is a **new `Interpolable`
  implementor** and therefore a genuine engine-tier design (E5), not a P11 detail.
- **Wire discipline (corrections #3): postcard is positional, no `skip_serializing_if`.**
  `DepUnit` is not serialized today (⚠ verified only by absence of a `Serialize` derive on
  `DepUnit`/`DeepStrata` — `recorder.rs:220`, `:279`). `MaterialId` *is* serialized, as the
  raw `u8`, with a hand-written impl specifically to keep the wire byte-identical
  (`dc-core/src/materials/mod.rs:216-230`). So a `MaterialId`-grade record inherits an
  already-disciplined wire type; a `GeoMemberIdx`-grade one would need the same treatment
  written from scratch **and** would put pack content ordering on the wire.
- **The append-last convention** (`recorder.rs:169`, `:271`) is how `chapter` and `species`
  were added without moving `size_of`. A field *replacement* (Litho → MaterialId) is not an
  append and is a **golden-moving change** — legitimate under the scratch-pad doctrine, with
  the why recorded.

---

## 4. Design question 3 — the `Litho` roster's fate

Three dispositions, with the brackets the arc itself sets (`ROADMAP.md:381-383`).

### 4a. **DISSOLVE into parent materials**

`Litho` disappears; its seven variants become **parent materials** in the hierarchy (the
`material-genesis-notebook.md` § 2.3 / `refinement.md` § 4 inheritance shape), and every
`[_; Litho::COUNT]` table becomes either per-member or per-parent-by-declaration.

- **For:** it is the literal cash-in of *"the class system's fixed-constant roster is
  scaffolding"* (`materials.md:520`) and of *"no proxy"* (`:519`). It is what makes
  transitions-on-material-definitions consultable by the deep sim without an S-3 parallel
  rule. It is P5's stated first slice (`ROADMAP.md:2046`).
- **Against:** it is the largest option, it re-opens the far field's ratified
  `ShareVec<6>`, and it inherits P5's OPEN EDGE gate.
- **⚠ It also deletes real physics unless deliberately re-homed:** `as_deposited` (C9),
  `litho_of_tag`'s environment→rock inference (the tag-derived default every non-transport
  depositor uses, `recorder.rs:304`), and the `LithoResistance` per-(rock, agent) model
  (`lithology.rs:144-219`) — the last of which is *not* class-grade at all, it already
  derives from a `MaterialId`'s property sheet via `resistance_of_material`
  (`lithology.rs:333`). **`LithoResistance` survives dissolution unharmed.** Only the
  `Litho`-indexed *table* around it (C3) is class-grade.

### 4b. **SURVIVE as a derived view** (Option D's partner)

`Litho` stays as a `parent_of(MaterialId)` aggregation, consulted only where member grade
is unaffordable (the fixed-N sites C2/C3/C4/C5/C7).

- **For:** cheapest by far; keeps every ratified fixed-N surface; the error term is legible
  and testable (a parent-blend vs a member-blend, asserted to agree at the identities).
- **Against:** the summary-is-not-an-authority test bites (§ 2 Option D). It owes a
  `stubs.md` entry, a named heir, and an agreement test — the doctrine's full price.
- **The honest framing:** this is *parent-edge approximation with a legible error term*,
  which the arc anchor already names as an acceptable outcome (`ROADMAP.md:382`).

### 4c. **SURVIVE unchanged for the deep ARITHMETIC only**

A narrower 4b: the record goes member-grade, but erosion/transport keep the 7-species
budget exactly as built, with the record's member identity carried *alongside* the class it
belongs to.

- **For:** zero risk to the calibrated solve. `COMPETENCE_SCALE`'s anchor (`erosion.rs:412`)
  survives untouched, and P2's re-pick is not entangled.
- **Against:** the record and the budget then disagree about what a species is — two
  authorities for one quantity, which is the exact shape the corpus keeps paying for.
  ⚠ It also does **not** deliver reason (1) of the arc's WHY: a 7-species transport budget
  still structurally cannot consult per-material transition edges.

### The bracket the audit will not cross

The user's **"no proxy"** (2026-07-22) and the arc's **"parent-edge approximation with a
legible error term — or dissolves"** (2026-08-01) are the two brackets, and they are in
tension by design. **Which side of it a derived `Litho` view sits on is a user ruling.**
The audit's contribution is to state the test that distinguishes them: *can any consumer
read the parent where the member is available?* If yes → proxy. If the parent is only ever
read where the member genuinely cannot be afforded, and a test pins the agreement → S-3
aggregation.

---

## 5. Design question 4 — MIGRATION

**The framing is settled and does not need re-deciding:** the testing world is a scratch
pad; goldens are regression tripwires, never ratified intent (CLAUDE.md § Conventions).
A hash move produced by ratified semantics re-captures the goldens with the why recorded.
**So there is no ratification loop on the new bytes, and no byte-identicality owed.**

What *is* an open mechanical question:

### 5a. Can class-grade records be up-converted deterministically?

**Yes, and the code already does it — twice, in two different places.**

1. `inventory.rs:1146` `derive_base`: `unit.species.reference_material()` — a **total,
   pure, seed-free** map from the 7 classes to 7 fixed materials. Deterministic by
   construction; produces the *reference* member, never a fitted one.
2. `geology.rs:726` `deposit_deep_history`: `select(class, form, u_draw)` where
   `u_draw = selection_field(seed, GeoDeep::SALT).uniform(cx·32+16, cz·32+16, k)` —
   the **fitted** up-conversion, deterministic in `(seed, salt, tag, voxel)` and stated as
   such (`geology.rs:254-256`).

So the migration options are:

| path | determinism | fidelity | cost |
|---|---|---|---|
| **replay `reference_material()`** | total, pure | every unit of a class becomes the same material — **destroys the diversity P11 exists to create** | trivial |
| **replay the fitness draw at read** | pure in `(seed, …)` — the same draw the collapse tier already makes | reproduces today's expressed world exactly | needs the *deposition-time* formation context, which **the old record does not carry** (that is precisely what P11 adds) — so it can only replay the *chunk-centre collapse-tier* context, i.e. stub #31's frozen context |
| **regenerate** | n/a | correct by construction | a world-format break in practice |

**The honest reading (assistant-proposed): this is a world-format break, and the
up-conversion paths are worse than regeneration for the thing P11 is for.** Replaying
fitness at read reproduces *yesterday's* answer under *yesterday's* frozen context — it
would migrate the defect (#31) along with the data. Since no world is persisted today
(⚠ verified only by the absence of a serializer on `DeepStrata`; the fact ledger is
likewise not serialized — S20 header, *"nothing in the tree serializes a ledger today"*),
**there is nothing to migrate**, and the question is really about the *future* format.

### 5b. What the future format needs, stated now while it is free

- The `EdgeDict` precedent (`inventory.rs:328-420`) is the shape: **identity keyed by
  qualified NAME, re-derived against the live registry, mismatch DETECTED rather than
  silently reinterpreted.** A member-grade `DepUnit` will need exactly this — a per-world
  material dictionary — the day a world is persisted, and it is far cheaper to design the
  record with that in mind than to retrofit it (S20 § 3.2 made the same argument for the
  ledger and it was ratified).
- ⚠ A `GeoMemberIdx`-grade record makes this *harder*: the index is positional in a
  **pack-assembled** `GeologySet`, so the dictionary would have to name member ids, not
  material names, and member ids are content the pack can rename.

---

## 6. Design question 5 — DETERMINISM AND CALIBRATION

### 6a. Determinism

The existing guarantees P11 must not weaken, all verified:

| guarantee | where | P11's exposure |
|---|---|---|
| all entropy from caller seeds; no wall clock | project rule; `geology.rs:257-259` states it for `selection_field` | **moving fitness to deposition means the deep sim needs its own addressed draw.** It has one — `Draws`/`Domain` salts are already used throughout `erosion.rs`. A new `Domain` (e.g. `DeepMember`) is the honest shape, not a reuse of `GeoDeep`'s salt (which addresses the *collapse* tier) |
| registration order cannot change a byte | `dc-core/materials/geology.rs:314-340` (canonical id sort) + `member_indices_are_registration_order_independent` | preserved iff the record names a **canonically ordered** identity. `MaterialId` is a compile-time enum (trivially stable); `GeoMemberIdx` is id-sorted (stable *given the same member set*) |
| adding a member diversifies a class, never inflates it | `abundance_is_normalized_within_class` | ⚠ **P11 puts this invariant at risk in a new way.** Today `lithology.rs:58-62` is explicit that the reference member is *"fixed per class rather than sampled from the live registry … adding an organism or material pack can never move terrain."* If fitness runs at deposition, **adding a pack member CAN move terrain** — because the deposited identity feeds `exposed_shares` → `susceptibility_table` → erosion rates. **This is a real consequence of P11 and it must be surfaced at ratification**, not discovered later. It is not obviously wrong (a world whose rocks are different should erode differently) but it reverses a stated design property |
| the merge key | `recorder.rs:326-335` — `species` joins it | member grade **multiplies distinct units**: two beds that merged as `ClasticFine` now split into mudstone and siltstone. **⚠ Unit count will rise, and the 5.54 M baseline is already stale.** This is the second-order residency risk in Option A and it is unpriced here because the split factor is unmeasured |

### 6b. Calibration and the literature

Per CLAUDE.md § *A closed system cannot detect its own scale error*:

- **`COMPETENCE_SCALE = 420`** (`erosion.rs:412`) is derived from *coarse clastic's*
  `settle_energy` at the facies rule's Low/Medium boundary. Under member grade the anchor
  member must be **named** and its property sheet cited. This is a derivation, not a
  re-tune — but it must be re-stated, or the constant becomes a number pretending to be a
  mechanism.
- **Anything reading "transitions per member"** — the transition-edge rates that motivate
  the arc — meets the rule head-on: a per-material weathering/transformation rate has a
  published counterpart (mineral dissolution rates, saprolite production rates), and P11 is
  the slice that first makes those rates *expressible*. **At minimum one member-grade rate
  should be checked against a published band before the arc closes.**
- ⚠ **`settle_energy` remains buoyancy-blind at member grade** (STUB #23,
  `stubs.md:789-820`): peat at 5 mm / 400 kg m⁻³ outranks sandstone. Member grade makes
  this *more* visible, not less — it does not fix it, and its heir (fluid identity,
  `flow.md` § 2.5) is unbuilt. **P11 must not be sold as fixing #23.**
- ⚠ **P10's grain-size continuum is the named heir for size, and P11 is about IDENTITY.**
  The two are easy to conflate (priors § 1.6 is the receipt). A member-grade roster gives
  mudstone ≠ siltstone as *identities*; it does **not** give a φ continuum, because
  `grain_size_mm` is still one scalar per sheet.

---

## 7. THE RIPPLE MAP

**Disposition vocabulary:** **PROCEEDS** (unaffected by P11) · **RE-GRADES** (proceeds, but
the species axis it names changes width/type — its logic survives, its arithmetic
re-derives) · **FOLDS** (should become part of P11 or be co-designed with it) ·
**HELD** (must not start until P11's representation is ratified).

The column that matters most is the last one: **what is BLOCKED vs what merely gets better.**

| # | slated item | anchor | what P11 changes | disposition options | BLOCKED by P11? |
|---|---|---|---|---|---|
| R1 | **record-terms composition slice** (rulings 1–2) | priors header `:3-18`; `ROADMAP.md:390` | the composition it records is per-`Litho` shares; P11 re-grades that axis. **Its LOGIC is untouched** — record composition, not a size proxy; nothing non-additive before WINDOW | **RE-GRADES.** Cost table re-derives (§ 3.4) but the *shape* answer (parallel CSR beats widening) survives at every width | **No — but building it first means building the widening twice.** Sequencing it after P11's representation ruling is nearly free; building it before costs a rewrite of the exact array it stores |
| R2 | **record-terms ruling 3** (face-vs-unit) | priors header `:24`; already **HELD** | whether composition lands on the face or the unit is entangled with whether the *unit* carries a distribution (Option B) | **HELD** (already so ruled). Note § 3.4: the face/CSR answer is robust to the re-grade; the *unit* answer is not (§ 3.2 prices Option B out) | **Yes, by the user's own hold** |
| R3 | **P10 grain-size continuum** | `ROADMAP.md:400-429`; graph `:78` | its size state *attaches to loads* — same class-vs-member grade question. And P10's own WHY is a class-grade symptom (*"the roster spans ~two distinct sizes, ClasticFine ≡ OrganicSoil exactly"*) | **RE-GRADES.** Its design pass is startable today and **gets materially easier after P11** — a member-grade roster gives real per-member `grain_size_mm` to build a release spectrum from. ⚠ It does **not** become unnecessary: identity ≠ size (§ 6b) | **No.** Design pass proceeds; the *state's grade* should be decided with P11, not before |
| R4 | **member-#0's near-path restructure (MM-3)** | `ROADMAP.md:1184-1200`; graph `:51`, `:72`; 13 files / ~40 sites | **the deepest entanglement on the board.** MM-3's restructure exists to fix U3 by giving the record a per-column shape; P11 may move `select` out of the collapse tier entirely, which changes what the restructure is restructuring. And Option A **structurally retires stub #31** (`stubs.md:1393`), which MM-3 names as its own heir | **FOLDS or HELD.** Two readings: (a) MM-3 is about the *record*'s sub-cell shape and survives any grade → PROCEEDS; (b) if fitness moves to deposition, the per-column formation context MM-3 carries has **no consumer left** → the slice shrinks. **This needs the representation ruling before the 13-file blast radius is spent** | **⚠ Yes, effectively.** Spending a measured 13-file / ~40-site restructure on machinery P11 may relocate is the single largest wasted-work risk in the ripple |
| R5 | **refinement member #1 (fluvial)** | graph `:51`; `ROADMAP.md:394` | its blockers are the record terms (R1) + P2. P11 sits upstream of R1 | **RE-GRADES** (via R1). Its § 7.1 assemblage — *"load budget, grain distribution, column inventory"* — reads terms, and Law 1 forbids it from inventing identity, which P11 makes easier to honour | **No, indirectly held** — already blocked on R1 and P2 |
| R6 | **P5 facies driver / genesis passes** | `ROADMAP.md:1981-2057`; graph `:73` | P5's first slice is *"convert the `DepTag→material` / `deep_class`/`dithered_member` determination into a declared pass that reads material PROPERTIES"* — **that is P11's C8 and C10, from the other end** | **FOLDS.** P11 Option E *is* P5's thesis at the deep tier. But P5 carries a **`🔖 OPEN EDGE`** gate P11 does not own | **No — but P11 should not silently pre-empt it.** If P11 picks Option E, that is a P5 decision made inside a P11 slice, which is exactly the shape corrections #65 forbids. ⚠ Flag |
| R7 | **P2 calibration re-pick** | `ROADMAP.md:1006-1029`; graph `:70` | P2 is a literature re-pick of `EROSION_CALIBRATION` against the fixed operator. P11 changes what a "species" is in the transport solve **only under Options C/E** | **PROCEEDS under A/B/D. RE-GRADES under C/E** — `COMPETENCE_SCALE`'s anchor is a class quantity (§ 6b) | **No.** P2 is unblocked, literature-only, and should run *first* if anything: a member-grade world calibrated against a broken constant is worse than a class-grade one |
| R8 | **stubs #25** (record knows what arrived, not who brought it) | `stubs.md:896-943` | #25's heir is a **packed `(species, mover)` byte, 3 bits each**. `Litho` has 7 inhabitants → 3 bits. **A `MaterialId`-grade species needs 5 bits, leaving 3 for the mover — still exactly fits one byte** | **FOLDS.** P11 is the natural carrier: the byte is being rewritten anyway, and #25's stated cost (*"a representation change across ~38 read sites"*) is P11's cost regardless | **No, but it becomes nearly free inside P11 and expensive outside it** |
| R9 | **stubs #31** (formation context frozen at the chunk centre) | `stubs.md:1393-1401` | **RESOLVED BY CONSTRUCTION under Options A/B/D/E** — if fitness runs at deposition there is no chunk-centre to freeze | **FOLDS.** ⚠ Its named heir (*"per-column formation context, part of the near-path record restructure"*) is **superseded** by a different mechanism. Whoever ships P11 owes #31 a banner in the same commit (read-first item 5) | **No — it is a beneficiary** |
| R10 | **stubs #23** (settling law cannot see buoyancy) | `stubs.md:789-820` | member grade makes the inversion *more visible* (peat vs sandstone becomes peat vs mudstone vs siltstone vs sandstone). Heir is unchanged: fluid identity, unbuilt | **PROCEEDS.** ⚠ **P11 must not be sold as fixing it** | **No** |
| R11 | **stubs #26** (channelisation thresholds fitted to one world) | `stubs.md:944-985` | untouched — χ is a topographic index, species-blind | **PROCEEDS** | **No** |
| R12 | **stubs #16** (bedrock as a flat granite structure span) | `stubs.md:524-540` | ⚠ **P11 sharpens it.** Today the inventory says *granite*-loose while the collapse expresses a clastic-fine weathering product (`geology.rs:551-559`) — a disagreement #16 owns. A member-grade record makes the basement's identity a *recordable* fact, so #16's heir gets cheaper | **PROCEEDS, improved.** Worth naming in the ratification as a beneficiary | **No** |
| R13 | **stubs #21** (`EdgeId` mixed radix, 51-material ceiling) | `inventory.rs:250-255` | P11 at 26 materials does not trip it. **The pack-registration ambition behind P11 does** | **PROCEEDS**, but P11 should state the ceiling out loud | **No** |
| R14 | **the classify/contents consumers** | `contents.rs:104-213`; `fill.rs:376` `mixed_contents` | **nothing changes.** They are already `[MaterialId; 8]`. `mixed_contents` takes `&[(GeoMemberIdx, u8)]` and immediately resolves to `def.material` (`fill.rs:381`) | **PROCEEDS** | **No** |
| R15 | **the far field / LOD path** (`surface_class`, `CoarseField<ShareVec<6>>`) | `collapse.rs:1581-1700`; graph `:72` | C7. Its alphabet is `DEEP_CLASSES = 6`, pinned to `deep_class_of_species` by test. **Its blend semantics were REJECTED by eye 2026-07-29 and ride as INTERIM** (`ROADMAP.md:2759`, user: *"the whole cake is swirled now"*), heir = a far register derived from refinement-operator budgets | **HELD-adjacent / RE-GRADES.** Widening a `ShareVec` whose semantics the user already rejected is work against a surface scheduled for replacement — the same argument that deliberately left stub #32 unfixed. **Under Option D it needs no change at all** | **No, but changing it now is probably wrong** regardless of P11 |
| R16 | **`identify(pos)` / the honest identity surface** | `ROADMAP.md:1866` | the arc that retires the stored `Block` summary in favour of a real mixture answer. P11 is the same doctrine one tier down: *stop summarising identity* | **PROCEEDS, reinforced.** `Identity::Unrecorded` is also the named shape for P11's basement/no-record case (§ 2 Option A) | **No** |
| R17 | **STRUCTURE-AWARE FINE EXPRESSION** | `ROADMAP.md:1662` | *"a deep cell's deposited material is spread ~uniformly across its column, not biased to where it physically settled"* — a member-grade record gives it more to bias | **PROCEEDS, improved** | **No** |
| R18 | **the member-dither guillotine** (Observed, unexamined since 2026-07-22) | `ROADMAP.md:1120`, `:3825`; `geology.rs:942-953` | `dithered_member` under chunk-centre formation context. **Same site as #31.** Under Option A the dither's *job* changes from "re-pick within a class" to "interpolate between recorded identities" | **FOLDS.** It stops being a defect and becomes a different mechanism | **No — it is dissolved rather than fixed** |
| R19 | **the "member stepping dominates U3" ruling** (Observed, ⚠ CONTESTS, needs the user) | `ROADMAP.md:2815-2853`; journal/0129 § 5 | P11 removes the census's teeth from the other direction: *"4 of 10 classes cap the dither at a coin flip"* stops being true when identity is recorded rather than dithered | **PROCEEDS.** The ruling is still owed and P11 does not answer it — but P11 makes the question partly moot | **No** |
| R20 | **the owed tour map** (a `Single` top span in a two-member class) | `ROADMAP.md:2851`; graph `:72` | this station exists to see the member dither. Under Option A there may be nothing to see there | **RE-GRADES.** Worth deferring until the representation is ruled — otherwise the walk measures a mechanism that is about to move | **No, but spending game time on it now is probably wasted** |
| R21 | **B0/B2 bodies arc** (the current close block's next build) | `ROADMAP.md:3986-4069`; graph § 2b | **no interaction.** Different subsystem, different tier | **PROCEEDS** | **No** |
| R22 | **`collapse.rs` decomposition / file-size chore** | graph `:143` | `collapse.rs` is 2,675 lines and P11 edits it. The decomp is downstream of E5 | **PROCEEDS**, but P11 will touch the same file — worth sequencing awareness | **No** |
| R23 | **stubs #20** (the weathering profile is a fixed shape) | `stubs.md:652-682` | #20's heir is *"the deep tier carrying the front as a depth-resolved term"*, and the entry itself says it is **sequenced-adjacent to #16's heir, since the profile depends on the rock**. A member-grade record makes "which rock" a recorded fact | **PROCEEDS, improved** — same beneficiary chain as R12 | **No** |
| R24 | **the three duplicate `Litho`→class matches** (C12) | `tests/erodibility.rs:176`, `tests/providers_common/mod.rs:419`, `examples/entry_species_probe.rs:123` | not a slated item — an **existing S-3 hazard P11 must pay**. Today the compiler catches a new `Litho` variant at all four sites; if the roster becomes *data*, it catches none of them | **FOLDS into P11's slice**: either delete the duplicates in favour of the one authority, or accept that the roster's cardinality stops being compiler-enforced | **No, but it is unbudgeted work** |

### 7.1 What the ripple map says, in three lines

1. **Only two things are genuinely blocked: ruling 3 (already held) and MM-3's restructure
   (R4) — and MM-3 is the expensive one.** Everything else re-grades, folds, or proceeds.
2. **Three stubs are beneficiaries, not casualties** — #31 resolved by construction, #25
   nearly free inside P11, #16 sharpened. That is unusual and worth stating: P11 pays for
   part of itself.
3. **The ripple is narrower than "all currently slated work" but the one wide edge is
   real:** every item that *records* a species (R1, R3) or *aggregates* one (R15) inherits
   the grade question, and the arc anchor's instruction — *nothing proceeds against the
   class-grade record without checking this map* — is doing exactly the work it should.

**One thing the map should NOT be read as saying.** "PROCEEDS" is not "unaffected by the
ratification schedule". R7 (P2) is the clearest case: it is genuinely unblocked and
literature-only, and running it **first** is the strongest available sequencing argument —
a member-grade world calibrated against a constant known to be fitted to a broken solve is
strictly worse than a class-grade one. That is an ordering opinion (assistant-proposed),
not a blocking edge, and it belongs to the user's sequence call rather than to this map.

---

## 8. NEEDS RATIFICATION

### 8a. USER-OWNED (the audit will not decide these)

| # | question | why it is the user's |
|---|---|---|
| U1 | **Which identity does the record name — `MaterialId` or `GeoMemberIdx`?** | it draws the engine/pack partition line the P11 graph row says the design pass must draw. Residency argues one way (§ 3.1: 0 MiB vs 42 MiB), the content model the other |
| U2 | **Representation: A / B / C / D / E** (§ 2) | the arc's own first-slice question |
| U3 | **The `Litho` roster's fate: dissolve, derived view, or arithmetic-only** (§ 4) | bracketed by the user's own *"no proxy"* (2026-07-22) and *"parent-edge approximation — or dissolves"* (2026-08-01), which are in tension by design |
| U4 | **Does a derived parent view count as a proxy?** (§ 4, the bracket) | a direct reading of a user ruling |
| U5 | **⚠ Is it acceptable that adding a pack member can now MOVE TERRAIN?** (§ 6a) | it reverses a stated design property (`lithology.rs:58-62`), deliberately. Not obviously wrong; definitely not an implementation detail |
| U6 | **Does P11 pre-empt P5?** (R6) | picking Option E decides a P5 question inside a P11 slice — corrections #65's exact shape. Needs to be an explicit user choice or an explicit deferral |
| U7 | **MM-3 disposition** (R4) | whether to spend a measured 13-file / ~40-site restructure before the representation is ruled |
| U8 | **The far field's `ShareVec<6>`** (R15) | its semantics are already rejected-and-riding-as-interim; whether P11 touches it at all is a sequencing call on a walked, ratified surface |

### 8b. INTEGRATOR-SETTLEABLE (defensible without a user ruling, recorded when done)

| # | item |
|---|---|
| I1 | Re-run `flow_cost_probe` / `flux_record_probe` / the strata itemisation to refresh the stale counts (§ 3.0) before any per-unit shape is committed |
| I2 | Add an `offset_of!`/`size_of` assertion pinning `DepUnit`'s layout, so the "zero padding left" claim is compiler-verified rather than arithmetic (the same gap priors § 4.2 flagged for `FluxEntry`) |
| I3 | Measure the **per-face load sparsity** (how many species a loaded face actually carries) — the one cheap number that would decide § 3.4's whole cost family |
| I4 | Measure the **merge-key split factor**: how many more `DepUnit`s a member-grade species field produces (§ 6a). Unpriced here |
| I5 | Choose the new `Domain`/salt for a deposition-time member draw (§ 6a) — a mechanical determinism decision with an established pattern |
| I6 | Stamp `stubs.md` #31 with a banner when P11's representation is ruled (read-first item 5: the correction's author stamps the target, same commit) |
| I7 | Stamp `docs/audits/2026-07-29-fluvial-record-terms-priors.md` with this audit's § 3.4 re-derivation (its header already anticipates it) |
| I8 | Refresh `docs/dependency-graph.md` § 4 "Ready to start today" — it lists neither P11 nor B0, and its *"Last full pass: 2026-07-29"* line is stale |

---

## 9. What this audit could NOT verify — flagged, not smoothed

1. ⚠ **The 5,535,837 live-unit count is 2026-07-25** and predates the 1000× denudation
   recalibration (journal/0111) and material creep (journal/0112). **Every per-unit MiB
   figure in § 3.1–3.2 inherits this.** Direction of error unknown.
2. ⚠ **`size_of::<DepUnit>() == 16` with zero padding** is field arithmetic here, not a
   compiler fact. It agrees with `recorder.rs:266-269` and `stubs.md:914-919`, which is the
   check, but no `offset_of!` was run (no cargo, per brief).
3. ⚠ **Whether two `GeoMemberDef`s may share a `MaterialId`** is not forbidden by any
   validator I found (`GeologySetBuilder::add_member`, `dc-core/materials/geology.rs:283-311`,
   checks id uniqueness, not material uniqueness). Today none do. If P11 records
   `MaterialId`, this becomes load-bearing.
4. ⚠ **`DepUnit`/`DeepStrata` are not serialized today** — asserted from the absence of a
   `Serialize` derive at `recorder.rs:220`/`:279`, not from a full consumer trace.
5. ⚠ **The per-face species sparsity is unmeasured** (I3). § 3.4's dense-share pricing may
   be a large overestimate.
6. ⚠ **The merge-key split factor is unmeasured** (I4) — a second-order residency risk in
   the cheapest option.
7. ⚠ **Option C's gen-memory cost is invisible to the residency probes** (§ 3.3) — they
   measure `DeepField`, and the four `n × SPECIES` planes are solver-local.
8. ⚠ **`COMPETENCE_SCALE`'s re-derivation under member grade is stated as an obligation,
   not performed** (§ 6b). It needs a named anchor member and a literature check.
9. ⚠ The claim that P11 Option A *"structurally fixes stub #31"* (R9) is this audit's
   reasoning from the mechanism, not a measured result. It is the audit's strongest
   original argument and therefore the one most worth attacking.

---

*Author: read-only design-pass agent, 2026-08-01. Read-only except this file; no cargo
invoked. Sections 2, 3.3–3.5, 4, 5a's verdict, 7's dispositions and 9 contain
assistant-originated analysis, marked inline where it goes beyond arithmetic over cited
measurements.*
