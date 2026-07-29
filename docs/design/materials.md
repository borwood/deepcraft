# Material volume model — design draft

Status: design discussion captured 2026-07-18; storage feasibility is spike S8.
Chunk format v1 (S3) reserves versioned sidecar sections for this model.

> **Companion:** material *behavior* (forms, the transition graph, agents, and the
> cellular/field pass shapes) is specified in
> [`material-behavior.md`](material-behavior.md) (ratified 2026-07-23). This doc
> owns material *definitions*; that one owns the substrate they behave over.

## The voxel as a container of eighths

A voxel is 8 volume-eighths. Each eighth holds exactly one material id (or is
empty). A voxel's loose contents are an **unordered multiset** — deposit order
is deliberately not tracked (mixing destroys information; that's what mixing
is). Three occupancy roles:

- **Structure**: a shape (full = 8 capacity, slab = 4, quarter = 2, stairs,
  …) occupied by structural materials (stones, etc.). The shape *reserves* its
  capacity in slots; filled/capacity = **density**, its complement porosity.
  A slab with 1/4 structural slots filled is porous and brittle. Fill may be
  heterogeneous (rubble/breccia — mixed stone types in one structure; the
  gameplay mechanics of *how* heterogeneous fill arises are open design).
- **Debris**: loose granular material (sand, gravel, snow, leaves, potsherds,
  knapping waste, …) filling unreserved volume in eighth increments, freely
  mixed. Renders as heightmap-blended layers of the constituents.
- **Pore occupants**: the structure's *unfilled reserved slots* can admit
  fine materials (mud, silt — a pore-filler type/subtype) and fluids.
  Packed pores alter the structure: insulation, permeability (waterproofing),
  perhaps strength/weight. Pack a crumbling wall with mud = wattle-and-daub,
  emergent from the physics rather than a recipe.

Fluids occupy empty eighths and pores (aquifers in porous stone, waterlogged
debris, quicksand — free consequences of the model).

## Packing

Two paths into pores:

1. **Deliberate**: a player/NPC action packs suitable debris into an adjacent
   structure's pores (tool/skill TBD — game design open).
2. **Overflow**: when new debris arrives and the cell has no free eighths,
   fines whose grain size fits the pores are shifted into them before
   accumulation spills to the cell above. (Falling debris checks: free debris
   slots → packable pores → stack above.)

## Granular material property sheet

Each granular material carries: **density** (stratification sort key, weight),
**grain size** (what fits into which pores; sieving), **cohesion** (angle of
repose, slumping), **per-damage-type extraction resistance** (see below),
**permeability** and **insulation** contributions when packed.

## Extraction

Block damage is *typed* (dig, chop, smash, cut, sieve, …). Sustained
application of a damage type yields materials in ascending resistance order
under that type — leaves come out of a leaf/sand/gravel mix almost
immediately, then sand, then gravel. Tool choice reorders and can destroy:
a careless pick pulverizes the potsherds a trowel would recover — archaeology
lives in the extraction mechanics, not a minigame.

## Stratification is derived, not ticked

Stratification degree = pure function of (contents, time since last
disturbance, agitation history from the ledger). Never simulated per-voxel:

- **Worldgen deposits** arrive fully stratified (deep time baked in).
- **Player-era deposits** stratify lazily — computed on observation from
  time-undisturbed; heavies band downward by density.
- **Agitation** (water flow, vibration, digging) resets/accelerates the clock.

Consequence: deposits are *readable history* — clean bands = old and
undisturbed; chaotic mix = recent activity. Mining a stratified deposit yields
clean sequential bands; a fresh mix yields by extraction resistance.

Compaction closes the deep-time loop: debris under overburden, over ledger
time, migrates into a structure slot as sedimentary stone. Worldgen strata,
gameplay middens, and geology are one process at different tick rates.

## Storage — RESOLVED by S8 (2026-07-18): GO on free-form mixtures

The feared gradient explosion is **combinatorially impossible** for small
material sets: eighth quantization caps a k-material locale at `C(k+8,8)−1`
distinct canonical states. Measured: 2-material processes (wind, scree)
saturate at exactly 45 states and stay flat forever after; an adversarial
continuous 4-material gradient caps at exactly 495. Real deposit chunks cost
0.14–0.38 B/m³ (vs 0.144 debris-free, 2.74 raw); even 12-material uniform
noise — which no plausible process produces — stays 2.9× under raw.
Debris-free chunks attach no sidecar and pay zero bytes.

Implementation (dc-core::materials, sidecars `materials/slots-v0` +
`materials/mixtures-v0`): region-interned canonical mixtures, chunk palettes
indexing the intern table, riding format v1. Curated recipes are NOT needed
for storage — retained only as an optional cap-and-snap guardrail. Extraction
ordering, lazy stratification, and the mixed-voxel LOD rule are implemented
and property-tested; details in docs/spikes/S8-results.md.

Still owed by later work: sparse sidecar encoding for thin drapes (index
array, not the table, dominates cost), region-table lifecycle/compaction,
angle-of-repose settling, the render-blend prototype (S4 input), pore-packing
and heterogeneous-structure-fill mechanics.

## Materials and forms (added 2026-07-18)

A **material** is one identity with one property sheet. A **form** is a
presentation of it in context:

- **Emplaced**: structure (shape slots), debris eighths, pore fill — above.
- **Item**: discrete object — world pickup (a dc-physics rigidbody), in
  inventory, in hand/socket.

**Form archetypes are registry entries binding to material classes** (chunk,
shard, brick, powder, …) — the same contract pattern as geology passes
(docs/design/geology.md). New material × existing archetypes = its whole item
family for free; a new archetype retroactively covers every qualifying
material.

**Breaking is a sampled, conservative distribution.** Typed damage × the
material's fracture stats × tool → a seeded (replay-deterministic) sample
over outcomes; mass that doesn't become discrete drops **remains in the voxel
as debris eighths** (mining brittle shale leaves a shale drape in the hole —
sievable, shovelable, or left for archaeologists). Nothing is deleted;
middens happen by physics. Granular/sub-threshold outcomes deposit as debris;
discrete outcomes persist as rigidbody pickups with the sleep→inert-item
handoff (S6).

**Terminology — noted 2026-07-19**: the design term for these unconsolidated
eighths is **loose (granular) materials** — "debris" implies a provenance
(broken from something) that fresh snow, dune sand, or river gravel don't
have. The S8 *storage role* keyword stays `debris` (decided wire format);
the rename is vocabulary, not bytes.

**Loose-material mechanics (user sketches 2026-07-19, future work)**:
- **Gravity by vertical march** — unsupported loose eighths fall
  column-wise (the loose-snow/sand/gravel Minecraft behavior), never as
  rigidbodies; deterministic, tick-boundary, cheap.
- **Body interaction/compaction** — a body standing on loose material can
  transform the bottom layer in place (loose snow → packed snow under a
  boot) when solid backing exists beneath, leaving the rest loose. Pairs
  with the deferred "sinking in partials" movement rules (visuals.md §
  mixture road). Compaction-by-use is the shallow-time cousin of the
  deep-time compaction loop above.

**Bodies/sockets**: designed — see docs/design/bodies.md (body plans,
sockets, animation, all ratified state lives there).

**Inventory — DECIDED 2026-07-18: mass/volume with encumbrance, behind a
realism knob.** The foundations (every form has real mass/volume; carry
capacity; encumbrance effects) are built once; a gameplay-mode knob scales
them from Minecraft-breezy to hardcore sim (a full granite voxel is a
two-handed drag or a cart job). Development default: tuned way low so it
never impedes testing. Same knob doctrine as world extent.

**Fracture — DECIDED 2026-07-18: per-damage-type outcome weights**, not one
brittleness scalar. Each (material, damage type) row carries weights over
outcome forms (smash → mostly shards; careful dig → mostly chunks) — extends
S8's typed-resistance table, and makes tool choice narratively legible.

## Open questions

- Heterogeneous structure fill: what gameplay produces it (construction with
  mixed rubble? partial mineral replacement over deep time?).
- Packing actions: tool, skill, NPC labor integration.
- Stratification rate constants; does *active* sorting (alluvial deposition)
  get its own fast path near water?
- Pore size model: single scalar from structure material + porosity, or
  per-material pore spectra?
- Freeze–thaw: water packed in pores + cold → cracking/spalling (delicious,
  deferred).

## Pore packability — DECIDED 2026-07-20 (user)

**Rule:** a material may be packed into a host's pores iff
`filler_grain_size ≤ K_PORE × host_grain_size`, one constant `K_PORE ≈ 0.25`
(derived: ideal-packing interstices fit 0.22–0.41 D; the geotechnical filter
criterion lands at D/4–D/5). For heterogeneous structure fill the host grain
is the **minimum** grain among structural components (finest grains set the
throat). Enforcement lives in a single shared `fits_in_pores(filler, host)`
helper consulted by every *transport-time* depositing process —
`VoxelContents` stays pure volume accounting by design.

**Exemption:** formation-context emplacement (worldgen authored — magmatic/
diagenetic inclusion, e.g. olivine in basalt) bypasses the mechanical rule:
those crystals grew in place; the pore is representational (3d decision).
The rule governs infiltration, not genesis.

**Composability:** sieve resistance ≡ grain size (registry invariant), so
what-packs-in and what-sieves-out-first are the same axis by construction.

## DECIDED 2026-07-21 (user, journal/0049 tour) — sand first; partials-first emission

**Sand is the first loose material to spawn in the world, emitted as
partials** (dune/loess blankets express as full voxels + a partial-height
top, not quantized-away). The substrate is largely built: S8 loose
mechanics + journal/0010's dormant partial-height loose rendering. And the
direction it generalizes: **"everything should be spawned in partials"** —
world gen emitting partial occupancies as the default, with a **forms
mechanism across all systems** (partials in structures, loose volumes,
bedforms — variety that breaks the cube matrix). The forms pass is
design-pass-sized and Sequenced; this entry records the direction so no
slice quantizes away something partials should carry.

## The forms design pass — ratifications of 2026-07-21 (live session)

The pass opened as a main-session conversation (ROADMAP § Sequenced,
journal/0049 station 2). Decisions landed so far, each from the user in
conversation:

1. **Fractions come only from the ledger (RATIFIED).** Partial occupancy
   is the expression of *recorded* quantity (the record's metres → span
   eighths + remainder) and *recorded* variance — never cosmetic noise.
   This scopes the defect-jitter idea (ideas.md § rock is not monolithic)
   as **property-dependent expression of real record variance**, user:
   "i think i agree with you here re: property-dependent expression of
   real record variance." Partials-first emission is consuming the
   ledger, not decorating it.

2. **Sand is a FORM of an existing clastic material (DECIDED).** No new
   "sand" identity: sand is the loose form of the clastic-coarse member —
   "one material has one property [sheet]"; within-identity grain-size
   gradation is not modelled (revisit later if a wall appears; user: "go
   with forms. maybe it gets revisited at a later time but go with
   form"). **Known consequence, accepted:** loose clastic is currently
   textured identically to structural — indistinguishable in a cut face —
   which implies a later *visual* decision (form-dependent texture
   variants; files to the visuals road, not this pass).

3. **The soil model (user).** Anything with roots carries a **root
   material in STRUCTURE form** — porous, holding the soil in its pores.
   This is an anti-erosion model for free: loose soil with no matrix
   falls with gravity; the root lattice is what holds a slope. Soil
   placements come **from simulation, in both forms**: layers of
   structural dirt (most structure slots filled) with loose layers
   typically above, and packing downward into porous rock below (the
   packable-soil thread, ideas.md § soil is loose but packable).

4. **Grass is suspended — do not express it (DECIDED).** Grass/turf is an
   ecology question: a *state riding on the substrate materials* of a
   block / its loose eighths, deferred to the ecology pass. The forms
   slices express the materials the sim says are present and do NOT paint
   grass. **Appearance change pre-ratified by the user: "it'll be a
   mostly brown world for a bit."** (This names the heir direction for
   the surface-veneer stub — stubs.md § 2 — but the veneer's retirement
   is its own slice, not a side effect.) Turf presentation sketch,
   captured for later, NOT to be built now: **ligatures + anisotropic
   variants** — a seeded dirt material occupying loose eighths or packed
   pores whose green displays only on the top surface, side faces getting
   a roots-texture additive instead. Non-grasslike ground cover:
   thoughts suspended entirely for later passes.

5. **The block-tier contract for partial voxels — RESOLVED, see
   ARCHITECTURE.md § "The fill contract".** The user's instinct (the block
   "reports its honest fractions") became the ratified contract: contents
   are authoritative, `classify(contents) -> Block` is a pure derivation,
   and `block == classify(contents)` is the regression invariant the user
   asked for.

   **The consumer audit that grounded it** (2026-07-21; 135 block-consuming
   sites across 31 files, 80 of them solidity-shaped — `!= Block::Air`,
   `== Block::Air`, `is_solid` — across 27 files). Who asks, sorted by what
   they actually want:

   - **"Can I stand on / pass through it?"** — `dc-core/collision.rs`'s
     trait (consumed by dc-physics), `authority.rs::is_solid_voxel`
     (player collision, grounding), walker-safety and spawn placement,
     raycasts (edit targeting, character senses). All are `!= Air`. They
     want an **occupancy threshold**, and visuals.md already reserved the
     answer ("solid ≥ 4/8").
   - **"Where is the surface?"** — the host surface scan, the authority's
     edit-allowance ceiling, `dc-core/lod.rs`'s column summarizer.
     Partials change their *answer* (a 3/8 sand top IS the surface) but
     not their question. `ColumnSpan` already models a column as a
     potential stack, so the far field anticipated this.
   - **"What do I draw?"** — the mesher and farmesh. This is where
     journal/0010's trust gate **inverts**: the block currently decides
     whether contents are believed, which must reverse or become a
     two-opinions bug factory.
   - **"How do I store it?"** — palette compression, `is_empty`, format
     v1. Semantics-indifferent; wants the atom small and positional.
   - **The future asker that already exists** — the S11 water spike's
     `water/vox.rs::is_solid` asks the block *today*. Promoted water does
     not want a bool: it wants **free capacity in eighths** (water fills
     empty eighths and pores). So does the loose-gravity march ("can the
     voxel below receive eighths?"), compaction (overburden on what?),
     and sim light (partial occlusion). Four future systems, one wanted
     primitive — which is the argument that carried the decision.

## The fill contract AS BUILT — 2026-07-21 (journal/0052)

The ratified contract above is implemented; this section records what shipped,
what it is scoped to, and what is still open. **Nothing here is a new user
ratification** — it is the engineering record of the decision already made.

### `classify` — implemented

`dc_core::classify::classify(&VoxelContents) -> Block`, pure, total, and
order-independent (it takes canonical `VoxelContents`; the order-dependent
`MixtureId` never enters — journal/0010's landmine rules unchanged). The rule
in two steps:

1. **Which multiset speaks**: structural fill, else debris, else pore fill.
   Structure is what a voxel *is*, so granite with an olivine pore inclusion is
   granite; with no structure the loose fill speaks, so a sand blanket is sand;
   pore fill decides only in the degenerate hollow-shell-packed-with-mud case.
2. **Who wins inside it**: the most abundant material, ties to the lowest
   material id. Both the canonical segment sort and the tie-break make this
   order-independent by construction.

> **⚠ `block_twin` NO LONGER EXISTS — deleted 2026-07-23 (journal/0087, the block↔material
> collapse).** Pointer added 2026-07-29 (baseline sweep S3/F2). This § *AS BUILT* block is an
> engineering record of **2026-07-21** written in the present tense; § *ONE NAMESPACE: BLOCK IS
> MATERIAL* below (DECIDED 2026-07-22, user) says so explicitly — *"the fifteen-name
> `block_twin` match and its `_ => Stone` arm die; no twin field is ever built"* — and
> `crates/dc-core/src/classify.rs` now states *"there is no `block_twin` re-translation any
> more."* **Read the paragraph below for its ARGUMENT, not its API:** it is the only place in
> the corpus that explains *why two members of one content class summarise to one name*, and
> that reasoning survives the deletion intact.

Then `block_twin(material) -> Block`, a **material→block table**, not a
class→block table. That relocation is the load-bearing bit: two members of one
content class (mudstone/siltstone, sandstone/conglomerate) summarize to the
same block because their *materials* do, not because a `GeologySet` said so —
which is what lets `classify` be pure over contents and independent of the
registry while still reproducing the retired class table's answer for every
member any registered pack has used. Loose materials fold into their lithified
twin's band (sand/gravel → Sandstone, silt/clay → Mudstone, loam → Dirt);
materials with no twin in today's block vocabulary (snow, ash, scree, bone,
potsherd, knapping debris, and the accessory grains gold-dust/olivine, which
never dominate a voxel) fall back to `Block::Stone` — the same coarse fallback
the class table used for unknown classes.

### Occupancy primitives — implemented

On `VoxelContents`, alongside the existing `solid_eighths` / `open_pores` /
`free_debris_eighths` / `structure_density`: `free_eighths`, `is_full`,
`loose_eighths`, `bound_eighths`, `has_structure`, `is_loose_only`,
`is_occupancy_solid`, and the constant `SOLID_EIGHTHS = 4` (visuals.md's
reserved "solid ≥ 4/8"). Their doc block names the four consumers they exist to
serve as ONE answer — fluid fill, the loose gravity march, compaction, and the
sim/collision tier — so none of them re-derives occupancy privately. **The
client's collision still reads `Block::is_solid`**; rewiring it to
`is_occupancy_solid` is its own slice.

### The absent-contents rule — decided in build, flagged

ARCHITECTURE.md states the invariant for "every voxel". As built it is scoped:

> for every voxel with **non-empty** contents, `block == classify(contents)`.

Voxels the generator never gave a contents record are **unclassified, not
classified as Air**: air above the surface, the surface-veneer stub block
(stubs.md § 2), the legacy soil band and unrecorded basement below the
deep-time record, ocean floor, and the border wilds ~~, and ruin posts~~ (**the
ruin posts were deleted 2026-07-28**, journal/0121). `classify`
does answer `Block::Air` for empty contents, but the generator does not apply
it there. The exception is *enumerated and pinned by test* — a geology block
appearing without a record fails the suite — so it can only shrink, and it
shrinks on its own as each stub acquires a real record.

### Still open (not built)

- **The fractional top.** The eolian blanket's real remainder as a
  partial-height loose top voxel is **deferred**: in today's column the only
  geometrically honest place for a fraction is the topmost voxel, and that
  voxel is the surface-veneer stub — whose retirement ratification 4 reserves
  for its own slice. Emitting the fraction anywhere below it opens a void under
  solid ground. journal/0052 carries the follow-on spec.
- Collision / water / sim light reading `is_occupancy_solid` and
  `free_eighths`.
- Form-dependent texture variants (visuals road, per ratification 2).

## DECIDED 2026-07-21 (user) — distribution-first expression: integrate the column, then slice it

The forms conversation reached the sub-voxel sieve (stubs.md § 12) from the
user's own framing: *"going fractional first and preserving simulation output
`H`, then generating chunks as distributions of the cell distribution, means
subvoxel boundaries become mixed blocks and we get more partials remainders on
the surface... why is nothing mixed?"*

**The diagnosis it produced.** Nothing is mixed because three gates prevent it,
and the first is the load-bearing one:

1. `deposit_deep_history` rounds **each unit independently** and drops it if it
   does not reach a whole voxel (`if tv < 1.0 { continue }`), with no remainder
   carried forward.
2. A voxel therefore belongs to exactly one event — contacts land on voxel
   boundaries by construction, so straddling is impossible.
3. `contents_for_event` fills all eight eighths from one member. The only
   heterogeneity in the world is *within* an event: placer ore substitution and
   igneous accessory pore fill.

**The reframe: this is a quantization-ORDER defect, not a resolution limit.**
The code computes `Σ round(tᵢ / 0.9)` where honesty requires
`round(Σ tᵢ / 0.9)`. The errors compound instead of cancelling. The tour's dune
field is the proof: 379 units summing to 7.99 m, averaging 0.021 m each, every
one rounding to zero — eight metres of recorded sediment expressing as nothing,
not because 0.9 m voxels cannot hold it but because the question was asked 379
times instead of once. This supersedes the heir filed in stubs.md § 12
("amalgamate adjacent sub-voxel units inside the record"), which patched the
symptom by pre-merging.

**DECIDED — the rule.** Metres survive to the voxel boundary; quantization
happens **once**, at contents construction. Each voxel's eighths are filled from
the recorded units overlapping its 0.9 m span.

**DECIDED (user) — presentation is non-blocking.** A mixed voxel loses the
internal order of what it mixes. The user: *"we don't have to rederive order in
a non-ordered voxel — yes it's lost order information in presentation but it's
far more honest than it was before and presentation can be reconsidered
later."* So the banded/layered presentation that materials.md § stratification
anticipates is **not a prerequisite**; contact voxels may render as the shipped
journal/0010 speckle for now. Revisit when the forms presentation work happens
(it shares a mechanism with the vegetation layer-vs-speckle mismatch in
ideas.md).

**DECIDED (user) — dithered eviction, not deterministic truncation.** When a
voxel span holds more material than eight eighths can carry, evicting the
thinnest deterministically deletes that material *everywhere*. The user:
*"to remain honest we could dither the material eviction across voxels... one
will evict a different partial than its neighbor so there's a fair
distribution."*

*Integrator generalization (PROPOSED — beyond the user's words, flagged as
such):* this is **stochastic rounding**, and it should govern the whole eighth
allocation rather than only the tie case. Each material takes its guaranteed
whole eighths; leftover eighths go to the materials whose fractional remainders
win against an **addressed** draw. A material with 0.3 eighths of true share
then appears as one eighth in ~30 % of voxels. Deterministic flooring is a
*biased* estimator that always loses mass; addressed stochastic rounding is
**unbiased** — expected composition over a neighbourhood equals the recorded
composition. Bias is traded for variance, which is the right trade for a
record: the ash band should exist *somewhere* rather than uniformly nowhere.

**Hard constraint.** The draw is addressed (seed + world position). Classic
error diffusion (Floyd–Steinberg) is sequential and order-dependent and is
therefore forbidden — the same rule that governs every other draw in the
generator. Precedent in-tree: `dithered_member`'s boundary dither.

**Expected consequences, to be measured not assumed:**
- The § 12 sieve loss should fall from ~75 % of the pile toward ~0; the drop
  threshold moves from 0.9 m to ~1/16 voxel (≈5.6 cm), a ~16× resolution gain.
- The **mixture table grows** — dithering deliberately makes neighbours differ.
  S8's cap bounds it (2 materials → 45 states; 6 → 3003), so it is expected to
  be affordable, but this is the slice's main risk and its main measurement.
- The clastic **veneer thickness budget should self-retire** for recorded
  columns: carry-`H` defined it as `round(H/0.9) − expressed`, and honest
  expression drives that difference toward zero without removing any code.
  Verify rather than surgically delete.
- Journal/0010's dither currently renders only placer fans, because nothing
  else is ever mixed. This lights up shipped, tested machinery world-wide.

**Sequencing — AMENDED 2026-07-21 (user): ONE slice, the surface folded in.**
The integrator proposed splitting buried strata from the surface voxel so the
appearance delta stayed attributable. The user overruled it and was right:
*"fold them together, we don't have to have this problematic of deciding which
material to skin the world with when the record already says. we already wanted
the veneer gone, if im not wrong that was part of this planned work from the
outset."*

They are correct about the history. The veneer is **two** mechanisms and only
one was ever bound for ecology:

- the **block rule** (`surface_sample`'s Grass/Dirt/Stone climate thresholds) —
  stubs.md § 2, whose named heir was the ecology system, but *specifically for
  vegetation*. Ratification 4 (grass suspended) already converted that half from
  "replace" to "delete", so removing the substrate half now is early, not new.
- the **thickness budget** in `clastic_pass` — rewired by carry-`H`, and
  self-retiring under distribution-first.

**So, added to the DECIDED rule:** where a record exists, the record decides
what the world is skinned with. The surface voxel expresses the column's top
remainder as partial fill and takes its block from `classify(contents)`. Grass
is not expressed at all; the resulting brown/rock-coloured world is
**pre-ratified verbatim** ("it'll be a mostly brown world for a bit").

**The constraint this must respect — one world answer.** `surface_sample` is
the kernel *shared* by `column` (the ground) and `coarse_surface` (the far-field
horizon), deliberately, so near and far are the same function at different
strides (ARCHITECTURE.md § One world-answer surface, RATIFIED 2026-07-19). The
record-derived block therefore belongs **inside `surface_sample`**, so both
paths inherit it structurally. Derived only in `column`, the ground would turn
sandstone-and-mudstone while the horizon stayed green and the LOD boundary would
become a visible lie. Height stays independent of climate and must not move
(`coarse_surface_matches_near_column_height` guards it).

**Fallbacks that remain legitimate** (to be stated explicitly in the journal,
each as genesis or absence-of-record rather than a surviving stub): the border
wilds (no deep run exists out there — stubs.md § Genesis), subaqueous columns
(`clastic_pass` returns early below sea level), and columns whose record rounds
to nothing (the 0.2 % bare-rock case from journal/0053), which should read as
their basement material rather than as painted Dirt.

**Bonus consequence.** The surface voxel carries *no contents at all* today
(`material_ids` skips `vy >= h`), which is one of the enumerated exceptions to
the fill-contract invariant in ARCHITECTURE.md. Skinning it from the record
**shrinks that exception** — and that list is supposed to only ever shrink.

## DECIDED 2026-07-22 (live session, late) — one namespace: block IS material

The user, examining the block_twin proposal, rejected the premise: *"they
should collapse. A material should contain all of the properties for all
forms of it… there are no edge cases where block != material in my mind."*

- **Block collapses into material + Air over time.** A voxel's one-name is
  "which material dominates this mixture" — `classify` keeps its job
  (mixtures still need their one name chosen) but answers with a material
  identity wearing its own face. The fifteen-name `block_twin` match and its
  `_ => Stone` arm die; no twin field is ever built (the interim
  registry-field plan is superseded — twins were the wardrobe for a
  distinction that shouldn't exist).
- **Cost accepted: art per material** (every material that can dominate a
  voxel gets a face) and a long-tail migration of the Block token's remaining
  consumers (storage palette, far-field span/pyramid, ~80 solidity checks
  already being drained by the fill contract's occupancy primitives, the
  mesher's layer pick, the player-facing name).
- **Categories stay, and become registrable** (direction): "a category list
  which can be registered to is defensible for materials, but the properties
  of the materials themselves is what matters, no proxy." Packs add classes,
  not only members. The class system's fixed-constant roster is scaffolding.

## DECIDED 2026-07-22 (same conversation) — transformation axes live on the
## material definition; both sims read the same rule

*"The material in the runtime game follows this rule, burned→charcoal, and
the same rule applies in the deeptime sim in these bulk calculations."*
(user). Transformations (burned, weathered, dissolved, compacted…) are
declared on material definitions — inheritable from a category default,
patchable per material, deletable — and are the ONE authority both the
runtime simulation and deep-time's bulk arithmetic consult. S-3 applied to
processes: a sim pass may aggregate and approximate a material's declared
transformation, never carry a parallel rule beside it. (The fires pass's
`soil × FIRE_CHAR_FRAC` is the standing counterexample: a bespoke rule over
a vegetation proxy the user explicitly does NOT avow. Its heir chain, in
order: ecology supplies the flammable inventory; the entry-species record
carries what actually burned; the burned-axis on organic materials supplies
the outcome. Charcoal and coal ride as-built until those heirs land — "I
don't care about coal and charcoal right now, they have heirs.")
