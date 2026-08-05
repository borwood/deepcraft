# Roadmap

The living sequence — the only non-append-only document besides code. Update
in the same commit as any journal entry: finished work moves to **Shipped**;
walk/field findings land in **Observed** first (the walk reports, the
diagnosis measures); only diagnosed work gets **Sequenced**.

## Shipped → [`ROADMAP-history.md`](ROADMAP-history.md)

**Archived 2026-07-26 by STATUS, not age.** Completed work and the superseded close blocks
moved to `ROADMAP-history.md`; each Shipped entry keeps its **journal number** as the stable
pointer. What stays here is what must remain *readable*: **In flight**, **Sequenced**,
**Observed**, and the **current close block**.

*Why: this board reached ~7,200 lines and stopped being read — it was grepped, and grep only
returns what you already suspected. `DeepField::chapters` sat unlisted through three spine
audits for exactly that reason.* **Archive an item when its status stops requiring it to be
read live, never when it gets old.**

## In flight

- **✅ S1 SHIPPED 2026-08-04 (journal/0157, merged): the correlation kernel.** The
  deposition clock made the rule *smaller* — the shared partition is the sorted union
  of the epochs the parents stamp, interval *k* IS epoch *k*, no bed is ever split.
  Pinch-out, P-4's onlap feather and seam-freeness all fall out with **no branch in
  the file**; mass is an identity. One partition table per chunk + a per-column
  iterator that allocates nothing (S0's I-5). 10 invariants, +0.00 s gate;
  cheap-evidence green (fmt 0 · clippy 0 · dc-worldgen 497/0). **BATCH DEBT: the full
  workspace trio is owed at the S2 boundary.** Nothing wired, no goldens moved.
  Inherited by S2: stubs #54 (mirrored weights, deadline S2's merge), the veneer's
  `unclocked_m` placement, and the M-C predicate below.
- **⚠ E4-2 IS NOT GREEN — DIAGNOSIS IN FLIGHT 2026-08-04.** Its agent never got the
  build slot, so the K2 kernel had never been compiled; the integrator compiled it:
  dc-core's 163 existing tests pass, but **both new structural fixtures FAIL** — the
  implicit step **flipped the grid-scale mode** (+20 → −18.3 at per-edge a = 5; holds
  at 0.5; the calibrated world runs at a ≈ 105), which is U-1's non-negotiable — plus
  fmt and 2 clippy errors. Hypothesis under test (integrator's, unproven): the
  limiter-as-projection guarantees mass and h ≥ 0 but **not no-overshoot**, so a full
  cell shipping 100 % of inventory to bare neighbours IS the flip — meaning accuracy
  sub-cycling is *required for monotonicity*, not optional for accuracy, which would
  collapse K2's cost advantage (break-even A_ACC ≈ 1.9). The diagnosis agent repairs
  fmt/clippy, bisects the true monotonicity bound, and measures whether sub-cycling
  restores it. **Do not merge that branch.** The **derived agreement bar** (U-4's
  mechanism, from the P2 ladder's own rows) is complete and survives regardless.

- **THE NORTH STAR — the engine shape everything converges to (RATIFIED
  2026-07-23; `docs/design/north-star.md`; now CLAUDE.md read-first item 0).**
  A **native** engine whose core is only cell storage + a pass-runner +
  field-solver **primitives** + a stable API surface — *every pass is content,
  including tectonics and erosion*, and **pass ORDER is authored per world**
  (`ARCHITECTURE.md`, DECIDED 2026-07-26). *(This line read "native
  field-solvers" until 2026-07-26; that framing was retired 2026-07-23 and the
  copy here outlived it — doc-topology finding #2.)* Materials (sheet + behavior slots +
  parent-shadowing hierarchy + slug→assets — the `Providers` **code shape** dropped
  to material level; *not* the world-level seam system, which is scaffolding and
  is meant to disappear — disambiguated 2026-07-28, `north-star.md` § Materials),
  their behavior, and the passes over them are **authored in a
  uniform, self-declaring, compiler-validated shape and tuned by data**. Behavior
  is code, tuning is data. **Plugin-first, closed-source-OK,** ~~untrusted-third-party
  safe** via a tiered backend behind ONE authoring shape: native `abi_stable` for
  trusted/first-party (incl. runtime), WASM sandbox for untrusted (gen-tier — the
  two-clock reconciliation; the Minecraft-space one-better).~~
  **🔴 THE TIERING IS RETIRED — struck 2026-07-29 (baseline sweep S5/F1).** `north-star.md`
  § Deviations **2** (user, emphatic): *"There is no trusted or untrusted until such a time as
  we need to"*; the capability tiering *"is **EXPLICITLY NOT THE MODEL** and must not shape any
  design."* `CLAUDE.md` read-first item 0 carries it: *"the trusted/untrusted split and the
  ABI/WASM backend tiering are **DEFERRED** … there is **no difference in permission between
  native and WASM**, so never justify a core/content placement by trust."* **ONE authoring
  shape survives; the two tiers do not.** *This bullet is the first thing a cold session reads
  after the read-first set, and it was handing over a retired trust model as the north star's
  live shape — the paraphrase outran its source and was never re-struck when Deviation 2
  landed, two lines below where the same entry already marks the "native field-solvers"
  correction.* The crossing
  constraint (plain-data + handles, no rich Rust across the seam) taxes the SDK
  *surface we design*, never the content author's expressiveness. Pursued
  **evolutionarily** — the seam-first march IS the path (~70% embryonic in-tree:
  pass graph, Providers, MaterialProps, transformation-axes, categories,
  build_checked, octree, two-clock). **Compliance loop live: all design flows
  through it, double-checked at plan/work/review, carve-outs through the user
  (strategic companion to spines).** Narrative: journal/0081. **De-risk before
  committing the arc (sequenced, not urgent):** (1) hierarchical material behavior
  resolution prototyped on wood/charcoal `combust→`; (2) one existing pass (fires)
  reformulated onto the Pass interface, byte-identical; (3) the ABI/WASM boundary
  research spike (`abi_stable` vs `repr(C)` vs `wasmtime`) — locks the SDK shape.

- **THE MATERIAL-BEHAVIOR SUBSTRATE — design pass RATIFIED 2026-07-23
  (`docs/design/material-behavior.md`, journal/0086).** The spec for the
  north-star content layer and the Crux 1 build. The deep-cell inventory IS the
  fill contract one tier up (a stack of spans indexed by depth; strata record =
  temporal authority, mutable working-inventory = current, chapter-commit = the
  compiler step). **Forms** are a closed machine set (structure/loose/pore-fill/
  fluid; void = the complement) with fractional occupancy orthogonal; **edges**
  (all form→form moves) are machine-complete and ungated — they ARE the process
  catalog; **agents** are content rate-terms folded on an edge (S16); **passes**
  come in two shapes — **cellular** (run edges, local) and **field** (compute a
  field, plant it) — with the reciprocal loop (S-4) carried by the chapter loop
  and dual processes split into field+cellular halves. Ownership: forms+edges =
  machine, materials+agents+passes = content; forms are NOT SDK-registrable.
  **Progress 2026-07-23:** (A1) present-tier collapse **MERGED** (`de984eb`,
  journal/0087, gate-verified on merged main) — 1-byte atom (64→32 KiB/chunk;
  intermediate `{Air,Material,+4 legacy S1}` on the ratified path, 2-variant at
  Crux 2), `block_twin`+`_=>Stone` deleted, `classify→dominant material identity`,
  and a latent S-3 violation the collapse surfaced was fixed. **User ratified the
  appearance change (within-class member variety now visible at block/far tier)
  on the identity argument, sight-unseen.** Tail: solidity→occupancy drain (~80
  sites, deferred), far-span `Block` token migration, legacy S1 retire (Crux 2),
  categories-registrable. (A2) deep-cell working-inventory spike **COMPLETE**
  (S17, `docs/spikes/S17-deep-cell-inventory-results.md`, branch unmerged):
  byte-identical under identity default, **per-stratum** granularity, and the
  inventory is derivable → a **per-chapter transient** (no permanent memory cost).
  **Open, blocking merge:** the record is keyed by depositional *environment* not
  material, so material-changing behaviors must **commit as appended facts (S-9)**,
  not tag rewrites — a design call that shapes the keystone. commit-semantics
  **RATIFIED 2026-07-24** (append facts; fact carries `cause`; apply-time edge
  logging). **Progress 2026-07-24 — keystone + instrument MERGED, gate-verified:**
  the **keystone** (`37b0398`, journal/0088) — the fact-ledger, `commit_chapter`
  diff-and-append, provenance read, byte-identical under the identity default;
  single-edge shape, apply-time logging + `cause` land with the first behaviors.
  The **contents inspector** (`f594580`) — `world_get_contents` + F3 HUD (see the
  Shipped dev slice). The inspector surfaced that the runtime stores only `Block`;
  contents re-derive from gen (edit-blind, S-2), so **break-gives-mixture needs the
  runtime edit-fact overlay** — the north-star's first runtime-process milestone,
  designed (commit-as-facts) but unbuilt.

- **The block↔material collapse — DECIDED, one namespace (user, 2026-07-22
  late session; materials.md two new DECIDED entries)**. Block collapses
  into material + Air; no twin field is ever built (the block_twin slice as
  drafted is SUPERSEDED); classify answers with the dominant material
  wearing its own face; categories become pack-registrable (direction);
  transformation axes live on material definitions and BOTH sims read the
  same rule (S-3 applied to processes — the fires pass's soil-scalar proxy
  is explicitly un-avowed, heir chain: ecology → entry-species record →
  burned-axis on organics; coal/charcoal ride as-built meanwhile). First
  slice when dispatched: kill the fifteen-name match + `_ => Stone` arm in
  classify (dominant material's own identity), byte-identity where faces
  exist today; the Block-token consumer migration (storage palette,
  far-field span, solidity checks, mesher layer, player name) is the
  long-tail arc.
  **NOW FRAMED AS STEP 1 OF THE NORTH STAR** (2026-07-23): "materials are the
  universal substance the API hangs off." **Cruxes RATIFIED (user, recon:
  `docs/audits/` block-consumer inventory, journal/0081):** (a) ~~**render-first
  wedge** — delete `meshing.rs::block_layer`'s geology re-translation, route
  `classify → material → atlas`.~~ **INVESTIGATED 2026-07-23, journal/0082: the
  premise was falsified — no deletion.** The near-field mesher's contents-bearing
  path already routes `contents → material → material_layer` directly (via
  `top_splat`; it never calls `block_layer`). `block_layer`'s geology arms are
  **not** dead: they are the block-only (contents-absent) summary, exercised by
  the far-field pyramid `push_quad` (production render), the benches, and the
  absent-contents geology fallback — the *same* mechanism as the four legacy arms
  the crux keeps, so they cannot be deleted in isolation without breaking the far
  field (out of scope) or making `block_layer` non-total. Crux 1 (below) subsumes
  them. Landed instead: a guard test
  (`block_only_geology_layer_agrees_with_direct_material_layer`) pinning
  `material_layer(m) == block_layer(block_twin(m))` for the seven primary geology
  blocks against silent drift (A-7). The only residual *real* Block→layer
  round-trip for geology is the far-field top face (`farmesh.rs::push_quad`) — a
  separate slice, feasible via the far pyramid's material store. (b) **Crux 1 —
  air is a peer:** redefine `Block` in
  place to `enum { Air, Material(MaterialId) }`, keep the token for is_solid/==Air,
  delete `block_twin`; niche-optimise `MaterialId` for a **1-byte atom** (64→32
  KB/chunk, a storage/perf win — verify). (c) **Crux 2 — retire, do not enshrine:**
  the four legacy S1 blocks (grass/dirt/stone/wood) and the `TerrainGen` path are
  **retired**, not migrated to materials ("it already exists" is not a reason;
  carve-outs kept for sentiment are the stumbling blocks the shape can't afford) —
  verify no live fallback first, re-point it at the real worldgen in the same
  slice. Order: render-first · redefine Block · drain solidity→occupancy · storage
  atom · retire legacy · then categories-registrable. Sibling forks
  (`block_uses_contents` trust gate; the render-edit-writes-materials question)
  parked downstream of the atom. Journal 0081 written (design pass).
  **COUPLED (user, 2026-07-23, from the weathering-spike shaping):** the
  **deep-cell material-inventory** question is the same question as Crux 1's
  storage atom, one tier up. Deeptime stores bedrock/regolith as *heights* (R, H)
  + a strata record, NOT a material multiset — so a behavior that "consumes
  bedrock, produces regolith" has no inventory to operate on. The weathering spike
  works around it with a thin ctx-adapter over heights (deliberately), but the real
  fix — a material inventory for the deep cell — **needs solving soon** and rides
  with the material↔block collapse. Filed so it is not the spike's silent debt.

- **The ratified sequence after the migration (user, 2026-07-22: "both
  revisions greenlit")**: **1. The perf window** — one dc-client cluster:
  ~~profiling slice (Tracy/tracing spans + vertical-drop baseline → ranked
  killer list)~~ **LANDED 2026-07-23, journal/0080 (`perf` feature + self-time
  aggregating layer + `--perf-drop` capture; baseline artifact stubbed pending an
  integrator GPU run)** · ~~**async-offload slice is now the sequenced NEXT** (chunk
  gen + far-mesh onto AsyncComputeTaskPool; the synchronous `stream_chunks`
  gen+mesh loop is the span-named suspect — `chunk.gen`/`chunk.contents`/
  `neighbor_fill.gen`/`mesh_chunk` on the frame thread)~~ **LANDED 2026-07-23,
  journal/0083 + journal/0084 — see `ROADMAP-history.md` § Shipped** (status drift only, corrected by the
  2026-07-25 staleness sweep row S-9). *The live successors are the two Observed items
  it left behind: the **throughput ceiling** and **per-thread attribution**.* · ~~erosion-budget dev flag
  (the walkable cranked world → the standing amplitude call)~~ **LANDED
  2026-07-22, journal/0076 (`--erosion-budget <mult>`)** · albedo-at-
  range once baseline numbers exist, carrying the ranges-as-player-config
  rider. This PROMOTES perf over the queued seam batch per the runtime-is-
  sacred convention; **the seam-conversion batch (materials/form,
  paleoclimate, doc riders) runs in PARALLEL** — headless, disjoint
  write-sets. **2. The dressing arc** — the user's filed anticipation
  ("back to block primitives + the artful procedural dressing") named as
  next-after: material heightmap SHAPES for legibility, form-dependent
  textures, turf presentation, block primitives; the settings-plumbing
  slice (game config menu for render knobs) opens with it. FF2b
  persistence + dirty-rail slots into the first free gap; remaining
  CoarseField migration follows the type freeze.

- **The threshold-quantization migration — GREENLIT, "we finish this today"**
  (user, 2026-07-22; audit: docs/audits/2026-07-22-threshold-quantization-
  audit.md; end-state: spines § S-4 "Ratified end-state"). Route: **A1
  dispatched** (share-weighted susceptibility blend over the dominance
  window — dissolves the S-4 contour by construction; journal/0072) → FF2b
  integration → **B1 LANDED** (surface_class plurality → membership dither, the
  move-B witness; journal/0073 — resolves the 10 km checkerboard Observed item;
  **NEEDS RATIFICATION**: the draw is the *coherent* bilinear field, not the
  audit's white noise — white noise doubled the far-tile mesh because the far
  field point-samples it, and the unbiased end-state is deferred to the type's
  share-summary read) → **CoarseField<T> — EXTRACTED 2026-07-22 (journal/0075).**
  Built in dc-core (headless): `CoarseField<T>` + `Interpolable` (move A, anchored
  blend exact at identities) + `ShareVec` (the quantity both moves read) +
  `sample_dithered` (cake-law boundary membership dither) + `summarize` (coarse
  read) + `DitherSource` (the source axis), with generic law-tests and a
  `compile_fail` doc-test proving the raw per-cell read is inexpressible. A1
  migrated behind it **byte-identically** (goldens unmoved); the pinned pair
  `outcrop_at`+`outcrop_shares` **consolidated to one slot** (verdict derived =
  `argmax ∘ outcrop_shares`). Boundary-dither + summarize falsifiers pass;
  cake-law adoption in collapse.rs/far.rs is follow-on for those owners, and the
  node-payload sampling vocabulary adopts the type in a follow-on
  (octree-substrate.md § 3). **NEEDS RATIFICATION (a finding):** the coherent
  source's aggregate bias is *majority-amplifying (away from 50/50)*, the opposite
  sign of carve-out 1's "toward-50/50" characterisation — it sharpens minority
  phases rather than flattening them (see spines § 4 amendment; Observed).
  Audit shortlist: A1 EXTRACTED · B1 landed · A2 (rides A1's table) / A3 / A4 / B3
  / octree node = held for their owners. User's framing on the record: "there are
  things i'd rather do but this is foundational."


- **The forms/partials fill contract — DESIGN PASS RATIFIED, FOUNDATION
  DISPATCHED** (2026-07-21, live session; ARCHITECTURE.md § "The fill
  contract", materials.md § "The forms design pass"). The user ratified
  **contents-as-authority: `classify(contents) -> Block`, invariant
  `block == classify(contents)`**, after a consumer audit (135 block-consuming
  sites across 31 files, of which **80 across 27 files are solidity-shaped
  checks** — `!= Block::Air` / `== Block::Air` / `is_solid`) showed four future
  systems (water fill, loose gravity, compaction, sim light) all want one
  per-voxel occupancy primitive rather than each re-deriving occupancy from
  that bool. Also ratified in the same conversation: fractions come
  **only from the ledger** (recorded quantity/variance, never cosmetic noise —
  this scopes the defect-jitter sketch); **sand is a FORM of the existing
  clastic-coarse material**, not a new identity (accepted cost: loose and
  structural clastic are visually indistinguishable until a later
  form-dependent-texture visuals decision); the **root-lattice soil model**
  (roots as porous STRUCTURE holding soil in its pores — an anti-erosion
  mechanism for free, since matrix-less loose soil falls); and **grass is
  suspended, not expressed** (an ecology state riding on substrate materials;
  the user pre-ratified the appearance: "it'll be a mostly brown world for a
  bit" — turf's ligature/anisotropic-variant presentation sketch is captured
  but explicitly NOT to be built). Background agent dispatched for the
  foundation: dc-core `classify` + occupancy primitives, contents-first
  rewire of `collapse.rs`, the byte-identity + invariant regression proof,
  then (if clean) the first ledger-justified fractional top on the eolian
  sand blanket. Client wiring (lighting up journal/0010's dormant
  partial-height renderer, moving collision onto the occupancy threshold) is
  a deliberate follow-on slice, not this one.
  - **FOUNDATION LANDED 2026-07-21 (journal/0052).** `dc_core::classify`
    ships with `block_twin` / `dominant_material`, plus the occupancy
    primitives (`free_eighths`, `loose_eighths`, `bound_eighths`,
    `open_pores`, `is_occupancy_solid`, `SOLID_EIGHTHS = 4`) whose doc block
    names their four intended consumers so none re-derives occupancy
    privately. `collapse.rs` is rewired contents-first. **Byte-identical:
    proven by goldens captured against the pre-rewire generator — and
    INDEPENDENTLY RE-VERIFIED by the integrator, who ran the agent's own
    fingerprint against pre-merge main and reproduced all three triples
    exactly, so the goldens are not circular.** The invariant holds over
    3.64 M contents-bearing voxels. Note the pre-existing
    `geology_world_regenerates_byte_identically` proves *determinism*, not
    invariance across a change — the goldens carry that claim alone, which is
    why the independent check mattered. Also landed: the journal/0050
    collapse-cache `evict()` gap (now reachable from `coarse_surface`,
    `column_record`, `surface_elev_m`, `lattice_point`, `surface_chunk_y`).
  - **Design decision worth knowing**: `classify` is keyed on the **material**,
    not the geology class — contents carry only `MaterialId`s by design, and
    passing a `GeologySet` in would make the block tier a function of the
    registry and destroy the purity the invariant rests on. Byte-neutral for
    every set in the repo; the one divergence from the retired class table is
    a pack binding a material into a class whose block disagrees with the
    material's own nature (e.g. `PEAT` registered as clastic-fine), where the
    material is now authoritative. Flagged for the user, not blocking.
  - **NEXT SLICE — the fractional top. ~~BLOCKED on a user-owned decision~~ —
    UNBLOCKED, and this line was stale for eight days** (found by the 2026-07-29
    classification pass): the blocking decision — the veneer retirement — is recorded
    **DONE 2026-07-21 (journal/0055)** ~150 lines below in this same section. The slice
    is startable and unowned.** The eolian remainder is real ledger (2.09 m / 0.9 m → 2 voxels
    + ~2.6 eighths discarded at `.round()`), but a partial voxel with solid
    material above it is a void underground: the only honest home for a
    fraction is the top of the column, and that voxel is the **surface-veneer
    stub**, whose retirement ratification 4 explicitly reserves for its own
    slice. Doctrine ("fractions come only from the ledger") forbids
    manufacturing a remainder for the veneer, whose thickness is a
    whole-voxel analytic budget. So the sequence is: **retire the veneer for
    recorded columns first** (user in the room), then `StrataEvent.top_eighths`
    emits the real remainder as `debris_only` — a quantity change on one
    voxel, since clastic strata are *already* emitted loose. The golden
    fingerprints move in that slice, and that move is its deliverable.

**Slated by ratification 2026-07-20 (all six unknown-unknowns landed;
sequenced, not yet scheduled):** marker beds + punctuation event types ride
the volcanism design (earth-processes § 2 flag); ore-genesis roster is a
user-owned content design pass (earth-processes § "payoff layer") to be run
as main-session design conversation; impact structures ride punctuation
hooks.


**Decisions waiting on the user (nothing else is blocked on them):**
0. **THE SOIL GAP — the biggest design hole the walk exposed.** The user, in
   world: *"the world does not generate dirt anywhere apparently — was only a
   veneer placement previously."* Verified: `MaterialId::LOAM` is registered in
   **no** geology class member, and the `organic-soil` class's only member is
   carbonaceous **mudstone**, a rock. **The world has no soil substance at
   all**, which is why 91.4 % of land skins to one block — both candidate
   explanations were downstream of that. It compounds with form: organics sit
   in the *structure* bucket, so soil would render as solid cubes even once it
   exists. **Fix order: substance first, then form-from-provenance**
   (consume-the-ledger piece (c)). This is the missing substrate under the
   already-ratified root-lattice soil model (materials.md § forms design pass)
   — that model currently has no material to hold in its pores. Content-set
   work, so the member roster is a **user call**.
1. *(**The surface-veneer retirement: DONE** 2026-07-21 — the user made the
   call live ("fold them together... the record already says") and it shipped
   in journal/0055. What replaces it as the live user question is the
   **appearance walk** above.)*
3. **Roughness recalibration** — three costed candidates, picked from
   pictures; C is the only one that changes which landforms exist. Sequence
   the `--horizon 6` landform walk first so the pick is made against real
   terrain.
4. **`classify` is material-keyed, not class-keyed** (journal/0052) — a
   judgment call the integrator accepted; byte-neutral today, diverges only
   for a pack that binds a material into a contradicting class.

**Design passes queued (main-session conversations, not dispatches):**
ecology (starts from biomes-ecology-agenda.md) · social sim (the big one) ·
the water/caves thread (water.md § Session capture 2026-07-21 — the
two-drainage-opinions finding and the bounded-drainage-refinement spike
question are the live items).

**Engineering still Sequenced:** *(**sub-voxel facies / the sieve: SHIPPED**
2026-07-21, journal/0055 — stubs § 12 retired)* · tectonic expression
(dip/fold + metamorphic — *the deep-time half now has a socket waiting for it:
`Providers::outcrop_at`, journal/0060*) · paleo-context provider (subsumes stubs
4/5/6) · **the littoral heir: fetch × wind into `Providers::wave_energy`**
(journal/0060; needs the S11 body graph's open-water field, so it will likely
arrive as a materialized plane and convert that slot from value-level to
pass-level) · **the waterlogging heir: the S11 saturation field into
`Providers::depth_to_water`** (journal/0061 — the slot and its `WaterPass`
contract exist; what remains is the field, and the units decision, since the
four consumers threshold a 0..1 index and a water table speaks metres) ·
`HostWorld` edited-chunk spill to the save layer (edited chunks are retained
unboundedly by design; ~640 MB per 10 000 edited chunks, asserted by test) ·
**the coarse-capacity void axis** (S15 design choice 3, journal/0062 — capacity
below a cell's floor plane comes only from edits today, which is complete for a
heightfield world and wrong the moment caves exist; the collapse tier's
"recorded conduit capacity → void intervals per column" is the same axis, so
~~decide it once in water.md rather than twice~~ **— RESHAPED 2026-07-25 (sweep row
R-2): its home moved out of `water.md`.** `flow.md` § 7 promotes the **interval-log
fill contract** from *proposed* to **necessary** (voids/conduits are intervals, not a
heightfield), and voids / conduits / springs are **FLOW continuation (c)**. Decide it
there, once, with (c)) · **re-run S15's natural-sill
falsifier when the cave families land** (not constructible today only because
the world has no 3-D structure — corrections #31; **the cave families now have a named
owner — FLOW continuation (c)**, which as of 2026-07-25 carries **three** obligations,
not two: the free/bound edge, void intervals, **and the conduit pairing rule** — a karst
conduit pairs by **void connectivity**, a third mode flow.md § 11.5's two-mode green
explicitly does not cover) · **when the store can fail
to answer (async streaming / disk-backed regions), an absent chunk must read
UNKNOWN, never solid**, or eviction manufactures false component boundaries in
the connectivity index (corrections #31; **there is now an in-tree precedent to copy
rather than re-invent — `Identity::Unrecorded`**, journal/0101: a "no record here"
answer that is distinct in the **TYPE** from an empty value, not a sentinel inside the
value. Copy that shape; a second, differently-spelled unknown would be an A-4) ·
reconcile S15's cell-granularity body
footprint with S11's air-component container (S15 design choice 2).


## Sequenced

### PER-CYCLE QUANTIZATION — the 12 fps call, taken (bodies thread; **DECIDED 2026-08-04, user**)

**WHAT.** Retire `ANIM_FPS = 12.0` as a fixed constant (`dc-client/src/body.rs:70`). Quantization
becomes **per cycle** (`N = round(cycle_duration × target_fps)`, clamped **N ≥ 2**), **per bone**
(weighted by the proportion of anims owning it), and **client-owned** (target fps is a client
performance/visual setting; the pack may declare a per-body **max**). A one-off is just a cycle
that does not repeat. **Authority: `bodies.md` § Stepped animation, the 2026-08-04 banner** — it
carries the full ruling, the rejected alternative and the foreclosed work; this entry does not
restate them.

**WHY.** A fixed rate **aliases a derived cadence**. Frames-per-cycle stopped being a number
anyone chose the moment gait was derived per body: biped **6.970**, longleg **7.729**, **stout 4.291** —
*(these read 6.977 / 7.742 / 4.286 until 2026-08-04: three-decimal figures carried from conversation,
off in the third decimal. **Measured at the build**, and journal/0148's own two-decimal table
— 6.97 / 7.73 / 4.29 — was right all along. **No N moves**; the ruling is untouched.)*
low *and* fractional, so its samples drift through the cycle and beat against it. The user saw
the hitch at the 0148 walk and diagnosed it unprompted. **A-1's fourth instance in this arc**
(after the four absolute-metre constants, the world-global walk speed, and the hardcoded gravity).
*The fix is that N becomes an **integer**, not that it becomes larger* — 4.291 → **4**, 6.970 → **7**,
7.729 → **8**. Nothing gets smoother; it stops beating.

**UNIFIES.** Third instance of *the sim owns the target, the client owns the approach* — framerate
is approach. First use of `dependency-graph.md` § 0b (**opinion vs absence**) to place a body knob:
the *rule* is an engine kernel, the *rate* is a client setting, and the per-body **cap** is a pack
opinion because two well-made packs would answer *"is this body mechanical"* differently.

**FIRST SLICE.** Client-side only, no firewall move, no sim state: derive `N` per cycle from the
target; clamp at 2; blend `Δt = duration / N` per bone by ownership; delete the fixed constant.
Acceptance is **the stout's hitch gone with the biped visually unchanged** (6.970 → 7 is inside a
rounding), and a test that `N` is integer and ≥ 2 for every plan across a cadence sweep. **Its one
open call: an explicit guard that quantization never feeds back into anything sim-visible** — if
B5's damage resolution reads a pose it reads the *unquantized* target, or two clients at different
fps resolve hits differently.

**CONTINUATION SLOT — this is a slice OF the derived-locomotion arc (B2 member 1 → B3).** After it:
the **per-body cap** as pack data (wants a body plan field, so it rides the B3 wire window with
`stubs.md` #34's binding key), and **B3 itself**, which brings the 20 Hz sim-tick vs client-step
question this ruling has now half-answered — the client's step is derived and per-bone, so B3's
question narrows to *what rate the sim publishes targets at*, which is a different question than
the one the board has been carrying. **Appearance-gated: the user rules the look from motion, not
from a still (corrections #77).**

### FOUR LIVE ARCS PROMOTED FROM CLOSE BLOCKS — **stamped 2026-08-03 (staleness F1: three
user decisions/greenlights and one superseding build had NO live-board entry — they lived
only in `dependency-graph.md` rows and the two close blocks, and a close block is a handoff
that gets archived).** Compact entries by design: ROADMAP wins on content, the graph on
sequence — each names its authority docs rather than restating them.

- **THE STRATIGRAPHIC-CORRELATION BUILD (geo thread; supersedes P11 slice 3's membership
  dither as plan).** User sketch + smoothness principle, 2026-08-03; design pass DONE —
  `docs/audits/2026-08-03-stratigraphic-correlation-design.md` (R-C shared-clock rule;
  mass argument holds; read-side, so **every deep-time golden must stay bit-still**).
  **Blocked on its § 9 picks (user-owned, being taken one at a time): ✅ P-1 RULED
  2026-08-04 — M-C with the DERIVED predicate, via the materials-roster philosophy
  (`materials.md` § DECIDED 2026-08-04) · ✅ P-2 RULED 2026-08-04 ("yes") — the
  inverted acceptance criterion + walk plan, two-register reading, sharpness
  guilty-until-process-claims-it (audit header banners carry both) — **P-3 PAUSED
  2026-08-04 by the user's architecture question ("they were laid down on the same
  clock. did we throw the time away?" — answer: yes, only the 8-chapter clock
  survived packing): the DEPOSITION-CLOCK design pass is in flight
  (`docs/audits/2026-08-04-deposition-clock-design.md` when it lands) — if a
  per-bed epoch prices sane, correlation matches by true isochrons and P-3
  largely dissolves; it also pays the explainability WHEN axis and knowledge.md
  req 3** · **✅ P-4 RULED 2026-08-04 ("an empty stack is a parent whose every chap
  thickness is 0") — B-1 onlap feather; hard edge survives at the true grid border
  only** · **✅ P-5 RULED 2026-08-04 ("yes, ratified") — the supersession is FORMAL;
  the § 5 retirement list executes in S2. Direction rider (user, at ratification):
  "re-implement the variety in our terrain as the result of purposeful standardized
  operators… most refinement was old native code, not on the plugin shape."** All
  picks closed or paused ~~(P-3 ⏸ on the clock pass)~~ — **and the clock pass landed
  same day: ✅ O-2b RULED 2026-08-04 (u24 thickness + u8 raw epoch in the second
  word, zero widening, unit count identical by structure; premise corrected at
  ratification — 16,384 m is the per-UNIT thickness cap, not layers/cell; user's
  bit-donation expectation recorded) → P-3 RESOLVED BY RESHAPING: correlation
  matches true epoch intervals, R-C demoted to within-interval interpolation
  ~~unconformity flag = measurable gap semantics~~ (corrections #99: gap DURATION
  is structurally unrecoverable — the flag fires only when a strip empties the
  record; the surface is real, its length is not)** (`2026-08-04-deposition-clock-design.md`,
  rulings in header). **ALL PICKS CLOSED.** Build sequence: ~~**the CLOCK SLICE first**~~ **✅ THE CLOCK
  SLICE SHIPPED 2026-08-04 (journal/0154; merged same day):** O-2b layout (u24+u8,
  zero widening, unit count bit-identical with the old-shape identity proof),
  unconditional `set_epoch`, monotonicity asserts (epoch AND chapter — the design's
  owed assert discharged), funnel widened (`StrataEvent` epoch span, carry-never-read
  pinned), exactly the 6 GOLDEN_RECORD* families re-captured once, all others
  byte-still; full trio **fmt 0 · clippy 0 · 1006/0 · 94 suites**. New stand-in:
  stubs #52 (epoch immutable under overprint; alteration-time axis is the heir).
  **✅ S1 SHIPPED 2026-08-04** (journal/0157 — the epoch-keyed join; see § In flight for
  what it inherits to S2). ~~**NEXT: S0**~~ **✅ S0 DONE 2026-08-04** (`2026-08-04-s0-correlation-measurements.md`,
  merged): I-5 = **(ii) lazy per-column evaluation** (naive = +104.8 % of `column()`;
  (iii) not needed); 2×2 epoch-partition mean 44.9 / p95 145 (clock 4.2× finer than
  chapters); mixture cap passes 14 vs 3,002; unit count settled **7,304,581** (the
  Observed entry resolved); `contents_contract` re-baselined 137.31 s; **two
  refutations stamped** — the gap-duration claim (corrections #99) and F4's 8.6×
  cost arithmetic (banner on the design). ~~**NEXT: S1**~~ **NEXT: S2** — wiring into `column()`, the
  ≤14-file `record_for` repayment, P-5's retirements (`cell_of`, the near
  `sample_source_cell` site, the `NearRecordMembership` domain), B-1 at the record
  extent, fold-per-parent, the veneer's placement, **stubs #54's deadline**, CONTENTS
  + SURFACE re-capture with deep-time stillness asserted in the same merge, and the
  **full workspace trio clearing S1's batch debt** (the arc-chunk boundary) → S3
  (acceptance walk, Claude drives).** Was: a sub-bullet of the § Observed field
  report only.
- **E4-2 + E4-3 — the implicit field kernel and its adoption (geo thread; greenlit
  "e4 yes. queue right away", 2026-08-03).** E4-1 shipped byte-identically
  (journal/0150, `dc-core::field::FieldKernel`, stubs #30 discharged). **Live
  ⚠ NEEDS RATIFICATION: the `dc-core` venue + its rayon rider** (audit U-3;
  `docs/audits/2026-08-03-e4-implicit-kernel-design.md` § 8 U-1…U-5). E4-2 =
  `Scheme::ImplicitBE`, non-default, convergence study; **E4-3 flips adoption WITH the
  P2 re-pick so goldens move once** (D3(M) ladder re-runs under the new integrator
  first). Graph row E4.
- **B7 — joint rotation limits (bodies thread; DECIDED 2026-08-02, user).** Design pass
  done (`docs/audits/2026-08-02-joint-limits-b7-design.md`, all seven calls closed);
  its five stubs land WITH the build, deliberately not before. Wants the pre-B3
  segment-identity window. Graph row B7; bodies close block item 3.
- **B8 — individual proportion variation (bodies thread; DECIDED 2026-08-03, user).**
  SIZE FIRST via allometric axes; range = engine, distribution = pack, pack owns
  fairness; does NOT wait on B4; size is free today (the bake returns angles + a
  height ratio). `bodies.md` § Individual proportion variation; graph row B8. Unbuilt,
  no design pass yet.

### PARENT MATERIALS — inheritance with tri-state shadowing — **filed 2026-08-04 (the roster-skill fold; ratified design, user-originated, ZERO code)**

**What it is:** parent pointer + per-parameter shadowing at the material tier —
ratified at `north-star.md:205-209` (origin journal/0081), the shadowing semantics
user-originated and PROPOSED at `material-genesis-notebook.md` §§ 2.3/2.5b. Parameters
are tri-state (*inherit* · *override* · *disable*) so "unspecified" and "specified as
nothing" are distinguishable; **no engine root, no engine defaults** — resolution
terminating undeclared yields **absence, never a substituted value**. Parenthood is a
RELATION, not a kind (user leaning 2026-08-04, tentative), and **parentage implies
nothing about material relations** (ratified same day — `.claude/skills/roster/SKILL.md`
§ 2b carries the constraints).

**Why filed now:** it was ratified with zero code and lived only in the notebook and
north-star — the close-block-only shape staleness F1 spent a day repairing. The bodies
session's findings doc (`2026-08-04-roster-skill-bio-and-parent-findings.md`) asked its
sequencing; the user ratified the disposition.

**Sequencing:** design pass **after the correlation arc settles** (that arc owns the
golden windows). First build per the north-star de-risk item: **hierarchical behavior
resolution prototyped on wood/charcoal `combust→`**. The **density/porosity sheet
reconciliation rides the same design pass** (solid density + porosity as axes, bulk
derived — the sheet's two-convention defect, roster skill § 2 step 1 caveat). B6-c
(bodies) is a second consumer, **not the justification** — the argument stands without
bodies.

**⚠ A THIRD CONSUMER APPEARED 2026-08-04, AND IT IS A LIVE BLOCKER — S1 measured it
(journal/0157):** P-1's ruled **M-C continuum predicate is not buildable today**, and
the reason is exactly this entry's subject. The ruling needs *regions adjoining on a
declared axis*; **no material declares a region** — `MaterialProps` is point-valued,
and FS-A's release spectra describe what a rock **sheds**, not where it **sits**
(granite sheds gravel→clay; its support is the whole ladder). Any predicate buildable
now would need a threshold on a scalar distance — the fitted taxonomy `materials.md`
§ DECIDED 2026-08-04 forbids by name. So the correlation build ships M-C's *hook* and
the predicate waits on the term-schema half of this work. **Constructive pointer from
the measurement:** `GrainGrade`'s Wentworth partition already declares real interval
bounds — that is the shape the property sheet needs, per material, per axis.

### THE BUILD SLOT MUST COVER PROBE **RUNS**, NOT ONLY BUILDS — **sequenced 2026-08-04 (user-greenlit fingerprint; NOT done at wrap on purpose)**

**What it costs today, measured in one session:** an agent burned an entire slice in the
queue and shipped zero measurements; another wedged twice; the wrap gate failed **three
times** on three different example binaries. Cause in one line: **a running probe binary is
neither `cargo` nor `rustc`**, so the mutex hook does not see it — and because probe
examples carry `test = true` (the probe doctrine, correctly), every one of them is a gate
build target that **cannot be relinked while it runs** (`LNK1104`). The failure wears a
compiler error's clothes rather than announcing contention.

**Why it was NOT folded at the wrap:** `scripts/cargo_mutex_hook.py` governs *every*
session and a parallel session was live; a wrong denial predicate blocks everyone. This
wants its own slice on a quiet machine, with the hook's own tests.

**Candidate shapes, none picked:** the liveness check also scans for processes whose image
sits under `target/release/examples` · probe RUNS take the lock the way builds do
(`cargo run --example` is already a cargo invocation the hook intercepts — the gap is that
the lock releases when cargo exits, while the spawned exe keeps running) · or a separate
target dir for probe runs. **Interim rule, live now (§ Observed):** treat `LNK1104` on an
example as machine contention, never a red — check `Get-Process` for the named probe first.

### ROADMAP § OBSERVED WANTS ITS OWN FILE — **sequenced 2026-08-04 (user-greenlit fingerprint)**

The file is **2.1× its REGISTRY threshold and archiving cannot fix it**: by the archive
pass's own (correct) rules, entries stamped this session stay live for a session of user
review, so the board grows faster than status-archiving drains it. § Observed is the bulk
and it is *entry-addressable* — read by ordinal, never whole — which is exactly the
registry shape the convention says to split. **Proposed:** `ROADMAP-observed.md` beside
`ROADMAP-history.md`, live entries only, with the archive pass draining resolved ones from
*there* into history. Not done at wrap because a split that moves entry numbering while two
sessions share the checkout is how references rot; it wants a quiet checkout and one pass
that fixes inbound references in the same commit.

### THE STAND-IN MARKER CONTROL — **owed 2026-08-03 (corrections #97, greenlit fingerprint); survey first, sweep second**

**The gap it closes.** CLAUDE.md read-first item 6 wants a deliberate loose end annotated **in
code AND** listed in a locus (`stubs.md` · `spines.md` § 3 · ROADMAP Owed). **Nothing checks the
second half.** `spine-audit` reads `spines.md` against code; the staleness sweep cannot see a
thread that was never on the board; `doc-topology` compares docs to each other. The inverse of
§ 3's *"built and nothing calls it"*, and equally structural.

**What it cost:** four `GaitKnobs` stand-ins marked `STAND-IN, heir B6` in source with bands and
a citation, reaching **no locus**, surviving the integrator's own full workspace gate, a merge to
main and a second slice — found by the **user** asking an unrelated question.

**Step 1 is a SURVEY, not a sweep, and this is the load-bearing part.** Scope is genuinely
unknown: the explicit `STAND-IN` convention is **three files**, but the word **`heir` appears at
200+ in-code sites across ~40 files** and only ~12 cite a `stubs.md` number. **Grep cannot
separate a real stand-in from ordinary design prose** — so the first deliverable is *how many of
those 200 are real*, not a fix. Answer that before designing the check.

**Step 2, once the density is known:** a check that walks the tree for stand-in markers and diffs
them against the three loci. **⚠ It must be a MECHANISM, not a rule asking anyone to remember**
— this project has watched that shape die twice (`JUSTIFIED-BY`: documented in two places with a
promised sweep, **3 uses, 0 in `crates/`**; and *"re-run the probes by hand after a merge"*, dead
in one day). Candidate venues: a pre-commit hook like the ordinal guard, or a fourth corpus sweep.

**Partly mitigated already** (2026-08-03, same fingerprint): `session-workflow` § Delegation now
says **harvest from the DIFF, not the report**, and briefs let an agent write a stub body while
the integrator assigns the number. That narrows the delegation hole; it does not close the
general one.

### GRAVITY BECOMES A WORLD PARAMETER — **DECIDED 2026-08-02 (user); step 1 LANDED (ungated), steps 2–4 OWED**

**Doc: [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) § *Gravity is a WORLD constant, and it
defaults to Earth*** (with its same-day SDK/geo-pass widening). `25.0 m/s²` was a bring-up
artifact nobody chose; *existence is not standing, applied to a physical constant*.

1. ~~**The flip + one authority**~~ **✅ LANDED 2026-08-02 (`bd0c82c`, renamed `26bf42f`)** —
   ~~⚠ UNGATED, the gate is the first thing owed~~ **GATED since, twice (stamped 2026-08-03,
   staleness F7): the bodies arc gate `0e344ca` (962/0, which also caught and fixed the stale
   `G_WORLD = 25.0` Froude print) and the geo wrap trio (984/0 at `090f778`+).**
   `dc-core::DEFAULT_GRAVITY_M_S2 = 9.81`; **three** hardcoded
   `25.0`s collapsed (`CharacterConfig`, `dc-client` player, and `PhysicsConfig` — the third found
   only by doing it, and its doc comment *named the file holding the authority and copied the
   digits anyway*).
2. **One place to SET it — the configuration half, still open.** Three configs each hold their own
   `gravity_m_s2` field, all merely *defaulting* to the constant, so a world wanting Mars gravity
   sets it in **three places**: the same defect moved from the literal to the field. End state is
   one world-level value the three derive from. **Consumer: E7's per-world manifest** — the same
   loader `CadenceTable` waits on. *Recorded in `world_constants.rs`'s module docs too, because a
   fix that closes the visible half and leaves the other half unnamed is how the visible half
   comes back.*
3. **SDK exposure** (user: *"world grav should be exposed on sdk surface"*) — gravity becomes a
   primitive passes declare against, per the north star.
4. **Pass adoption, one slice each, each with its own literature check.** **Measured 2026-08-02:
   ZERO worldgen passes consume gravity** — every `gravity` hit in `dc-worldgen` is a *doc comment*
   describing the mechanism beside code that never receives it (`creep.rs:18` calls itself *"the
   gravity/mass-wasting member"* and takes no `g`). Real consumers: lithostatic pressure `ρ·g·h`
   (burial/compaction/diagenesis) · grain settling velocity (Stokes) · transport capacity and
   stream power · isostasy · hillslope diffusion. **⚠ A pass may NOT factor `g` back out of an
   empirically fitted constant and call the result derived** — `EROSION_CALIBRATION` is the live
   case, already owed a literature-derived re-pick (P2).

**⚠ ORDER IS LOAD-BEARING AND STEP 1 IS ALREADY BANKED.** Once a pass reads `g`, gravity joins the
seed and the frozen content set as **world identity** and changing it moves **terrain**. Nothing
consumed it on 2026-08-02, which is exactly why the flip went in that day — it moved **no terrain
golden**, only character/player/item motion. **Steps 3–4 close that window permanently.**

**The one taste consequence, now consciously owned rather than inherited:** jump **apex is
unchanged** (the jump is height-parameterised — launch velocity `√(2gh)` puts the apex at exactly
`h` at any gravity, a good piece of bring-up design that survives), but everything is **≈1.6×
floatier in TIME** — longer arcs, slower falls. High gravity for snappy jumps is a common voxel-game
choice and is the likeliest unexamined origin of `25`.

### MEMBERS INTO DEEP HISTORY — **DECIDED 2026-08-01 (user), TOP PRIORITY — ripples through everything below**

**WHAT.** The deep record stops being class-grade: member materials (registry
`MaterialId`s — mudstone, siltstone, sandstone…) enter deep history as first-class
recorded state. ~~Today the ~7-entry Litho roster carries all of deep time and members are
invented at expression (the member dither under formation context); after this arc,
identity is *recorded*, and expression expresses.~~ **(Tense stamped 2026-08-02, staleness
F8: slices 1–2 SHIPPED that half — identity IS recorded, transport carries it, the draw
survives only for genuine degeneracy. The Litho roster's residue dies in slice 4.)**

**WHY (user, verbatim):** *"we'll just keep re-deriving and punting this, while getting
side-tracked on bug fixes coming from class machinery (quantization etc). members need to
go into deep history, proper. big work, big prio. will cause a ripple in all currently
slated work."* The wall it removes, hit three ways in one week: **(1)** transition edges
live on material definitions (DECIDED 2026-07-22) and a class-grade sim structurally
cannot consult them — every class-keyed constant becomes an S-3 parallel rule the day
edges land; **(2)** the member census (journal/0129 § 5): 4 of 10 classes cap the member
dither at a coin flip — expression cannot diversify what history never distinguished;
**(3)** the composition term had to be scoped provenance-not-size partly because the
roster is class-grade (record-terms priors § 1.6).

**UNIFIES.** Block-is-material (one namespace, no proxies) · *"the class system's
fixed-constant roster is scaffolding"* (user, 2026-07-22 — this arc cashes it) ·
transitions-on-material-definitions as the ONE authority both sims consult · the
parent-inheritance sketch (class-grade aggregation, if any survives, becomes
parent-edge approximation with a legible error term — or dissolves) · genesis-passes /
facies ambitions (P5) · the class-vs-member open question filed this same day.

~~**FIRST SLICE — a DESIGN PASS, not code**~~ **✅ DESIGN PHASE CLOSED 2026-08-01, same
day** — `docs/audits/2026-08-01-members-into-history-design.md`, six rulings in its
header (ruling 6 added 2026-08-02 at slice 1's harvest; the header prints them 1–4, 6, 5): the record names **`MaterialId`** (history records the rock, not the road to it) ·
representation **A-CLEAN** (identity per unit, fitness at DEPOSITION, **no class view
survives storage or physics** — killed by the user's entrenchment challenge) · transport
planes **SPARSE DAY ONE** (dense scales cells × registry, the wrong asymptote) · U5
dissolved (corrections #84) · **MM-3/near-path restructure FOLDS IN** (one surgery, not
two).

**BUILD SEQUENCE (drafted 2026-08-01, integrator sequencing — veto welcome):**
1. ~~**The identity swap + deposition-time fitness**~~ — **✅ BUILT 2026-08-01**
   (journal/0136). `DepUnit.species: Litho → MaterialId`, **zero-byte
   (compiler-asserted, the audit's I2)**; member fitness runs at **deposition**, per
   deposited unit, under that cell's own temperature/precipitation that epoch, through a
   new `DeepMember` draw domain addressed `[depositor, cell, epoch, k]`; **every** identity
   writer converted, including the two that are not surface events — pedogenic overprint
   and **burial diagenesis, which re-picks under the slab's real burial P/T, so the coal
   class's rank axis is finally expressible**. Expression reads the recorded id
   (`StrataEvent::dither = false` for deep history; the veneer's dither rides on,
   legitimately). `deep_class_of_species` **no longer runs on the deep-history expression
   path**; `derive_base` stopped up-converting through `reference_material` and the
   `MaterialId`-grade deep-cell inventory (Crux 1) finally receives the grade it was built
   for. `lithology.rs`'s guard comment rewritten (A-2, corrections #84). Stubs **#31
   narrowed, not closed** (identity resolved by construction; the veneer's chunk-centre
   context is live) and **one new stub opened, ordinal pending** (the deep tier's content set is
   hard-wired to vanilla — the door exists, nothing upstream passes through it). Acceptance:
   `examples/member_diversity_probe.rs`, gated.

   **MEASURED (seed 1337, Medium, gated):** both multi-member deep classes now carry both
   members — fine `dc:mudstone` **55.3 %** / `dc:siltstone` **44.7 %**, coarse
   `dc:sandstone` **62.5 %** / `dc:conglomerate` **37.5 %**; the four one-member classes
   read 1, which is correct and not a null (coal records **zero units** on this world —
   corrections #51's finding, unchanged).
   **⚠ THE PRICE THE DESIGN AUDIT LEFT UNPRICED (its I4) IS NOW MEASURED AND IT IS NOT
   ZERO: the merge-key split factor is 2.4053× — 10,951,030 units against 4,552,847 —
   i.e. +97.6 MiB of RESIDENT record (167.10 vs 69.47 MiB at 16 B/unit).** The field is 16
   bytes per unit either way, so the audit's *"zero-byte swap"* holds; what it did not price
   is that identity in the merge key multiplies the units. **The cause is a design fork, not
   overhead:** the member draw is addressed per **epoch**, so a stable environment records an
   alternating stack rather than one thick bed. `(cell, chapter)` addressing is the cheaper
   and arguably more honest alternative (journal/0073's coherent-vs-white lesson, one axis
   over, in time). **Flagged for ruling, not decided.**

   **⚠ THE GATE IS RED ON ONE TEST AND IT IS THIS SAME CAUSE.** Measured against `main`
   `957edfc`, three uncontended samples each side: **Medium pregen 42.79 s → 47.48 s
   (+11.0 %)** and **`Pregen::approx_resident_bytes` 357,169,261 → 472,970,797 (+110.5 MiB,
   +32.4 %)** — the residency corroborating the probe's +97.6 MiB record arithmetic
   independently. `s7_measurements::pregen_time_vs_extent` asserts Medium **< 60 s**;
   uncontended we pass at 47.5 s, but inside the full workspace gate (where that binary's
   sibling test builds its own Medium world concurrently) it measured **62.7 s and FAILED**.
   `main` uncontended is 42.8 s, so under the same contention `main` sits near 56 s — **the
   budget was already ~95 % spent and this slice tips it.** The 60 s is a ratified number
   (S13 called it that); **it has NOT been moved, because moving a gate to admit one's own
   work is not a fix.** Two resolutions, both user-owned: renegotiate the budget (the
   standing *"worldgen time is not a constraint"* position, which S13 already framed as a
   renegotiation rather than a blocker), or take the `(cell, chapter)` draw fork above —
   **one change fixes both costs, because they have one cause.**
   **✅ RESOLVED AT MERGE (2026-08-02, tip `5914e708`): the `(cell, chapter)` interim
   address was applied per ruling 6's "not a real fork" — measured at tip: split factor
   **1.2775×** (5,782,931 units), residency **≈ main** (357,154,365 vs 357,169,261),
   Medium pregen **faster than main** (41.8–42.2 s vs 42.6–42.9 s), and
   `pregen_time_vs_extent` **passes**. Neither user-owned resolution was needed; the
   budget stands unmoved. The two paragraphs above are the dated per-epoch measurement,
   kept as the record of why the address matters.**
   **⚠ GOLDENS: the final address change moved goldens that are NOT re-captured — slice 2
   moves them again (draw retirement) and owns the single capture. Until then the golden
   families are EXPECTED RED on main: the RECORD halves (`GOLDEN_RECORD` + 5 variants),
   CONTENTS (two Medium triples), the SURFACE family + `GOLDEN_GEOTHERM`/`GOLDEN_FLUX`/
   `GOLDEN_FAR_SURFACE`, and `GOLDEN_HEAD`. Accepted as-is by user ruling 2026-08-02;
   the full workspace gate was deliberately NOT run on merged main — slice 2's merge
   gate is the next full verification.**
2. ~~**Sparse member-grade transport**~~ **✅ SHIPPED 2026-08-02 (journal/0141, merge
   `07bd694`; foundation journal/0138 + the erosion/ split journal/0139 preceded it).**
   The four budget planes are CSR-sparse over `MaterialId` — **measured 44.67 MiB,
   0.70× the class-grade dense planes replaced** (p = 3.721 mean / 8 max, 14-material
   axis); Law-3 closure re-proven at ~1 ULP with derived bounds. The composition term
   folded in (per-face split in the sparse rows). **Ruling 6 executed: 77.21 % of
   recorded metres take identity from the arriving composition and never reach a draw —
   and 31.9 % of the whole record would have been named a DIFFERENT rock by the
   deposition site's own climate.** The salt collision closed (`DeepTimePerturb`
   registered byte-identically; `DeepMember` → `0x5900_0003`). All 14 expected-red
   golden families captured once, whys per family (semantics vs float accumulation —
   #89's discipline). Full workspace gate on the branch 928/929; the one red was the
   60 s pregen budget under gate contention, **renegotiated by the user at merge
   ("as long as it doesn't take 20min") → 1200 s**, prior value kept for audit.
   Record units 1.9064× (the record got bigger because it got more honest); slice 3's
   ratified packed `DepUnit` recovers it. Corrections #90–91, stubs #37–38 from the
   slice. **Record-terms ruling 3's re-derived numbers are in journal/0141 § 11 —
   ready for its re-entry.**
3. ~~**The near-path restructure** (ruling 5)~~ **✅ SHIPPED 2026-08-03 (journal/0145 —
   renumbered from a colliding 0144, the ordinal guard's catch: bodies' gait-gravity
   entry claimed 0144 first (22:51 vs 02:09), the later claimant moved per the
   3cab931 precedent;
   merge `cd1058a`; 3a+3b as ONE merge unit — deviation priced by the audit, reasoning
   in the journal).** Per-voxel-column record membership (`SubCell` + `cell_bundle`, the
   F2 mass-coupled trio inexpressible to un-bundle) + packed `DepUnit` L-8 (8 B/unit,
   accessors, sub-quantum carry, grain 3 bits UNSET for FS-A, **mover IN the merge key**
   on M0's measured 1.0308×). Full trio on the branch: **938/2, both reds accounted**
   (the inherited dc-client red · the hollow-gate bound corrected to 0122's derived
   10 m bar — it was an underived 1 m sitting on the dimple floor). **The ~460 m near
   tile is dead in code; the ACCEPTANCE WALK is owed** (borders → interfingered
   contacts, knowing the far-tier blend semantics ride along — P-6/P-7).
   **DESIGN PASS DONE 2026-08-02 (`docs/audits/2026-08-02-p11-slice3-design.md`; build
   banner in its header).** Load-bearing findings: the three NEAREST reads are
   MASS-COUPLED (record + regolith `H` + ledger bedrock slot must move as one bundle per
   column or Law 3 leaks at every frontier — F2); blast radius re-measured **14 files /
   ~71 sites** (the gated tour probe post-dates the old count); **the layout is RULED
   2026-08-02 (user, P-2): L-8, 8 B/unit, record 116.31 → 58.15 MiB** — premised on the
   accessor conversion making future widening cheap (user endorsed accessors as the
   repo-wide pattern; the hint byte stays unforeclosed as exactly such a widening, with
   eco/civ-era axes the named who-knows-how-soon unknown); **U4 RULED same day: 5 φ
   classes** (grain = 3 bits); the grain split-factor gate structurally cannot bind at slice 3 (grain
   is uniform until FS-A writes real state — the gate transfers to FS-A's writers; the
   measurable-today analogue is the MOVER split, owed before the mover joins the merge
   key). ⚠ F6: the near dither inherits the far tier's cell-wide-blend semantics the user
   rejected by eye — not a contradiction (ruling 5 folded MM-3 in knowingly) but the
   acceptance walk must verdict it knowing borders become ~460 m gradational interfingering.
   **CO-RIDER, U5 RULED 2026-08-02 (user): the packed `DepUnit`** — fixed-point
   thickness + bitfield axes, funding grain (3 bits) AND the agent axis (stub #25) from
   reclaimed padding/over-precision at a net residency REDUCTION (~16 → ~8–12 B/unit);
   one layout surgery, one golden move, one migration. **Gated on a measured grain
   split factor** (the #88 count-model lesson) and the sub-quantum remainder-carry
   (Law-3 bound becomes derivable: quantum × merge count).
   **✅ BUILT 2026-08-02/03 (slice-3 worktree, journal/0145 — *was `journal/pending-p11-slice3`,
   the dangling-pointer shape again, fixed 2026-08-03 staleness F9*; audit header
   carries the banner).** M0 measured first: mover split **1.0308×** → **M-1, the mover
   is IN the merge key** (P-4 resolved integrator-side, under the ≲1.1× bar); max unit
   thickness **65.38 m** (2⁻¹⁰ m u32 quantum stands; u16 would have overflowed);
   `run_strata` = **0.2 % of `column()`** (the 4× prior is noise). Restructure: `SubCell`
   (own module) + `ColumnRec.strata → records`/`cell_of` + `DeepField::cell_bundle` (the
   F2 mass-coupled read) + the `NearRecordMembership` domain on `Octaves`; acceptance
   instrument gates in `appearance_tour_p11` (structural asserts — the audit's binomial
   floors were unsound under its own coherent-source pick, corrected loudly in the
   banner). Pack: 8 B/unit (bits + u32 quanta), accessors everywhere, per-cell carry,
   grain named + UNSET (`grain()`/`set_grain()`/`GRAIN_UNSET` await FS-A). Stubs #25
   discharged, #31 heir re-pointed. Deviation: shipped as ONE merge (the § 7-priced
   combined option — the two surgeries share fourteen files); all-family golden move in
   that merge, whys per family in test comments.
4. **Litho dissolution residue** — delete `deep_class_of_species` and the ~250
   remaining class-speaking sites; re-shape or minimally patch the far `ShareVec<6>`
   site (rejected-interim; its real heir is the far register — do not gold-plate).
**Record-terms ruling 3 (face-vs-unit) RULED 2026-08-02 (user): FACE** — composition as a
sparse CSR sidecar to the flux record; `DepUnit` stays single-species (U5 packing
proceeds). The user's rationale is directional, beyond cost: the face carries *what
direction / how materials entered a region* — provenance hints for the refinement
operators (full record: priors header). **Rider from the same ruling: the partially
implemented refinement machinery needs re-visiting now that material identity is in the
record** — folded into member #1's resumption below. **CONTINUATION SLOT:** P10 (grain
axis) and member #1 resume against the member-grade record; P5 stays its own
deliberately-opened arc.

### THE GRAIN-SIZE CONTINUUM — a real size axis on sediment — **SEQUENCED 2026-08-01 (user ruling: "option 2 — I just don't want to lose track of anything needing done")**

**WHAT.** Sediment gains a **grain-size state**: source rocks release a size *distribution*
(not their sheet's single `grain_size_mm`), transport **evolves** it — abrasion shrinks grains
downstream (Sternberg, measured e-folding 50–100 km), sorting separates them (coarse settles
first; the ratified capacity+competence+settling sketch, `material-behavior.md` § 13) — and
the record stores the evolved distribution where deposition happens.

**WHY.** The § 8 prize cross-section opens with *"a channel gravel with its placer streak,
fining upward"* — **all three are grain-size children and none is expressible today.** The
existing `grain_size_mm` is a broken proxy at the deep tier: the roster spans ~two distinct
sizes, ClasticFine ≡ OrganicSoil exactly, peat outranks sandstone in settling, and nothing is
gravel (measured: `docs/audits/2026-07-29-fluvial-record-terms-priors.md` § 1.6). The fluvial
record-terms ruling (2026-08-01) records load **composition** now and explicitly does NOT fake
grain size from the proxy — this arc is the named heir, ON THE BOARD so it cannot be lost.

**UNIFIES.** material-behavior § 13's user-endorsed sorting sketch (*"sound and promising,
refine on build"*) · stubs **#23** (`settle_energy` blindness) · `earth-processes.md` § 3e's
*"widen recorder tags (agent axis + grain continuum)"* — the one prior corpus ask for exactly
this · P2 (Shields/Sternberg/gravel-sand-transition literature bands become the calibration
targets once magnitudes are real) · refinement member #1 (fining-upward and placers are what
the channel operator expresses *with* it).

**FIRST SLICE — a design pass, not code:** what IS the size state (φ-classes? a small
histogram per load parcel?) · where it lives (a release-spectrum on materials + evolving
state on the load) · how it composes with the composition term just ruled · literature
anchors per the measure-against-the-literature rule. **→ DESIGN PASS DONE 2026-08-01
(`docs/audits/2026-08-01-p10-grain-axis-design.md`), and its rulings have landed: U1
(grain is an AXIS, 2026-08-02) · U5 (packed-`DepUnit` funding, 2026-08-02) · U7 RULED
2026-08-02 ("R2 it is — authored edges"): release spectra are pack-authored EDGE
PRODUCTS, and the primitive is general per the user's directive (a product may be a
loose grade of the source OR a totally different material — mods may want this; engine
admits any `MaterialId` product, provenance-keeping is vanilla's authoring; recorded in
`material-behavior.md` § 3). U6 collapsed by U1. **FS-A ✅ SHIPPED 2026-08-03 (journal/0146,
merge `4635056`): `EdgeProducts` on the material definition (per-mille shares, exact-sum
by equality, compile-closed, multi-modal capable), 8 vanilla grus/saprolite tables
literature-cited, `InvCtx::release` with a tested bit-identical vanilla collapse. On the
shipped world: 130,426.8 m shed as 35/45/5/15 % gravel/sand/silt/clay (flag-on
instrument; production stays flag-off). ⚠ THE GRAIN WRITER WAS WITHHELD ON SEMANTIC
GROUNDS AND THAT IS THE RIGHT CALL PENDING RATIFICATION: the split-factor gate PASSES
(≤1.0329×, under U5's bar) but grading a recorded bed by its own release spectrum
answers "what would this rock shed", not "what grain IS this bed" — the slice-3 audit's
rejected O-1 wearing a new hat. The honest writer is P10's PROPAGATED grain; the seam
(`grain_write_seam` + `set_grain`) is marked and every unit verified GRAIN_UNSET. FS-A's
acceptance WALK is owed (stripped upland vs distal basin; tour exemplars in the merge
report).** Still open: U2, U3; and
the grade/form LEGIBILITY presentation question (user, filed § Observed 2026-08-02).** **CONTINUATION SLOT:** transport
evolution (abrasion + sorting) → the record axis → operator consumption. **Calibration is
gated on P2** (pre-P2, no cell can carry sand — competence ceiling 0.283 vs 0.840).

### POSTURE AND GAIT ARE DERIVED, BAKED PER SPECIES, SAMPLED CHEAPLY — **CAUTIOUSLY RATIFIED 2026-08-01 (user)**

**Doc: [`docs/design/posture-gait.md`](docs/design/posture-gait.md).** Bones (§§ 2–6) ratified;
**§ 7 members are directions, not build orders** — each needs its own design pass against the
bones, and a pass that finds them missing machinery says so loudly. Same reading as
`refinement.md`'s 2026-07-29 ratification.

**WHAT.** Stop authoring what physics determines. A body's resting posture, hip height, knee
angle, stance width, bob amplitude and cadence become **outputs** of `(segment tree + masses)`,
solved **once per species at pack build** and sampled at runtime. A gait is two keyframes
(`neutral`, `extreme`) plus a per-limb `(phase, amplitude, duty)` triple. The sim owns a small
quantized **phase**; the client adds only cosmetics.

**WHY.** Measured, not supposed (journal/0130–0131): the pelvis is pinned at an authored hip
height and the legs are asked to reach the floor, so `dc:body/longleg` **squats at −56.2°**
where a real animal would just stand taller. `root_bob_m` is a **leaked requirement** — a walk's
bob is what alternating stance legs *do*, so authoring it creates two authorities for one
quantity. Four absolute-metre stand-ins became definitions because, with one body plan, **a
length is a ratio** (anti-shape A-1). User's verdict, watching it move: *"the squat does not read
realistic… posture actually is responsible for keeping center of gravity."*

**How it UNIFIES.**
- **It is the two-clocks doctrine applied to bodies**, and structurally the *same move* as
  baking a frond texture per species and inheriting it down the phylogeny — two instances of
  one shape.
- **It is what makes the body/evolution arc reachable at all.** Authored clips break the moment
  topology changes, so an evolution pack would generate bodies nobody could animate. A bake from
  `(segment tree + masses)` means a **mutated body gets a plausible stance and gait by
  construction**, no animator in the loop. This is the unblock for the whole
  engine-owns-bodies / packs-define-them / evolution-mutates-them chain.
- **The mass integral is shared three ways** — harvest yield, evolutionary fitness, standing
  posture — which is the independent third argument for per-segment materials.
- **S-9, third instance in one conversation** (meadow↔tuft, swarm↔individual, species
  gait↔this wolf's limp). Individual deltas are not a new mechanism.
- **It is checkable against the literature**: gait taxonomy *is* phase offsets; duty factor is
  the walk/run discriminator; cadence scales as √(g/L) with the transition near a predictable
  Froude number. A baked gait can be falsified against published bands rather than tuned to a
  look.

**FIRST SLICE — the resting-posture bake (member #0). ✅ SHIPPED 2026-08-02 (journal/0137,
design pass `docs/audits/2026-08-02-posture-bake-member0-design.md`): acceptance met by
measurement — longleg 1.020 m > biped 0.880 m, standing not squatting; hip is an output;
predicted table held to 1e-9. Gate debt per the 2026-08-02 batching ruling: full workspace
gate + dc-client suite (leg_rigs hoist) batch with the consumer slice.** Original spec:
Headless, no rendering change, no
engine constant moved. Derive a resting posture and hip height for a `BodyPlan` from its own
geometry, using segment volume as the mass proxy **with per-segment material named as its heir**.
- **Acceptance is an OUTCOME, and it is falsifiable:** `dc:body/longleg` must come out
  **standing taller than the biped, not squatting**; hip height must be an output, and the three
  shipped plans must land in plausible proportions. Report the numbers.
- **Name what it is NOT:** it does **not** fix the hover (that is `character.rs:219`'s missing
  `root_bob_m`, corrections #80, and it is a separate call); it does **not** touch the clips, the
  renderer, `CROUCH_ROOT_DROP_M`, or the half-voxel window; it does **not** build gait,
  colliders, or damage. A slice that "fixes the look" has escaped its scope.
- **The one hazard:** the cheapest correct answer for a symmetric standing biped is *legs
  straight, hip at leg reach*. That is **the right answer** and must not be mistaken for a
  degenerate one — the effort-minimising solve is a later refinement, not this slice.

**CONTINUATION SLOT — this is a slice OF the tier, and the arc continues with:**
(a) the **gait bake** — **✅ SHIPPED 2026-08-03 (stamped by staleness F2: the entry still read
as an expanded docket after the work shipped three times over): headless `bake_gait`
journal/0144 (predicted table confirmed ~1e-5) → consumer journal/0147 (Froude ladder,
`root_bob_m` out of the schema, `biped_walk` retired) → walked journal/0148 (verdicts
recorded; new stand-ins `swing_gain` stubs #43 + the 12 fps aliasing finding); arc gate
`0e344ca` 962/0.** ~~(docket EXPANDED
and user-ratified 2026-08-02: `posture-gait.md` § 7 member 1's banner now carries the honest
input inventory (geometry + volume-at-density-1; no strength/mass until B6), the four-kind
AUTHORED-KNOBS taxonomy with the taste-vs-stand-in rule, the keyframed-non-locomotion
guarantee for both authoring routes, and corrections #94's gaze/bend vocabulary)~~; (b) the **sim-side
phase tuple**, which brings the firewall's new line *and* the 20 Hz vs 12 fps cadence question
(0.05 s and 0.0833 s do not divide — the stepping must land evenly, and that is a choice about
the stop-motion identity, not a technicality); (c) **derived collider sets** with bounded `k` and
yaw buckets, retiring the world-global `CharacterConfig`; (d) **per-segment damage** against the
nominal pose, and the injury→gait-delta loop that produces a limp nobody authored.

**Not foreclosed, and it costs three properties today:** bake **parameters, not frames**; the
seam is a **function signature** (`pose(species_gait, instance_delta)`, identity default,
byte-identity tested); and **do not over-quantize** the baked gait or a 3 % limp becomes
inexpressible — which is corrections #80 in a new costume, already paid for twice.

~~**Blocked on nothing.**~~ **SUPERSEDED the same day (user): sequencing is `structure →
posture → gait`** — and **B0 SHIPPED 2026-08-01 (journal/0135)**, so **member #0 (this first
slice) is now STARTABLE**, with its contacts input reading B0's **declared soles**
(`segments_with_role`) instead of deriving feet from three bipeds' naming habits — the N=1
trap (corrections #78) is structurally closed. ~~The open user call in `bodies.md` § IK …
is **downstream** of this, not upstream.~~ **The § IK call is CLOSED, not open (corrections
#81)** — dissolved by `posture-gait.md` § 1 the day it was ratified; this paragraph was written
before that was caught. **Binding-key ruling 2026-08-01 (user):** clips stay address-bound
through B0; role binding for animation + cardinality enforcement = **stubs #34**, decided at
the gait-bake design pass, before B3 moves the firewall (`posture-gait.md` § 7b).


- **Octree-substrate follow-ons (re-filed 2026-07-29 — the parent entry moved to history
  with journal number intact; these four were live inside it and stay owed):**
  persistence + the dirty-rail · synthesized sub-surface strata · partial-coverage
  composition · deep-span greedy merge. Unowned, unsequenced relative to each other;
  see the archived entry for each one's reasoning.

- **✅ ALL FIVE USER DECISIONS RULED 2026-07-28 — journal/0120.** Surfaced, parked, and then
  taken in a dedicated unpacking session the same day. **Rulings are inline against each item
  below; the context is kept because the reasoning is what makes each ruling legible.**
  **⬇ The five rulings and their reasoning moved to [`ROADMAP-history.md`](ROADMAP-history.md) on 2026-07-29 — nothing there is owed. What stays live here is the *not user-owned* tail below.**
  **What shipped from these rulings:** `worldgen.md`'s ON-HOLD banner + new § *Sequencing* ·
  `north-star.md` § *The core/plugin boundary* — **the refinement tier RESOLVED to (c)** ·
  CLAUDE.md's *recorded ambition vs unratified content* clause and the **immutable body,
  mutable header** spike policy (read-first item 5) · banners on `S10-results.md` and
  `S2-results.md` · `spines.md` § 3's **third exit** (held-as-candidate) · corrections #12's
  supersession note. **Nothing here is owed.** *Two follow-ups the rulings created are filed
  as their own entries below — the unswept-banner backlog and the completeness-check gap.*


  **Also surfaced, NOT user-owned — recorded so they are not lost with the above:**
  - **`spine-audit/SKILL.md:47` tells every future auditor to grep `JUSTIFIED-BY`** — a marker with
    **3 occurrences and 0 in `crates/`**. `spines.md` § 5 flagged exactly this on 2026-07-24 and
    left it as *"a flag to the main session, not a unilateral rewrite"*; the instruction is still
    there. **Main session owes: does the marker earn its first real uses, or does § 5 get rewritten
    around the prose form actually in use?**
  - ~~**`session-workflow/SKILL.md:889-894` still presents archive-by-status as a future proposal**
    (*"**Moving** older Shipped entries… **would** shrink the live board"*) — it shipped
    2026-07-26. Integrator fix, no ruling needed.~~
    **✅ ALREADY FIXED — struck 2026-07-29 (baseline sweep S5/F8).** `session-workflow/SKILL.md`
    now reads *"**Archive by STATUS, not by age. — ✅ SHIPPED 2026-07-26 as
    `ROADMAP-history.md`**"*, and the quoted text is not in the file
    (`grep -n "would shrink the live board\|Moving.*older Shipped"` → nothing). This row was the
    only thing still claiming otherwise. *The fix landed with no back-pointer to the row that
    asked for it — the same one-directional shape the row itself was reporting.*
  - **New Observed candidate:** `approx_resident_bytes` is **non-monotone in extent** (Medium
    **377 MB** > Large **213 MB**) because the deep grid is width-capped
    (`cell_m = max(extent_m/DEEP_MAX_WIDTH, DEEP_CELL_M)`) and record size is
    deposition-dependent. A reader of `s7_measurements`' table has no note to reach. *Not caused
    by the removal.*

- **🔴 THE BASELINE SWEEP'S FINDINGS — ~80 pairs, 100 % corpus coverage, RANKED AND MOSTLY
  UNAPPLIED**, *and now largely APPLIED — the four user calls and all five highest-blast-radius
  items were ruled or applied and moved to [`ROADMAP-history.md`](ROADMAP-history.md) on 2026-07-29* (2026-07-28, `docs/audits/baseline-2026-07-28/` — nine audits, ~3,900 lines,
  watermark `f652b60`). **The audits ARE the record; this entry is the index.** Applied at
  baseline: only the integrator's own same-day stalenesses + the `ARCHITECTURE.md` ON-HOLD
  banner. *Everything below is real, cited `file:line` on both sides, and NOT yet fixed.*
  - **🟠 STRUCTURAL — the dominant failure mode, now measured rather than suspected:** **every
    enumeration checked came back short.** corrections #65 names 3 superseded topo-sort sites
    (`geology.md:29-32` is a 4th) · north-star Deviations #2 names 3 voided sections (**6** are
    live, incl. its own opening blurb) · `stubs.md` "four" (5) · `CLAUDE.md` "four" ecology
    heirs (**2**) · `doc-topology`'s own "three confirmed-live pointers" (**2 repaired**).
    **The code layer has had this control for months** (`build_checked` refusing a class with
    zero members); the docs never got it. *Five instances in one afternoon promotes this from
    "recorded as a pattern" to the thing most worth fixing.*
  - **🟠 BULK, mechanical:** **37 Observed entries (29 %) already say ✅ DONE in their own
    bodies** — a pure archive job, no judgement. **8 spike/audit files need supersession
    banners** (5 have, 24 correctly need none, each with its reason recorded). **`journal/0059`
    is a LOST entry** — orphaned assets, a substantive walk surviving only as
    `ROADMAP.md:4811-4830`, never in any deliberate-gap list. **journals 0110/0111/0112 have no
    archive entry at all**, including **0111, the scale recalibration**.
  - **🟠 A LIVE DEFECT, not a staleness:** **`flow_cost_probe` — the probe the entire
    `test = true` rule was earned on, named twice in `dc-worldgen/Cargo.toml:21` as the
    cautionary tale — was never converted.** `examples/flow_cost_probe.rs:412` still holds a
    bare `assert_eq!` in `main`, no `#[test]`, no `[[example]]` block. **The remedy's own
    motivating case is unremediated.** Candidate corrections entry.
  - ~~**⚠ NOT COVERED BY THIS BASELINE: `spine-audit`'s question.**~~ **DISCHARGED 2026-07-29:
    the first spine-audit ran (`ec9858e`), its watermark is set (`96ab14b`), and the S-6
    finding it was waiting on was APPLIED (`e568c5b`).** *(Staleness sweep 2026-07-29 F7 —
    three clauses of this bullet had gone stale in one day.)*


- **🟠 THE SUPERSESSION-BANNER BACKLOG — the policy shipped, the sweep did not** (created
  2026-07-28 by the decision-5 ruling; **not** a defect, a deliberately-bounded scope).
  **Immutable body, mutable header** is now doctrine (CLAUDE.md read-first item 5), and it was
  applied only where a refutation was *already known*: `S10-results.md` and `S2-results.md`.
  **The corpus was never swept.** Measured baseline: **14 of 30** audit/spike files carry no
  staleness marker of any kind, and **8 of 15** full-path correction→file edges are
  one-directional. *Those numbers are the size of the hole, not a list of defects — a file with
  no banner is only wrong if something actually refuted it.*
  - **The sweep is mechanical on one side and a reading task on the other.** Every
    `corrections.md` entry that names a `docs/spikes/` or `docs/audits/` path is a candidate
    edge, and whether the named file already points back is a grep. **What is not mechanical**
    is a refutation that never cited its target by path.
  - **Do it as a `doc-topology` pass, not a new instrument** — this is exactly that skill's
    job (docs vs *each other*), and it already forbids the sweeper from resolving what it
    finds. **The banner text is a claim about what was refuted; it is not a sweeper's to
    author** where the refutation is contested.
  - **⚠ The policy's real test is the NEXT correction written, not this backlog.** If the
    same-commit obligation holds, the hole stops growing and the backlog is finite. If it does
    not, sweeping is treating a symptom — *and we would know within a week, which is the
    cheapest possible falsifier.* **Watch that before investing in the sweep.**
    - **✅ THE FALSIFIER RAN TWICE AND THE POLICY HELD — sweep UNBLOCKED (staleness sweep
      2026-07-29 F10):** corrections #72 stamped journal/0116 (verified at source), #73
      stamped both its targets. The backlog is finite; the banner sweep is now a justified
      spend. **With one sharpening from the same sweep's meta-observation: the obligation
      holds where the target is a spike or journal, and FAILS where the target is a ROADMAP
      body entry** — six of its eleven findings were answers recorded in `spines.md` / the
      graph / a journal / the close block and never written back into the asking ROADMAP
      entry, twice inside entries an author had open for a different edit. The sweep covers
      spike/audit banners; the ROADMAP-body variant has no mechanism yet.

- **🟠 NO ENUMERATION IN THE DOCS IS CHECKED FOR COMPLETENESS — and 2026-07-28 added a fourth
  instance while RESOLVING one** (sharpened by the refinement-tier ruling). The user's
  engine/plugin split closed north-star's *"an entire tier absent, never decided, arrived at by
  default"* — **and the user flagged the replacement as non-exhaustive in the same breath**
  (*"this list may not be exhaustive"*). So the boundary is decided and its inventory still is
  not; both docs now say so explicitly rather than reading as closed.
  - **The asymmetry is the finding, and it is damning:** the **code** layer already has this
    control — `build_checked` refuses to build a world if a class a pass selects from has zero
    members, which north-star § *Validation by construction* calls the entire meaning of the
    phrase — **and it has never been applied to the documents that specify the code.** Every
    known instance (stubs #29's four post-incision phases with isostasy missing; the
    core/plugin boundary; a pass's `reads` set) was **found by a human noticing.**
  - **Not sequenced as a build.** The doctrine that governs it is already ratified —
    `session-workflow` § Seam-first #6, *"do not build the general mechanism first: convert the
    cheapest cold seam, let it teach the shape."* **Four hand-found instances is a thin evidence
    base.** Recorded so the fifth instance lands against a named pattern instead of being
    re-discovered. *(This bullet previously cited `stubs.md:22`'s "general registry… four
    conversions" clause — **withdrawn 2026-07-28**, there was never a registry. The seam-first
    rule alone was always the one doing the work here.)*

- **🟠 DOC-TOPOLOGY RESIDUALS — the 19 findings not actioned 2026-07-26** (full audit:
  `docs/audits/2026-07-26-doc-topology-sweep.md`). Six were actioned the same day (the
  ecology carve-out, the field-solver framing, the two ORDER strikes, the S-/A- undercount,
  the CLAUDE.md trust-tier line). **The rest are owed, and are recorded here rather than
  left in an audit nobody re-opens:**
  - **The ABI spike is described as "locking the SDK shape"** — contested by north-star
    § Deviations 1, which defers the whole trust/backend question.
  - **⚠ THE COAL EVIDENCE BASE IS LABELLED "THE PRODUCTION WORLD" IN FOUR LIVE DOCS**
    (~~`stubs.md:348-351`/`:360-362` — the sentence justifying `COAL_BURIAL_M = 8.0` —
    plus `corrections.md:1164`~~; `ROADMAP-history.md:1228/1234`, `journal/0066:43`;
    `geology.md:163` carries the number with **no world label at all**).
    **⚠ ADDRESSES REPAIRED 2026-07-29 (baseline sweep S5/F7) — locate these by CONTENT, not by
    line.** `COAL_BURIAL_M = 8.0` is in `docs/design/stubs.md` § 13/§ 14 (`deeptime/biotic.rs`
    reference) and the *"production world"* label is the *"of 35 382 peat-derived units in the
    production world exactly 13"* sentence in the **same entry**, not at `:348-362` (which is
    the **wave-climate** stub). The `stubs.md:378-379` *"user ratification directly on top"*
    citation addressed a **section heading**. In `journal/corrections.md` the surviving
    *"production world"* uses are in the charcoal, residency and `exhum`/`t_crust` entries, not
    at `:1164`. *This is corrections #67's shape exactly — every one of these was accurate when
    written and now addresses different text, indistinguishable from a paraphrase.*
    **The FINDING is unaffected and still live:** it is the **warm reference** fixture
    `0x0D5E_ED57_2026`, and the shipped world has **zero coal**
    (corrections #51). **The label is wrong with certainty; the correct VALUES are unknown**
    — they predate the geotherm recalibration — so this needs a **measurement**, not an edit.
    Only the addresses rotted.
  - ~~**NEXT SWEEP'S SPINE: `ROADMAP.md` § Observed (~1,970 lines)**~~ **DONE 2026-07-29:
    swept (`d69cdb3`) and reduced to 1,162 lines by archive pass 2 (`610d2b5`)** — the
    "structurally could not reduce" claim did not survive the by-status archive.
    *(Staleness sweep 2026-07-29 F8.)*

- **🟠 THE COAL CANDIDATE COUNTS ITS OWN HALF-THICKNESS AS BURIAL DEPTH** (defect, surfaced
  by journal/0114's confound analysis; **user ruled 2026-07-26** that burial-dominant coal
  rank is correct physics — *"of course we want to do what Earth's coal does"* — which
  leaves this as the one genuine defect in that thread rather than one option among three).
  - A bed is not buried by itself. At the shipped world's ~2 m overburden the half-thickness
    term was noise; at the calibrated world's 451 m it **dominates**, which is why the
    geotherm's coal-relocation separation collapsed 1.37× → 1.02× and its guard had to be
    restated as a depth-stratified claim (a real loss of magnitude sensitivity, recorded in
    the test's doc comment).
  - **The integrator originally listed this as one of three OPTIONS.** It is not an option;
    it is a defect, and the user said so. *Recorded because the framing error is instructive:
    a defect surfaced inside a decision inherits the decision's framing.*
  - **Blast:** coal rank everywhere once `calibrated_rates` flips; nothing today.


- **🔴 CORPUS ADDRESSABILITY — FRONTMATTER, TAGS, QUERY SCRIPTS, AND VERSIONED STANDING
  MODELS** (user, 2026-07-26 close — **a design conversation booked for next session**, not a
  build). *"A way of adding frontmatter, tags, and scripts for querying the corpus that raise
  related docs, open designs, etc. We should be versioning our standing models and always
  referring to versions when cross-referencing… I think this could be strong for wrangling
  this beast before we're further dug in."*
  - **WHY IT IS THE RIGHT NEXT MOVE ON DOCS OPS, stated from measurement rather than taste.**
    Today's two docs-ops shipments addressed **volume** (the archive) and **topology** (the
    `doc-topology` sweep). Neither makes the corpus *addressable*: a reader still cannot ask
    *"what else bears on this noun?"* or *"which version of the pass model is this doc written
    against?"* except by knowing to grep for it. **The first sweep found 19 contradiction
    pairs and five of its top eight were unsuspected** — i.e. unreachable by grep by
    construction. A sweep is a periodic human-scale control; addressability is the mechanism
    that would make most of those findings impossible to author in the first place.
  - **VERSIONED STANDING MODELS is the load-bearing half.** Today's failures were *not* stale
    facts — they were docs written against **superseded versions of a model** while reading as
    current: `north-star.md`'s own line 21 against its § core/plugin boundary; `ideas.md`'s
    ORDER clause against `ARCHITECTURE.md`'s authored-order decision; the coal evidence base
    labelled *"the production world"* against a fixture. **A cross-reference that named a
    model version would have made all three self-diagnosing** — and would give the
    `doc-topology` sweep a mechanical target instead of a reading task.
  - **OPEN, and deliberately not pre-designed** (this is the user's thread; recorded so it is
    not lost, not to constrain it): what carries a version (a doc? a § ? a decision?) · how a
    version is bumped and who may bump it · whether tags are free-form or a closed vocabulary
    (the same open-vs-closed question the pass-resource vocabulary just answered) · what the
    query surface is (script, hook, skill) · and how it degrades when a doc is *not* tagged,
    since partial adoption is the only realistic path.
  - **CROSS-REF:** it subsumes part of the still-open **file-size conventions** item — a file
    that is addressable by section may not need to be small — and it is the third leg of the
    docs-ops triad beside the archive and the sweep.
  - **🟢 READING PASS DONE 2026-07-28 — [`corpus-knowledge-notebook.md`](docs/design/corpus-knowledge-notebook.md)
    (the argument) + [`corpus-knowledge-evidence.md`](docs/design/corpus-knowledge-evidence.md)
    (the reading).** User-directed: *no design opinions until the systematic deep reading is done
    personally*, and *"a theory that does not habitually cycle through empiricism fails this
    task."* **The design conversation is still open and still the user's**; what is settled is the
    *characterisation*. Headline results, because each constrains the ontology and two contradict
    the sketch's framing:
    - **The corpus is amended IN PLACE, AT THE CLAIM SITE, ALMOST PURELY BY ADDITION.** Design
      docs delete **2–4 %** of what they add (`flow` 3 %, `material-behavior` 3 %, `north-star`
      4 %, `ideas` 2 %, `session-workflow` 2 %); only `ROADMAP.md` is genuinely revised (**49 %**).
      Hunk positions spread through the body (mean 0.41–0.81; `spines`/`CLAUDE`/`ROADMAP` put only
      **3–9 %** of hunks in the final tenth). **Consequence, mechanical: `doc-topology` shape 6's
      own check — *"diff the paraphrase against its source"* — STRUCTURALLY CANNOT FIRE, because
      the source still literally contains the sentence the paraphrase was made from.**
    - **Where the falsifier actually was — TWO INDEPENDENT CODINGS, quoted as bands** *(the
      second agent was forbidden to read the first analysis; audit
      `docs/audits/2026-07-28-corrections-recoding.md`)*: MEASUREMENT **37–46 %** · CODE
      **22–36 %** · THE USER 9–12 % · SAME-ARTIFACT 9–11 % · **DISTANT DOC 1.5–5 %** ·
      LITERATURE 2–7.5 %. The independent coder derives that **46–66 % needed no experiment at
      all** — the falsifier was already in the repo — and puts **DISTANT DOC at ONE entry, calling
      even that a bad fit**. **So "surface the related documents" targets the smallest band either
      coder found**, and the direction is agreed even where the magnitudes differ by ~2×.
      In every SAME-ARTIFACT case the refuting text was already on the author's screen (#58 forty
      lines · #65 two sentences · #53 two subsections · #19 the author's own quotation from hours
      earlier). **What was missing was not access; it was an obligation to reconcile before
      writing.**
      *⚠ **29 of 67 entries (43 %) were coin-flips for both coders**, so these are bands and not
      measurements. This bullet quoted the single-coded figures as settled for several hours after
      the re-coding revised them — **the "summary that outran its source" shape, committed by the
      author of the finding, in the live board.** Fixed at the wrap; recorded rather than quietly
      corrected, because it is the cheapest possible demonstration that the shape does not care
      how well you understand it.*
    - **⚠⚠ STALENESS IS 3–8 % OF OUR RECORDED FAILURES; ~85–91 % WERE WRONG THE DAY THEY WERE
      WRITTEN.** The sharper question — *for a watcher to fire, a change event must exist, so did
      the claim BECOME false or was it born false?* — was coded twice: **GENUINELY STALE 2 (3.0 %)
      independently, ~5 (8 %) by me** (my overcount, in the direction that flatters tooling; it
      kept only #11 and #35, both git-verified, and documented a **no-true-era** for #7, #12, #27,
      #32, #42, #57, #66). **So a stale-ref detector has a single-digit ceiling BY CONSTRUCTION —
      for ~90 % of these there is nothing to watch.** The binding failure is at the moment of
      **assertion**, not decay. *And it explains every control that has worked here: all six
      "make-the-illegal-state-unrepresentable" instances are **write-time impossibilities**, not
      alarms. The project has been solving the 85 % all along, in code, and never once in the
      corpus.*
    - **⚠ THE ADOPTION LAW, after the assistant falsified its own first version.** *"Adopted iff a
      machine consumes it"* is **false** — five documented conventions with no consumer sit at
      97–100 % (journal filenames **118/118**, screenshot names **153/155**, `blogworthy`
      **117/118**, `Co-Authored-By` **753/776**, `heir` **651 uses**). What survives: **a
      convention is adopted when it is inseparable from an act the author must perform anyway, or
      is the natural expression of the thought they are already having; it dies when it asks them
      to RESTATE in a second notation something already said in prose.** `JUSTIFIED-BY` —
      documented in `spines.md` § 5 *and* `spine-audit` check #4, with a promised sweep — got **3
      occurrences, 0 in `crates/`**. Cleanest test, relation held constant: **`heir` 651 vs
      `HEIR:` 48 = 7.4 %.**
    - **⚠ THE CORPUS ALREADY AUTHORS THE GRAPH; NOTHING READS IT.** 651 `heir` obligations · 525
      `RATIFIED` · 418 `DECIDED` · 154 `falsified` · 118 `HYPOTHESIS` · 92 `SUPERSEDED` · ~5,000
      stable-id citations against only **154** whole-document citations (**~35:1** sub-document) ·
      a hand-applied 4-field schema over 29 stub nodes · and an **append-only node-versioning
      idiom converged on independently in FOUR files and never named** (`### 29 (original)`;
      inline strikethrough + dated blockquote; `🔴 SUPERSEDED <date> (user)`). **The authoring
      problem is solved. Extraction and discharge are untouched.**
    - **⚠ NEW STRUCTURAL TYPE — THE ONE-DIRECTIONAL EDGE. Two live instances, both read-first.**
      (i) `north-star.md` § Deviations #2 declares the capability-tiering in three *named* sibling
      sections *"EXPLICITLY NOT THE MODEL"* — and § Passes still reads *"trusted … first-party"*,
      unstruck and unmarked. (ii) **`corrections #12` declares itself *"the pointer"* for
      `S10-results.md`'s ~2×-wrong cost table — the number a user ratified a ship decision on —
      and `S10` holds no reference to it**, while `CLAUDE.md` read-first item 5 sends every session
      to the spikes saying *"measured numbers, don't re-guess them."* **A one-directional pointer
      is not a pointer; it is a note to whoever already found the answer** — and it is the
      mechanical cause of the *chains of authority* problem, since a chain cannot be walked from
      the stale end if the stale end holds no link. *Also a live POLICY COLLISION: corrections
      treats a spike as immutable testimony, read-first treats it as live authority — both
      reasonable, in different files, never reconciled.*
    - **⚠ NO ENUMERATION IS CHECKED FOR COMPLETENESS — three instances, three tiers.**
      `stubs.md` #29's four post-incision phases (one **impossible**, a fifth — **isostasy** —
      **missing**) · `north-star.md`'s two-column core/plugin boundary (**an entire tier absent**,
      *"never decided, arrived at by default"*) · a pass's `reads` set (`Exposed` **declared and
      never read**; `current_chapter()` **read and never declared** — *"a project-wide idiom, not
      an exception"*). **All three found by a human noticing. The asymmetry is damning: the CODE
      layer already has this control** — `build_checked` refuses to build a world if a class a pass
      selects from has zero members, which `north-star.md` § *Validation by construction* calls the
      entire meaning of the phrase — **and it has never been applied to the documents that specify
      the code.**
    - **OBLIGATION IS AT LEAST SEVEN KINDS, and exactly one has a ledger with a denominator:**
      heir · **indictment** (`### What this indicts today` → `DeepAxis` still present) ·
      **ratified-but-unbuilt** (RATE) · **co-retirement** (*"an heir that lands one without the
      other is a world-scale defect"*) · **guard against the obvious fix** (*"before fixing
      nearest→bilinear, read `field.rs:374-383`"*) · **void heir** (stub #1's heir is the social
      sim, ruled NOTHING) · **negative decision** (no representation at all — #35). The one that
      works: `spines.md` S-5's *"34 seams inventoried; **5 converted**."*
    - **C3 IS PROVEN HERE SIX TIMES, and the corpus names it *"structural, not disciplinary"***
      (S-4): `Option<f64>` · `Option<fn>` · `Identity::Unrecorded` · `Domain`/`Draws` ·
      **`CoarseField<T>`** (raw read does not type-check, `compile_fail` doctest) ·
      **`EdgeId::declared`** (an undeclared transition is *unnameable*). **The last is the closest
      in-repo precedent to anything a knowledge layer needs** — we already built *"you cannot
      assert an undeclared relation"*, for material transitions and nothing else.
    - **THE DOCTRINE GOVERNING THIS THREAD IS ALREADY RATIFIED:** `session-workflow` § Seam-first
      #6 — *"**do not build the general mechanism first**… a registry designed before its callers
      exist is the same mistake in a new coat."* **So the ontology may not be designed up front.**
      *(This bullet also cited `stubs.md:22`'s "general registry… four conversions" — **withdrawn
      2026-07-28 (user)**: nobody ever proposed a registry, and a provider seam's success condition
      is that it **disappears**. It never governed this thread. **One pillar removed; the
      seam-first rule stands and is sufficient.**)*
    - **Falsifiers run:** T4 **falsified**; T2 **sharpened, not falsified**. **Still owed:** the
      65 re-coded at § granularity, and a **second reader** re-running the 3.5c coding rule — it is
      single-coded by the assistant whose errors most of the entries record.


- **🔴🔴🔴 THE PASS ARCHITECTURE — AUTHORED ORDER, OPEN VOCABULARY, AND THE RATE AXIS BUILT**
  (**DECIDED 2026-07-26, user**; record of decision `ARCHITECTURE.md` § *The engine is
  plugin-agnostic, and pass ORDER is authored*; corrections **#65**; the user's original
  shape `ideas.md` § *Pass cadence*, 2026-07-23).
  - **WHAT.** Three changes, one arc. **(1) ORDER becomes data on the world**, chosen per
    world beside seed and epoch count; `{reads, writes}` become the **validator**, not the
    generator. **(2) The resource vocabulary opens** — `DeepAxis` retires as a closed
    engine-owned enum; a pack declares its own ids (the kernel is *already* generic over
    `Axis: Copy + Ord`, so the enum is a caller's choice, not a kernel constraint).
    **(3) RATE gets built** — per-pass phase length, sub-turns per epoch, and a real `dt`
    that scales transformations ~~(`dt` is pinned to `1.0` today and nothing scales by it)~~.
    **✅ (3) SHIPPED 2026-07-29, journal/0123** — see the FIRST SLICE block below. **(1) and (2)
    are still owed and this arc is still open.**
  - **WHY.** *"Our passes are PLUGINS… THE ENGINE MUST BE MOD/PLUGIN AGNOSTIC, FULLSTOP,
    EMPHATICALLY"* (user). An engine cannot derive a third party's intended order; requiring
    it to try forces the order into the declarations, and encoding it in engine-owned
    resource names **is** how `DeepAxis` came to hold the default pack's pass roster. The
    seven revision tokens are the hand-declared canonical order wearing a derivation's
    clothes — journal/0090 says so in its own summary, which is corrections #65.
  - **UNIFIES.** The north-star's crossing constraint (*opaque ids*) · the genesis notebook
    § 2.2 **term-keyed edges** (*adding a material never changes any pass's read-set* — the
    same openness on the material axis) · fields-as-plugins (`dc:field/temperature` and
    `dc:field/head` are already declared passes; the **vocabulary** is what is still closed)
    · and the erosion blocker below, which is what RATE's absence produces.
  - **🔑 FIRST SLICE — RATE. ✅ SHIPPED 2026-07-29 (journal/0123).** *The arc stays OPEN — see
    the continuation slot below; three of four parts are still owed.*
    - **WHAT LANDED.** `crates/dc-worldgen/src/deeptime/cadence.rs` (new): `Cadence { period,
      sub_turns }`, both `NonZeroU32` so a zero cadence is **inexpressible** rather than
      validated, plus `dt = period / sub_turns` — epochs of world time per turn. `CadenceTable`
      is the **authored data**: keyed by pass id, applied over each pass's *declared default* by
      `runner::deep_passes_with`, reaching a world through
      `deeptime::run_cells_with_cadence` / `build_field_cfg_cadence`. The runner takes the
      declared **sub-turns** with the cell state carried between them (the sketch's *"a phase is
      handed the cell state at its start"*). `dt` is **live** in the rate-shaped passes
      converted so far — hillslope creep, uplift + its ledger, crustal thickening, and
      inventory weathering, which already read it.
    - **THE ACCEPTANCE TEST WAS A HASH COMPARISON AND IT CAME BACK IDENTICAL.**
      `crates/dc-worldgen/tests/rate_axis.rs`: an empty cadence table reproduces
      `GOLDEN_SURFACE 0x15A6_B756_7A84_29FB` and `GOLDEN_RECORD 0x820B_A198_49DD_234A`, through
      the same distillation path the goldens were captured through. **And the mirror half**,
      which is the one that is easy to skip: an authored cadence *moves the world* (creep
      sub-turned ×2; the eolian agent at period 3), still closes `Δ(ΣR+ΣH) = uplift + biotic`,
      and leaves the un-authored forcing pass's integrated time **bit-identical** — a table that
      changed nothing would have satisfied the golden test perfectly and been a decoration.
    - **STILL OWED, and deliberately NOT done here:** stream transport, bedrock weathering, and
      the wind/wave agents still assume `dt = 1.0` in their magnitudes. Each needs a **modelling**
      call rather than a mechanical one — bedrock weathering is an exponential approach, so its
      honest form is `1 − exp(−k·dt)` and not `k·dt` — and choosing silently re-tunes a constant
      the `EROSION_CALIBRATION` re-pick is about to pick against the literature. **Sequenced
      WITH that re-pick** (`dependency-graph.md` § 4 item 6, P2).
    - **THREE PASSES HONESTLY IGNORE `dt`, and the axis is what made that visible:** the climate
      march, the geotherm and the head field are **relaxations toward an equilibrium set by the
      current state**, not rates integrated over an interval. Their coarse `period` says *when to
      resample*; there is nothing for a duration to scale. Recorded because "pass X does not use
      `dt`" now reads as a claim rather than an omission.
    - **~~⚠ ONE FORK THE DOCS DO NOT SETTLE — NEEDS RATIFICATION.~~ ✅ RATIFIED AND BUILT
      2026-07-29 (journal/0124) — see continuation slot (f) below.** *"A coarse-rate pass does
      not fire at epoch 0"* was a property of the **runner**, not of any declaration. It was
      written for the three passes seeded before the loop (`climate`, `geotherm`, `head`), where
      firing at epoch 0 would redo the seed; once a world could author a coarse period onto
      **any** pass it silently also said *"and your re-rated erosion pass does not run in epoch
      0"* — which nobody decided. **The `Schedule` sum type replaced it and the skip rule is
      deleted.** *The guess in this entry was wrong in an instructive way: a declared `seeded`
      flag beside the cadence cannot express `run-once-only`, which is why the ratified answer
      is a sum type and not a bool.*
    - **⚠ EXTRACTION TAKEN AND MEASURED INSUFFICIENT — module split still a user call.**
      ~~1,694 lines, not taken~~ the history extraction landed (`02aa81a`) and the file is
      **1,908 lines today** — the commentary was not the weight; **927 CODE lines** remain,
      so the remaining lever is the module split the close block already files as waiting on
      the user. *(Staleness sweep 2026-07-29 F9 — both the number and the disposition had
      moved.)*

    - **✅ RATE IS NOT EXPANDED — scope-expansion flag WITHDRAWN 2026-07-29, same day, by the
      user.** For a few hours this entry said the stand-in widened RATE from *authored cadence*
      to *derived stability substepping*. The user rejected the widening rather than accept it
      — *"I really hate to make RATE more complex now. Couldn't substepping be solved within
      the field instead, where it takes `dt` from outside and calcs its own internal multiplier
      to stay within bounds?"* **Yes.** Stability substepping now belongs to the **S-10
      field-solver primitive** (`spines.md` § S-10; `stubs.md` § 30's heir re-pointed there).
      **RATE stays exactly as ratified 2026-07-24: authored cadence and a real `dt`.**
      *Reason: only the kernel knows its own stability constant — it is a property of the
      discretisation. In RATE, every plugin author inherits a von Neumann analysis as a
      prerequisite for writing a diffusion pass; in the kernel, the unsafe call is
      inexpressible.*
    - **RATE still supplies the `dt` the kernel sub-divides**, so this slice is unchanged in
      substance and smaller in scope than it was this morning. **✅ And that is exactly how it
      was built** — `Erosion::diffuse` now reads `let rate = cfg.diffusion * dt;` (authored, from
      outside) four lines above `let diff_sub = rate / f64::from(n_sub);` (derived, inside). The
      ruling turned out to cost one line and to name the E4 extraction target precisely.
    - **WHY IT NOW LEADS.** The shipped world sat **2.1× past its own stability bound** for
      weeks with every golden green, and the only thing that caught it was one author doing
      the analysis once, in one pass. ~~**Every future field pass that diffuses anything has
      the same trap and no defence.**~~ **The defence EXISTS since 2026-08-03 (stamped by
      staleness F11; `stubs.md` § 30 is FULLY DISCHARGED): E4-1 extracted
      `dc-core::field::FieldKernel` — the kernel owns its own bound and sub-cycle, and the
      unsafe call is inexpressible** (journal/0150). That is the engine-shaped hole filled;
      `spines.md` § S-10
      names the shape it belongs to — with the sub-cycle explicitly excluded from the spine
      *because* it belongs to the kernel.
  - **CONTINUATION SLOT** (this is a slice OF *"passes are plugins and the engine is
    agnostic"*): after RATE — **(b)** authored order + the validator (and with it the
    retirement of the revision chain); **(c)** the open resource vocabulary, `DeepAxis`
    deleted; **(d)** epochs declared with pass members + chapter count **or a terminating
    condition** (the sketch's other half, never built); **(e)** the vocabulary split
    question — whether schedule coordinates and world resources need to be different
    *kinds*, or whether authored order dissolves the distinction entirely; **(f) ✅ RATIFIED
    2026-07-29 (user) — ✅ BUILT 2026-07-29 (journal/0124): the `Schedule` sum type**
    (`Seed`/`Step`/`SeedAndStep`; `ARCHITECTURE.md` § Schedule, `deeptime::schedule`). Heir
    of `Seed`: (d)'s setup epoch, named in the variant's doc comment. **Do not close the arc
    when RATE lands.**
    - **THE AUDIT'S VERDICT: ALL THREE PRE-LOOP INCUMBENTS WERE `Step`, AND ALL THREE
      PRE-LOOP BLOCKS ARE GONE.** The discriminator that decided it, once the skip rule was
      deleted: *a seed is an initial condition only if something OBSERVES it before the pass
      itself first steps* — and under "epoch 0 fires for everyone" each of the three first
      steps inside epoch 0, ahead of its own only reader. `climate` and `geotherm` were
      *literally the same call* pre-loop and in-loop; `head`'s seed was a **degraded copy** of
      its step, relaxed over a bare surface because no drainage had run yet. **The seeds were
      repair for the skip, not initial conditions**, and they died with it.
    - **HASHES: exactly one artifact moved, and it is a fix.** `GOLDEN_SURFACE` /
      `GOLDEN_RECORD` held (`providers_golden.rs`, `rate_axis.rs` green), and `grid.precip`,
      `grid.geotherm`, `grid.head` are bit-identical. The **flow record** moved: on the golden
      fixture 315,320 → 314,070 entries, the whole −1,250 in the **vertical** family (24,668 →
      23,418), lateral/boundary/divergent/convergent bit-identical. Chapter 0's vertical faces
      used to be integrated from a head field solved on a landscape with **no drainage at
      all**; they now come from the real epoch-0 solve.
    - **⚠ AND THE FLOW RECORD HAD NO GOLDEN — a 834-test gate could not see it move.** It is
      a pure sidecar to both existing goldens, so nothing guarded the largest thing the ritual
      keeps; the move had to be measured with a throwaway harness. **`GOLDEN_FLUX` added in
      the same slice** (`tests/flux_record.rs`), with the move recorded on the constant.
      *An artifact with no tripwire cannot have an "authorized move", because nobody can see
      it move.*
    - **The gate invariant landed as a tiling claim, which is stronger than the sentence that
      ratified it.** `tests/schedule_axis.rs`: a firing at epoch `e` opens a phase covering
      `[e, e+period)`, firings land on the multiples of `period` **from zero**, so the phases
      tile `[0, iterations)` exactly once — `Schedule::integrated_dt(N) == N` for every pass,
      every period, every run length, clipped only where the world ends mid-phase. Σ`dt` = N
      exactly is the corollary when the period divides the run (it does: 20 and 40 into 200).
      **The old skip rule lost exactly one period** — 180 epochs of world time against its
      neighbours' 200. Asserted at zero world-build cost: when a pass runs is a property of
      the roster, not the terrain.
    - **`Seed` / `SeedAndStep` ship with no production declarer** — `spines.md` § 3 row. They
      are executed and tested (`DeepSchedule::plan(None)`), not decorative. The deep-time
      roster's genuine setup work — the biotic layer's identity planes, the tectonic chapter
      table — is *not pass-shaped yet*; that conversion is (d).
  - **⚠ THE PROCESS FAILURE IS PART OF THE RECORD.** The user's sketch was **reconciled, not
    contested** — its ORDER half died inside an implementation slice, in one clause of a doc
    the user does not read. New CLAUDE.md rule: *a user-originated design may not be
    superseded by an implementation slice.*


- **✅ THE HILLSLOPE OPERATOR IS FIXED — SHIPPED 2026-07-29 (journal/0122).** The blocker
  — **archived 2026-07-29 to [`ROADMAP-history.md`](ROADMAP-history.md)** — is discharged; it is kept live only because **one inference in it is falsified and one
  new item falls out of it**, and both need reading before anyone touches the calibration.
  - **THE MECHANISM, isolated rather than argued.** The pass is an explicit four-neighbour
    Laplacian; its grid-scale mode decays only below a per-edge coefficient of **1/8**
    (`g = 1 − 8a`). The world's peak effective coefficient is **0.261 shipped** and **12.60
    calibrated** — 2.1× and **100.8×** past that bound. The flux limiter then caps the export
    at the cell's whole inventory, which converts the divergence into an exactly
    amplitude-preserving **period-2 limit cycle**. Isolated on a bare grid
    (`erosion.rs::hillslope_operator_tests`, the two populations swap to the bit) and
    confirmed on the world by a **temporal** discriminator with opposite predictions:
    `corr(conc_h[N], conc_h[N+1])` = **−0.90 calibrated** against **+0.76 shipped**.
    journal/0116's period-2 hypothesis was right as written.
  - **⚠ CORRECTIONS #72 — the (b) block of the archived conveyor entry (`ROADMAP-history.md`) is WRONG and #63 (ii) with it.** It *was* a
    stability limit. journal/0116's 4× refinement took the calibrated arm from 100.8× past the
    bound to **25.2× past it**, so its null was a statement about the number 4. Every
    measurement in that entry stands; one inference does not. **Do not cite "not a time-step
    limit" from that archived entry without reading #72.**
  - **THE FIX.** `Erosion::diffuse` sub-cycles each epoch into
    `ceil(max_cell eff_diff / CREEP_MAX_EDGE_COEFF)` steps. `n = 1` is the old operator bit for
    bit. Not implicit (the `h ≥ 0` obstacle plus exact mass through an iterative solve costs
    more than it buys at ~5 cells of diffusion length) and **not a cap** (capping would
    reinstate stubs #27's conveyor). Measured on production-Medium:

    | arm | sub | conc(h) rms | ACF(1) x/y | surf conc rms | surf ACF(1) | hollows >10 m | deepest | mean h | gen |
    |---|---|---|---|---|---|---|---|---|---|
    | calibrated, before | 1 | 62.42 | −0.821 / −0.853 | **40.42** | −0.867 / −0.912 | **818** | 112.8 m | 43.72 | 34.7 s |
    | calibrated, after | 100 | **2.66** | **+0.001 / +0.887** | **0.30** | **+0.185 / +0.105** | **12** | **15.8 m** | 1.40 | 238.9 s |
    | shipped, before | 1 | 3.42 | −0.102 / −0.182 | 0.21 | +0.380 / +0.269 | 0 | 0.0 m | 4.57 | 33.9 s |
    | shipped, after | 2 | 2.87 | −0.098 / −0.142 | 0.21 | +0.414 / +0.317 | 0 | 0.0 m | 3.91 | 37.6 s |

  - **THE SHIPPED GOLDENS MOVED, and only a little.** `diffusion = 0.12` is inside the bound,
    but `eff_diff` folds in the lithology's creep susceptibility and peat is the softest thing
    in the world, so the shipped peak is 0.261 and the shipped world takes **two** sub-steps:
    mean regolith 4.57 → 3.91 m, relief −0.9 m, mean surface −0.2 m, closed hollows 0 → 0.
    The unbounded operator stays reachable and hashed (`DeepConfig::creep_substep`,
    `GOLDEN_SURFACE_UNBOUNDED_CREEP`, `tests/creep_operator.rs`).
  - **⚠ THE HEIR, AND IT IS USER-OWNED: `EROSION_CALIBRATION = 45` WAS FITTED AGAINST THE
    BROKEN OPERATOR.** journal/0114 measured that raising `diffusion` bought almost nothing
    (100× on transport → 1.6×) and concluded the pass was a one-cell-per-epoch conveyor — true
    *of the capped operator*. With the cap gone the same multiplier strips the world: **mean
    regolith 1.40 m**, below the *shipped* world's 4.57 m, and 16,347 cells carrying >1 m of
    closed-hollow fill on a surface whose concavity rms is 0.30 m — broad shallow basins on
    scraped bedrock, not pits. **A constant fitted against a broken operator does not survive
    fixing the operator.** `calibrated_rates` still ships **false**; it is no longer blocked on
    a defect, it is blocked on a number, and journal/0122's ladder is the input.
  - **THE GUARDS ARE RE-ASSERTED (corrections #62, in scope and done).**
    `no_interior_cell_is_cut_below_all_of_its_neighbours` is kept and is now the first of
    three: `no_interior_cell_carries_a_closed_hollow` (fill depth — **rises** as the defect
    generalises, bar derived from the clamp's own statement) and
    `the_surface_carries_no_grid_scale_oscillation` (concavity ACF, which cannot saturate at
    all, bounded at −0.5 from the closed-form references). The four census primitives moved to
    `deeptime::census`, closing journal/0116's open extraction.


- **🔴 RE-PICK `EROSION_CALIBRATION` AGAINST THE FIXED OPERATOR — and flip the flag**
  (sequenced 2026-07-29, user: *"yes"*; journal/0122). **The constant was fitted to a broken
  solve and does not survive its repair.**
  - **⚠ ENTRY OVERTAKEN — stamped 2026-08-03 (staleness F4, second consecutive sweep):
    the work has moved three stages past this body.** The literature pass shipped
    (`ee9fb94`); the **measurement runs are DONE 2026-08-02**
    (`docs/audits/2026-08-02-p2-measurement-runs.md`): target **doubly confirmed**
    (2.63 derived ↔ 2.653 from the world's own Airy decomposition), measured
    D3(M) ≈ 0.0070·M puts the target at **M ≈ 375–380** — the derivation's [60, 240]
    bracket was low 1.6×. **The flip is blocked on three USER calls** (runs doc § 4):
    the M re-pick · ~~the pits-bar conflict~~ **DISSOLVED 2026-08-04 (corrections
    #98, user: "the dimple test does not make sense"): the hollows>10 m = 0 guard
    carried a never-decided premise and is retired as a blocker — replacement is
    the process-ownership criterion (a closed depression is legitimate iff a
    process owns it; the pit safari becomes a provenance check)** · the
    gen-time/register trade —
    the last **mostly dissolved by E4** (the 38-min gen time is explicit-stability
    CFL tax; the implicit kernel is the ruled answer-shape, and **E4-3 flips adoption
    WITH this re-pick so goldens move once**, dependency-graph E4/P2 rows). **Net:
    the flip is blocked on the M re-pick alone, sequenced with E4-3.**
  - **What moved.** `EROSION_CALIBRATION = 45` was derived on the capped operator. With the
    cap gone, 45× strips the world to **1.40 m** mean regolith — *below* the shipped world's
    4.57 m — and the ladder **inverts journal/0114's headline finding**: cover now **thins**
    with the multiplier (3.91 → 4.40 → 3.02 → 2.18 → 1.65 → 1.40) where it used to thicken
    (4.6 → … → 782 m). **Transport now outruns supply**; the joint balance point has moved.
  - **How to pick it — the ONLY legal method.** Against the **published literature**, not
    against a look. `CLAUDE.md` § *a closed system cannot detect its own scale error*: a
    constant tuned until an output looks right is a number pretending to be a mechanism; one
    derived so a measured quantity lands in a published band is evidence. The target is the
    craton denudation band journal/0111 established. **Do not re-fit to internal consistency
    — every internal instrument was green at 1000× wrong.**
  - **✅ THE PRE-FLIP BLOCKER IS DISSOLVED (user, 2026-07-29).** The three excluded agent
    magnitudes (`wave_erosion`, `eolian_deflation`, `frost_weathering_gain`) **scale with
    everything else now** — *"I do not care if frost/wind magnitudes increase 45×… I do not
    care about my previous ratification on looks there."* **Byte-identicality on the
    calibrated arm is explicitly NOT wanted; do not spend a slice on it.** `stubs.md` § 28
    dissolved, `earth-processes.md` § 4 stamped.
  - **Acceptance** pairs an aggregate with a **neighbour-relative** measure (corrections #61)
    and re-asserts the three pit guards (corrections #62, discharged by journal/0122). The
    ladder from journal/0122 is handed forward as **input, not as a recommendation** — the
    agent that measured it deliberately did not act on it, which was correct.


- **🔴🔴 CALIBRATE THE DEEP-TIME CLOCK — the largest measured defect on the board**
  **(⚠ PARTLY BUILT 2026-07-26, journal/0114 — see `ROADMAP-history.md` § Shipped. It is BUILT, MEASURED and
  DELIBERATELY OFF, blocked on stubs #29 — **discharged 2026-07-29 by journal/0122; that entry is now in `ROADMAP-history.md`**. ~~The band is NOT reachable at any multiplier;
  read stubs #27 before assuming a bigger number fixes it.~~ **⚠ CONTRADICTED 2026-07-29
  (staleness sweep F1): journal/0122 inverted 0114's ladder on the default-on operator —
  "no safe multiplier" became "there are five." The evidence base for the struck claim is
  VOIDED; reachability needs RE-MEASUREMENT under the fixed operator, and neither the old
  claim nor its inverse should be asserted until then.)**
  (journal/0111, corrections #56, stubs #24, 2026-07-26). **User-owned and appearance-class:
  the multipliers are the user's to bless. Do NOT let an agent pick them by taste — the target
  is a PUBLISHED BAND, and that is what makes this a derivation rather than a tuning.**
  - **THE MEASUREMENT.** The shipped world denudes at **0.0110 m/Myr** catchment-averaged —
    **9× slower than the slowest landscape ever measured on Earth** (McMurdo Dry Valleys /
    hyperarid Atacama, 0.1–1), **493× below the global ¹⁰Be outcrop median** (5.4), stripping
    **5.48 m** over the ratified 500 Myr where a real craton strips **5–10 km**. Denudation is
    **2.7 % of uplift**; the land *builds* 204 m over the run. **The single most active cell of
    44,264 (0.134 m/Myr) is still below the global floor** — so this is not "quiet interior,
    active margins"; there are no active margins.
  - **THE SHAPE IS RIGHT AND THE CLOCK IS WRONG**, and that distinction decides the repair. A
    weathering-limited landscape routed by creep, with minor rivers, regolith armouring its own
    weathering front and erosion mildly concentrated on steep ground, is a **textbook low-relief
    craton**. Every qualitative statement journal/0110 made survives. **Only the magnitude is
    wrong** — this is a calibration, not an architecture.
  - **🔑 NEITHER LEVER PAYS ALONE — this is the finding that makes the slice tractable.**
    Measured by sensitivity sweep, production untouched: **100× erosion budget → 1.4×**;
    **10× creep → 1.7×**; **together → 132×**, which is **59× more than their product**. The
    coupling is the cover taper `exp(−H/H*)`, `H* = 3 m`: raise **supply** alone and the
    regolith you make **shields the rock that made it** (supply-limited → transport-limited,
    ratio 0.98 → 0.36); raise **transport** alone and there is nothing to carry. **So it must
    be a JOINT calibration of supply AND transport, or it will read as "the knob does
    nothing."**
  - **⚠ THE KNOB THAT EXISTS STRUCTURALLY CANNOT DO IT (stubs #24).**
    `DeepOverrides::erosion_budget` is documented as *"**the** terrain (erosion) amplitude"* but
    scales `weathering` / `k_transport` / `k_bedrock` and **NOT `diffusion`** — the process doing
    **96 % of the eroding**. Fixing its scope is part of this slice.
  - **EXPORT DECOMPOSITION, which re-ranks the whole material-behavior arc:** **96.0 % hillslope
    creep across the shoreline · 3.7 % wave · 0.3 % eolian · 0.02 % FLUVIAL.** journal/0110 put
    rivers at 0.109 % of *routing*; at the shoreline they are **0.02 % of yield**.
  - **ACCEPTANCE INSTRUMENT EXISTS:** `examples/denudation_probe.rs` with its cited literature
    table. **Target: the 1–10 m/Myr stable-craton band.** Behind `DeepConfig::denudation_ledger`,
    off in production, gate-asserted bit-inert.
  - **This moves EVERY golden** — same event class as the erodibility / biotic / tectonic /
    full-agent flips. Announced, never silent.
  - **Sequencing:** must land **after** material-aware creep (journal/0112) and hybrid-`p`
    (journal/0113), because it calibrates whatever the final physics is. Hybrid-`p` is only
    weakly coupled (it moves 0.02 % of the yield), but creep is 96 % and is being rewritten.
  - **It also DEMOTES the rest of the material-behavior arc's urgency.** Making creep
    material-aware is worth doing — but *a landscape moving a thousandth of the right amount of
    sediment will still not show a facies gradient.* **The calibration should land first, or at
    least alongside.**
  - **⏳ The 500 Myr / 200 epochs = 2.5 Myr-per-iteration register is STIPULATED, never fitted**
    (two independent corpus sources agree). `earth-processes.md` § 3e's oldest owed item —
    *"calibrate iteration↔Myr against a real orogen"* — is now the **named heir of a measured
    10³× error**, not a nicety. **No physical rate in this engine had ever been checked against
    this clock.** The probe therefore also prints every rate per epoch, so a future session can
    re-anchor either the clock or the rates.

- **🟡 COARSEFIELD ADOPTION — the ratified cure that lost its entry** (revived 2026-07-29 by
  user ruling: *"i think we revive it too"*). **This is the fix for U22 (the cake law) and U3
  (the per-chunk palette checkerboard), and it had no owner from 2026-07-24 to 2026-07-29.**
  - **✅ THE FAR HALF SHIPPED 2026-07-29 — E5 member #0, first build slice (journal/0125).**
    `collapse.rs::surface_class` now samples a `CoarseField<ShareVec<6>>` window through
    `sample_dithered`; `DitherSource` has its first production impl (`draws.rs::Coherent`);
    `draw_class` and its unbiasedness test retired into `ShareVec::draw`; `DeepField` hands out
    its own `Registration` (**in voxels** — the MM-6 units trap) and a producer-side
    `record_at_cell`. **U22 is discharged**, with a world-scale acceptance test beside
    `coarse.rs`'s synthetic one. Design + rulings:
    `docs/audits/2026-07-29-member0-coarsefield-design.md`.
  - **⚠ A MEASURED CONSEQUENCE THE USER MAY WANT TO WEIGH — the membership dither is a
    CELL-WIDE BLEND, not a perimeter treatment.** `coarse.rs`'s doc comment says *"away
    from a boundary one weight ≈ 1, so it reduces to the containing cell's own shares"*,
    which is true only **at the cell centre**. Averaged over a cell the weight on the home
    cell is `4·(∫₀^½(1−x)dx)² = 9/16`, so **~44 % of far voxels draw a NEIGHBOURING deep
    cell's shares, everywhere** — not just near a 460 m frontier. Measured from the side:
    near/far agreement fell **0.8225 → 0.7922** and tripped its floor, which is the whole
    44 % effect. *Defensible — real facies contacts are gradational and the record's hard
    cell edges are a resolution artifact, not a fact about the ground — but it is a bigger
    claim than "the frontiers got softer" and it was not what the slice's brief described.*
    The floor was **re-derived from the mechanism, not lowered to fit**: 0.80 → 0.70, where
    0.611 is what independent neighbours would give and 0.7922 back-solves to a
    neighbour-cell coincidence of 0.753 (journal/0125).
  - **⚠ WHAT THE FAR HALF DID *NOT* FIX, stated so it is not inherited as done:**
    (1) **U3 / the near site** — untouched, and the design pass found its plan does not survive
    contact with the record: it needs **MM-1** (the membership dither, separately callable and
    generic in `T`) and **MM-3** (the working sub-cell state, which has no declared type), and
    every golden moves when it lands. (2) **The coherent source's majority-amplification bias**
    (corrections #39) — its named heir was `summarize`, which the same day was **ruled to the
    octree node contract** (MM-4), so the bias now rides the shipped far field with **no heir
    inside this tier**. (3) **The member-dither guillotine** (`geology.rs::dithered_member`
    under chunk-centre formation context) — U22's named sibling, different site, still
    unexamined. (4) **`coarse_surface`'s `level_stride` footprint** — one `Block` painted
    across 1.8–14.4 m, the smaller square left standing; owned by MM-4.
  - **THE NEAR PATH IS NOT SUFFICIENT ON ITS OWN, and that is settled, not suspected.** See
    § Observed *"The U3 checkerboard's dominant signal is the 28.8 m MEMBER STEPPING, settled
    2026-07-24"*: the dominant signal is the **single-octave member dither**, so a perfect
    near-path fix leaves the squares on screen. **The octaves `DitherSource` is a co-requisite
    of U3, not a follow-on** — and its socket now exists and has a worked example in it
    (`draws.rs::Coherent`), which is the cheapest this co-requisite will ever be. *Do not
    re-derive this as "unprobed"; the member-#0 design pass did, and only the user's memory
    caught it.* Residue: a confirm re-shoot at U3's reference pose rides the next appearance
    walk.
  - **✅ THE CO-REQUISITE SHIPPED 2026-07-29 — the octaves `DitherSource` (journal/0128),
    slot (a) of the continuation pair.** `draws.rs::Octaves`: a **normal-score-transformed
    fBm** over a **pairwise-coprime prime** stride ladder (509 … 13 voxels; the head, 509
    voxels = 458 m, is the deep cell itself). Two things no prior plan had, both found by
    doing the arithmetic before writing the code:
    - **The obvious fBm SUM is unusable for a source that feeds an inverse-CDF draw.**
      Normalised, six octaves of bilinear uniforms is a bell with **σ ≈ 0.085 around ½** —
      any class whose CDF band lies outside the middle half of the interval would **never be
      drawn at all**. Adding octaves makes it monotonically worse. So the corners are drawn
      **normal**, not uniform, and the output is `Φ(S/σ)` with `σ²` a closed form of the
      interpolation weights: a bilinear blend of normals *is* normal, so the marginal is
      **uniform by construction**. That is the geostatisticians' **truncated-Gaussian facies
      simulation**, arrived at from the wrong end.
    - **Prime strides, not dyadic.** A dyadic ladder from 512 keeps a 32-voxel kink lattice
      (every coarse octave's kinks land on the fine ones). Measured: on/off-lattice curvature
      ratio at the chunk scale **1.36** for `Octaves` against **1.0e14** for the shipped
      `Coherent` — because a bilinear field is *linear inside a cell*, so **all** of its
      curvature sits on one grid. That 10¹⁴ **is** the 28.8 m square-edge signature, and it is
      now a gate test.
    - **Consequence for corrections #39, which had lost its heir:** `Octaves` renders a 0.6/0.4
      share vector at **0.6053** where `Coherent` renders it at **0.6829**. The majority
      amplification is a property of `Coherent`, **not of coherence** — so the near path gets
      an unbiased coherent source, and #39's "CDF-corrected source" heir exists for the far
      register whenever its own semantics are re-decided.
    - **Persistence 0.6 is derived, not tuned:** `H = 0.737`, and 2D-fBm level sets (which is
      what a contact is) have fractal dimension `2 − H ≈ 1.26` — inside the published 1.2–1.3
      band for traced geological boundaries. A literature anchor, named as one.
    - **It shipped with NO visible outcome and was labelled so.** Its consumer is the
      near-path restructure (journal/0129), same worktree, same evening — the pair was kept a
      pair precisely so this did not become a second `CoarseField` sitting uncalled for seven
      days.
  - **✅ AND ITS ADOPTION AT THE NEAR MEMBER DITHER SHIPPED 2026-07-29 (journal/0129),
    WITH MM-1 — but slot (b) is only PART done and the rest is named here.**
    - **Shipped:** `geology::interp_select_draw` retired into `selection_field` =
      `Octaves`; the member dither *and* the record's own representative pick now read
      ONE field at ONE voxel address (their agreement at the centre column went from
      approximate — `fx = 0.5` vs `0.515` — to exact). **MM-1 discharged:**
      `CoarseField::sample_source_cell`, with `sample_dithered` **rewritten in terms of
      it**, so there is one membership dither in the tree and the new call has a
      production caller the hour it landed. Its law had never been tested and now is
      (home cell 9/16 of its own cell exactly; at a corner all four answer ¼ and the five
      outside the stencil never; white source on purpose — a coherent source's finite
      window measures its own autocorrelation, not the draw).
    - **Perf, because chunk load is a hot path:** the dither was **hoisted from per-VOXEL
      to per-(voxel column, event)** — it was always a pure function of that pair and was
      being called up to 2 × 32 768 times per chunk. That is what pays for the octaves' 6×
      hashes: measured `contents_contract` (240 chunk generations) **103.72 s → 102.43 s,
      −1.2 %, no measurable cost.**
    - **Goldens: 2 of `contents_contract`'s 3 triples moved, mechanism recorded beside the
      constants**; the Small row is byte-identical for the reason its header already states
      (its sampled chunks carry no strata record). No other workspace golden moved.
    - ~~**⚠ NOT SHIPPED — the near-path RECORD restructure (U3's 460 m half).**~~
      **✅ SHIPPED 2026-08-02/03 as P11 slice 3 (stamped 2026-08-03, staleness F3 —
      the answer lived in the graph while this asking entry stayed open):**
      journal/0145, merge `cd1058a` — `ColumnRec.strata` dissolved into per-column
      `records: Vec<SubCell>`, blast radius re-measured 14 files / ~71 sites before
      the cut, and **MM-3's type shipped WITH its consumer** exactly per the
      withdrawal condition below. *(The membership DITHER that rode it was rejected
      at the walk and superseded by the stratigraphic-correlation design — see that
      audit's § 5 for what survives as substrate.)* Historical body kept: blast
      radius was first measured 13 files / ~40 sites; MM-3 was written and
      deliberately WITHDRAWN pending its consumer (a declared type with no consumer
      is `CoarseField` on 2026-07-22, the exact failure this arc teaches).
    - **⚠ AND THE COST QUESTION THAT SLICE OWES IS WORSE THAN THE DESIGN PASS SAID.**
      *"≤ 9 touched cells, typically 1"* reads the stencil as *"cells the chunk
      overlaps"*. The bilinear stencil is **always 2×2**, so a chunk in a cell's interior
      touches **four** cells, and with 1024 columns drawing, even a weight of 0.001 is
      realised somewhere in the chunk. **Typically 4, up to 9, and ~1 only within half a
      metre of a cell-centre line** — so `run_strata` per chunk goes 1 → ~4. It is the
      same *"one weight ≈ 1 away from a boundary"* misreading that cost journal/0125 a
      failing gate, in its third outfit.
    - **⚠ Still unfixed at the same site, and neither is a consequence of this slice:**
      (a) `dithered_member`'s **formation context is still the chunk's** — the draw is
      multi-scale now but `event.temp_c`/`precip`/`depth_m` were recorded once per chunk at
      the chunk centre, so the *fitness landscape* the draw indexes still steps at 28.8 m
      (U22's named sibling, the member-dither guillotine, unexamined since 2026-07-22);
      (b) the **FAR summary's member dither is still the single octave**, deliberately —
      same defect, other tier, and the far register's semantics were rejected the same
      evening, so changing its appearance would answer a question nobody asked (annotated
      at the call site).
  - **WHAT WAS RATIFIED**, 2026-07-22, in the strongest language in this thread: `CoarseField<T>`
    as a **boundary type** whose only fine accessors are two legal moves (`sample` /
    `sample_dithered`), making the raw per-cell read **inexpressible**. The user: *"we finish
    this today"*; *"there are things i'd rather do but this is foundational."* End state:
    ***"solved by construction, approximately once."***
  - **WHAT HAPPENED.** The extraction **shipped** (journal/0075) and was then never **adopted**.
    journal/0075 § "What was NOT done" assigned adoption to *"those owners"* — **and those owners
    were never named.** Two days later an assistant clause handed apparent ownership of the
    palette-quant fix to the genesis-passes arc (struck above), and the real cure stopped having
    a home. It survives only as a trailing half-sentence at `ROADMAP.md:~179` (*"remaining
    CoarseField migration follows the type freeze"*).
  - **BUILT, AND NOTHING CALLS IT — the whole API** *(historical testimony, true at `ae4bb29`;
    **RESOLVED 2026-07-29, journal/0125** — `sample_dithered`, `sample` and `DitherSource` now
    have production callers, per `spines.md` § 3's CoarseField row; staleness sweep F2)*.
    Verified workspace-wide at `ae4bb29`:
    `sample_dithered`, `sample`, `summarize`, `DitherSource` (trait, **zero production impls**)
    and `CoarseField<T>` itself all had **zero callers outside `dc-core`**. Only `ShareVec<N>`
    landed. ~~`collapse.rs:949` still defers to *"the `CoarseField<T>` extraction (audit Part 2)"*
    as though it were pending~~ *(that comment text is gone from the tree as of 2026-07-29 —
    whether answered or merely deleted was NOT determined; staleness sweep F4)* — **the
    extraction is done; the adoption is half-done (far ✅, near owed).**
  - **⚠ AND IT WAS INVISIBLE TO ALL THREE LOOSE-END LOCI** *(historical; **DISCHARGED** — the
    `spines.md` § 3 row exists, `spines.md:1474`; staleness sweep F3)*. It was in **neither**
    `spines.md` § 3 "Built, and nothing calls it" **nor** `stubs.md` **nor** — until
    then — ROADMAP. `spines.md` § S-4 called it the *"Ratified end-state — **EXTRACTED**"*, which
    reads as **done**. Per read-first item 6 an unlisted loose end is *the defect, not a licence*:
    the largest built-and-unconsumed mechanism in this thread was hidden from the lookup that
    exists to find exactly this.
  - **THE TWO SITES ARE ONE ROOT, AND A FIX TO ONE DOES NOT FIX THE OTHER.** The root is
    `DeepField::record_at_voxel` reading **NEAREST** at the ~460 m deep cell — its own doc comment
    flags it (*"a variable-length unit sequence cannot be interpolated, so the facies story steps
    at the ~460 m deep-cell grid — FLAGGED sampling choice"*, `field.rs:758-772`). Two expressions:
    **U22 / far field** at `collapse.rs::surface_class` (the class draw's *shares* are still
    nearest-per-cell, so minority phases die at the 460 m line), and **U3 / near field** at
    `collapse.rs:1409` *(site RETIRED 2026-08-02/03 — journal/0129's `selection_field` + slice 3's
    per-column records; line number kept as history, staleness F12)* (one point-sample per chunk
    shared across all 1024 columns). The 2026-07-24
    diagnosis audit § 5 already drew this line and nobody has read it since: *"**One mechanism at
    the level of root cause; distinct at the level of site.** A fix targeting one site would not
    automatically fix the other."*
  - **⚠ DO NOT INHERIT journal/0088's HEADLINE.** *"Three or four separate-looking bugs turned out
    to be one"* is **overstated relative to the audit it summarises, published the same day** —
    doc-topology shape 6, a summary outrunning its source, and the probable origin of a week of
    user uncertainty about what they had ratified. The honest count: **U22 + U3 are two sites of
    one root; U4 is a different bug** (a static `REDUCTION_STANDOFF_M` distance gate, geometry half
    shipped in journal/0091, material half still owed); **the 16-voxel member squares are a fourth
    thing**, one stage downstream, that no fix to the root will touch and that cannot touch the root.

- **🔴 REFINEMENT PRIMITIVES — the tier nobody assigned an owner** (opened 2026-07-26 by the
  user's question: *"who owns the **clever** refinement operators for presenting the
  interpolated/upscaled runtime world? how would a mod hand-roll emergent rivers on their own
  via the SDK, **that are visible**?"*). **Read `north-star.md` § "The refinement tier is in
  neither list" first — the hole is recorded there in full.**
  - **THE HOLE — ✅ THE OWNERSHIP HALF IS CLOSED (2026-07-28); the CONTRACT half may still be
    live.** *Marked 2026-07-29 (baseline sweep S5/F3).* `north-star.md` now carries
    **`### ✅ THE REFINEMENT TIER — RESOLVED TO (c), user, 2026-07-28`**, and this document
    records the same resolution twice more — in the § Sequenced entry above and, **twelve lines
    below inside this very bullet**, as *"DIRECTION — DECIDED 2026-07-26 (user)… option (c)."*
    **So the entry recorded the decision and still opened by saying it was never made** — the
    corrections #65 shape at 12 lines instead of 400. Read the rest of this bullet as the
    *record of the hole*, not as an open question:
    ~~`north-star.md`'s core/plugin boundary enumerates both columns and **the
    refinement tier is in neither.** It was never decided; it was arrived at *by default*.~~
    `flow.md` § 4 names it as one of four tiers **and gives it a contract** (*pure fn of
    (record, shared face data, position)*), while the ownership document does not know it
    exists. In code it is `collapse.rs` — **2,415 lines, essentially zero declaration.**
  - **THE CONSEQUENCE.** The engine hardcodes the **vocabulary of expression**. A mod gets
    emergent **valleys** for free (mass moves → heights drop → collapse renders it) but can
    **never** get a visible **channel**, because sub-cell geometry has no declarative route to
    a voxel. And this **separates two nulls we had been treating as one**: 2b's facies null is
    a **magnitude** problem (material variation *does* have a path); the missing channel is a
    **path** problem (no magnitude of flux record can fix it).
  - **DIRECTION — DECIDED 2026-07-26 (user):** *"visible river channels will be the first
    built on refinement primitives, as you suggested"* — i.e. option **(c)**, the split that
    mirrors the 2026-07-23 field-pass move: **refinement PRIMITIVES are core** (sub-cell
    boundary-value solver, position-addressed noise, interval fill), **refinement OPERATORS
    are content**. ~~*The mechanism is not designed; the direction is set.*~~ **✅ The
    mechanism IS designed: `docs/design/refinement.md`, CAUTIOUSLY RATIFIED 2026-07-29**
    *(doc-topology F1 — north-star routed readers here while this line still said
    undesigned)*.
  - **FIRST SLICE — a DESIGN PASS, not code** (user: *"refinement primitives want a design
    pass and then a decomp and port of existing collapse"*). **✅ DONE AND RATIFIED 2026-07-29:
    `docs/design/refinement.md` (CAUTIOUSLY RATIFIED — bones only; members need their own
    passes) answers every question this bullet posed** *(staleness sweep F5 — all four ROADMAP
    mentions of this were previously inside the close block only)*. Field-notebook-first per the
    earth-processes method. It answered: what is the primitive set · what the operator
    contract is in SDK terms (the pure-fn constraint **is** the sandbox contract — one
    constraint, two payoffs) · **invocation granularity** (see the amendment
    below) · and how a **mod** authors a visible channel end-to-end.
    - ~~per-cell or per-chunk, **never** per-voxel — *runtime is sacred*, and seam-first
      already forbids a provider in a hot loop answering a question that does not change
      inside it~~
    - **⚠ AMENDED 2026-07-29 (user) — NARROWED TO ITS OWN RATIONALE, not overruled.** Read the
      struck wording against the reason it gives: it forbids a provider in a hot loop answering
      **a question that does not change inside the voxel**. That is a ban on *redundant*
      invocation, not on per-voxel granularity as such — and the octave dither's answer **does**
      change per voxel, which is its entire purpose. **The stated reason permits exactly what
      the stated rule forbade.** Taken literally the old wording silently forbade the port of
      `sample_dithered` and `interp_select_draw`, both per-voxel-column today, and per-voxel is
      precisely what makes them C0-continuous and square-free.
    - **The rule now: invoke each question at the granularity at which its ANSWER changes.**
      The tiers **layer** (user, 2026-07-29) — the expensive coarse work runs coarse, the cheap
      evaluation runs per-voxel:

      | granularity | what runs there | side |
      |---|---|---|
      | **per-cell** (~460 m) | the facies driver — a smooth field from a field pass | **content** |
      | **spanning** | the octaves — bridging cell → chunk → voxel; this is what an octave decomposition *is* | **engine primitive** |
      | **per-voxel** | the draw (`sample_dithered`) — cheap, and *must* vary here or it reads as a grid | **engine primitive** |

    - **There was never a measurement behind the ban.** The only number nearby is B1's — white
      noise doubled the far-tile mesh (**21.5 → 43.5 MiB**) — and that measures source
      **coherence**, not invocation **granularity**. It is an argument *for* octaves, not
      against per-voxel. *A bound with a derivation is evidence; one chosen until it reads well
      is not* (§ Gates tolerance doctrine, and the closed-system rule).
  - **SECOND SLICE — the DECOMP AND PORT of `collapse.rs`** onto those primitives. Note this
    is also the largest instance of the file-size problem below, so it discharges two chores at
    once.
  - **WHY IT LEADS.** `flow.md` § 4 already says refinement *"is not 'carving a shape' but
    **solving a small boundary-value problem inside a cell**, with face fluxes as Dirichlet
    conditions and the load budget as mass"* — and we now **have** the face fluxes (journal/0096,
    0109). **The primitive the channel needs is the one FLOW spent three slices building.**
    This is also FLOW continuation **(b′)** by another name; the two entries should be read
    together.

- **🔴 REQUIRED CHORE — FILE SIZE IS A CORRECTNESS PROBLEM IN AN AI-NATIVE WORKSPACE**
  (user-directed, 2026-07-26). *"A file of that length is a red flag in an AI-native workspace —
  Claude must grep through, never reading all, **missing context or reading too much irrelevant
  context**. Same issue with ROADMAP and other docs."*
  - **THE REASONING, recorded because the convention must be derived from it, not from taste.**
    A file is a **unit of context**. Past a threshold an agent has only two options and **both
    are lossy**: grep it (and see only what it already knew to look for — the failure mode the
    2026-07-25 close block named, where `DeepField::chapters` sat unlisted through *three*
    spine audits) or read it whole (and burn context on irrelevance, crowding out the files it
    actually needed). **This is the same defect as "the ROADMAP outgrew reading", generalised
    from one doc to the repo.**
  - **THE MECHANISM (~~build it~~ ✅ SHIPPED 2026-07-28 as `scripts/filesize_hook.py`, scope
    corrected by corrections #85 — stamped 2026-08-02, staleness F6): a hook on file write that reminds, with an instruction to
    restructure** — pull separate concerns into separate files. **Do not answer "the tool
    cannot see X" with a rule asking people to remember X** — that was tried for probes and
    **failed in one day** (CLAUDE.md § Gates). A reminder the harness issues is a mechanism; a
    line in a doc is not.
  - **ADOPTION (user's terms): immediate for NEW files; gradual refactor of old work WHEN IT IS
    TOUCHED.** No big-bang rewrite.

- **Finish the draw-domain conversion: the residual hand-rolled sites** (opened 2026-07-25
  by journal/0105, which converted 26 of them and named these). Small, and each is named in code
  so it cannot be lost. **(b) is DONE 2026-07-26** and its rider **re-measured (a)'s blast radius,
  which was overstated — see below**.
  - **(a) `dc-sim/engine.rs`'s region-step and agent-step draws** address `[seed, k, SALT, …]` —
    the sample index sits before the domain, so putting them on `Draws::of` (which fixes the
    domain at slot 2) changes the key. ~~**Still a user-owned appearance slice, but a smaller and
    differently-shaped one than this entry claimed**~~
    **🔴 NO LONGER AN APPEARANCE SLICE AND NO LONGER THE USER'S — corrected 2026-07-29
    (baseline sweep S5/F2).** The whole visible consumer chain below was **deleted 2026-07-28**
    with the bootstrap history content (`journal/0121`, corrections #66, −713 Rust lines).
    Re-verified at source 2026-07-29: `grep -rn "ruin_posts\|Pregen.sites\|pregen/history"
    crates/ --include=*.rs` → **no matches**. This document already says so 500 lines above
    (*"draw-domain part (a) is **discharged by removal** rather than conversion"*).
    **What remains is real and unbuilt:** the two hand-rolled draws still sit in
    `dc-sim/src/statistical/engine.rs` and still want `Draws::of`. But there are no ruin posts
    to move, so — exactly as this entry already says of the agent-step half — **both halves are
    now byte-identical housekeeping.** *Dispatched as previously written, a slice would have
    gone looking for `collapse.rs::ruin_posts` and waited on a user ratification that cannot be
    owed.* Read the trace below as the 2026-07-26 record it is:
  - **(b) `deeptime/{grid,biotic}.rs`'s three call sites — ✅ DONE 2026-07-26, byte-identical**
    (journal/0118, which also carries the rider above).
    They now open `DeepTimeRoughness` / `BioticFire` / `BioticFlood` through `Draws::of`. Two
    side-effects worth knowing: `SALT_BIO_FIRE` / `SALT_BIO_FLOOD` were **deleted** — with no
    production reader left they were copies of an authority nothing else consulted, so the
    agreement test guarding them asserted nothing (clippy's dead-code error is what surfaced it);
    and `SALT_DT_ROUGH` **stays**, because `deeptime/refine.rs` has a **fourth** roughness-jitter
    call site journal/0105's "three" did not count. **Owed: convert `refine.rs:155` and the
    constant and its agreement test can both retire** — left to that file's owner (a sibling held
    it during the (b) slice).
  - **(c) Tag space inside a domain** (`GeoSelect`'s tags 0–3, `GeoAccessory`'s `tag` /
    `tag + 1024`) is hand-laid sub-domaining with journal/0105's exact failure mode at smaller
    scale; a `Domain` per decision fixes it and costs a longer list.
  - Byte-identity impact: **(b) none (measured)**, (c) none if the tags are kept, (a) **the
    region-step draw moves every world's ruin posts; the agent-step draw moves nothing**.


<!-- Two arcs sequenced 2026-07-24 with full reasoning + a reserved continuation
slot each, per the user's "slice-of" principle: never lose what a completed slice
was a slice OF. Each names WHAT, WHY, how it UNIFIES with the larger threads, its
FIRST SLICE, and the CONTINUATION SLOT that outlives that slice. -->

- **FLOW IS ONE PROCESS — flux on FACES, facts on STRATA; retire the receiver tree**
  (arc opened + **RATIFIED 2026-07-25**, user "standing blessed"; design doc
  **`docs/design/flow.md`**, reasoning **journal/0095**). *Supersedes the river half of
  `water.md` and the **expression half** of `earth-processes.md` § 3e-2; § 3e-2's
  constraint half survives **qualified** — the no-divide-crossing rule binds the FREE
  regime only, or regional groundwater is foreclosed.*
  - **WHAT.** One process — *matter moves down a **potential** gradient through a
    resisting medium, carrying a load it exchanges with the substrate.* Recorded as one
    atom: **at (cell, stratum-slot), flux crossed faces F with magnitude M, in form P
    (free/bound), carrying load L of fluid f, under cause C, at chapter K.** Three
    ratified collapses: **(1)** free/bound is an **occupancy** — Fluid is already a form,
    so free↔bound is an **edge on the form graph (S-8)**, which makes a spring a derived
    outlet, a cave free-phase flow below the surface, and karst the already-dormant
    `Cause::Dissolution`; **(2)** in a stratigraphic record **depth IS time** (an
    unconformity is a flow signature, not missing data); **(3)** flux lives on **FACES,
    not cells** — lateral (cell↔cell per slot), **vertical (slot↔slot in a column)**, and
    boundary (atmosphere/ocean).
  - **WHY.** `flow_to: Option<u32>` is one out-edge per cell — a spanning tree. A tree
    represents convergence and **cannot represent divergence at all**: no distributaries,
    braids, fans or deltas. **The Mississippi delta is unrepresentable**, and no threshold
    or grid change fixes a primitive. Today's carve is also **half-wired and invisible**
    (`let (elev, _riverbed) = carve_rivers(…)` — riverbed discarded at both call sites) and
    is routed at ~14.7 km on **pre-erosion** topography, i.e. across a landscape deep time
    then destroys. And a **face is shared**, so refinement built on face data agrees from
    both sides *by construction* — **the seamlessness requirement and the divergence
    requirement have the same answer.**
  - **UNIFIES.** = the paleo-channel/layering need · **§13.8 flow-biased sub-cell fill**
    (which finally has a path to bias toward — the flux record is *the rich thing others
    bias to*, never a second geometry model) · **Movement 2b material-aware transport**
    (the load-exchange half; wind/ice/lava/turbidity are the same machinery per §13.2) ·
    **karst/caves** (corrections #17: dissolution is a term on the weathering edge, not a
    new incision model) · **S11's runtime** (bodies-not-links, `sat.rs`, zero cave-specific
    code) · the **interval-log fill contract** (voids are intervals, not a heightfield) ·
    **`identify(pos)`** (a voxel that is "air that is a cave that was a phreatic tube" is
    exactly what a 1-byte summary cannot say).
  - **THE DEEPEST DEFECT IT FIXES.** *The record is the ONLY seam between deeptime and
    runtime — if refinement needs something, deeptime must have recorded it.* Today
    drainage is solved **every epoch and discarded** (`recv`/`area` are documented as "the
    **last** routing"): we run the process 200× and keep the final frame. Every paleo-flow
    signature the design wants was computed and thrown away.
  - **STATUS 2026-07-25: slice 1 (journal/0096) and continuation (a) (journal/0098) have
    SHIPPED — the ARC STAYS OPEN.** Slice 1: divergence 175,320 / convergence 60,915 on the
    production world; record 40.66 MiB at 8.386 % face sparsity. **(a) the head field:** the
    `head` condition-field (`dc:field/head`) as the second §5 field pass, and its consumer —
    the **vertical faces slice 1 left honestly zero now carry 307,364 crossings** on 131,586
    columns (44.3 % of cells), and **artesian occurs naturally**: 60 subaerial columns stand
    their water above their own ground under a confining bed (max excess 2.94 m), which
    `H = y + sat` cannot express at any resolution — and 24,935 more carry a water table
    *below* ground, a depth-to-water derived from the rock rather than from present-day
    precipitation. Terrain byte-identical; +6.96 MiB (149.21 → 156.16), +0.5 s gen.
    **(b)–(e) are untouched and still owed**, and three riders remain in the slot: the
    **slot-pairing rule for the BOUND regime** (free flow pairs by chapter; bound may need
    paleo-elevation — the user's call; **note (a) did NOT force it**, because vertical faces
    pair *within* a column), the **marine-sink residency lever** (79.38 % of entries,
    31.37 MiB, sized and deliberately not pulled), and now the **recharge term** (a) left as a
    seam — `∇·(T∇h) = −R` needs `R/T` in real units, which is the same missing
    precipitation-depth scale that keeps the lateral source uniform, and it is why (a)'s
    artesian excesses are metres rather than the hundreds of metres of a real basin.
  - **FIRST SLICE — the RECORDING half only.** Face flux per chapter in deeptime: replace
    the receiver output with **face-flux records** (3D faces), attach flow facts to unit
    slots, keep the existing priority-flood/route/accumulate **solve** (good numerics —
    only its output representation changes). **Nothing expressed at runtime yet**; the
    world stays honestly river-less rather than gaining a second fake. Acceptance =
    a **convergence AND a divergence** present in the record on a production world (a
    tree cannot produce the latter), plus a **measured resident cost** (gen time is free;
    residency is not).
  - **CONTINUATION SLOT** (this is a slice OF *"flow is one process, faithfully recorded
    and purely refined"*): after recording, the arc continues with **~~(a)~~ SHIPPED
    2026-07-25 (journal/0098) — the potential/head field pass.** `dc:field/head` is the
    second §5 field pass: it derives each column's transmissivity / vertical conductivity /
    **confinement** from the strata record's own permeabilities (a marine mud over a fluvial
    sand *is* a confined aquifer — no landform code path), relaxes `∇·(T∇h) = 0` under
    Dirichlet conditions at the sea, lakes and perennial streams, and leaves **confined**
    columns **uncapped** — which is the whole artesian mechanism, expressed as *the absence
    of a seepage cap*. It fills the vertical faces (307,364 crossings; 1,137 of them
    artesian **rise**). It leaves as named seams: **recharge** (`R/T` needs real units),
    **precipitation weighting** (would desync the record's two halves, and the lateral half
    moves terrain), **buoyancy/density** (ρ named and multiplied in, inert at 1.0 — heir is
    (d)), and **unsaturated/transient Darcy** (capillary rise needs it). **It did NOT do
    MFD** — deliberately, since that changes routing and therefore the world.
    *The directive that governed it, kept for the intervening slices that still apply:*
    **⚠ USER DIRECTIVE 2026-07-25:** *"build assuming that we want the head field to exist
    soon — i.e. do not foreclose, leave seams where possible, keep direction in mind."* So
    every slice must (i) treat "flow descends **potential**" as the target, never hard-code
    elevation-descent, (ii) keep the **vertical faces** structurally present, and (iii) not
    bake the free/surface **no-divide-crossing** rule into anything the bound regime will
    inherit (flow.md §2.4 — bound flow genuinely crosses surface divides; (a)'s solve has no
    divide term at all, pinned by `bound_head_crosses_a_surface_drainage_divide`).
    ~~**The head field is also what unlocks a multi-flow-direction solve**, and therefore
    *simultaneous* divergence (flow.md §2.6) — today's divergence is still aggregation-window
    avulsion only, and **MFD is now the nearest-term continuation**~~ — **🔴 FALSIFIED
    2026-07-25, corrections #54 (journal/0109).** MFD does **not** read `dc:field/head`. It
    partitions the **free-surface potential** (the priority-flood `filled` array), which
    already existed — and using the **bound** regime's plane would have made rivers cross
    their own divides. The claim was written in **four** places and never questioned because
    it sounded like a sequencing argument. **(a) was still correctly sequenced first** — but
    because it made the record's vertical faces honest, *not* because the numerics needed it.
    **✅ MFD SHIPPED 2026-07-25 as continuation (b′) — see `ROADMAP-history.md` § Shipped, journal/0109.**

    **⚠ LETTER COLLISION, introduced 2026-07-25 and named here rather than silently
    renumbered:** this list's **(b)** is *"refinement as a boundary-value problem"*, but the
    MFD slice was **dispatched as (b)**. Both now exist. Until the arc's owner renumbers,
    read **(b) = MFD (shipped)** and **(b′) = BVP refinement (unbuilt)**.

    **(b′) refinement as a boundary-value problem** — face fluxes as Dirichlet conditions,
    the load budget as mass, solved *inside* a cell, pure-of-position (never reading a
    neighbour's refined output, so the pure-fn chunk holds); **(c)** the **free/bound edge
    + void intervals** (caves, conduits, springs, `Cause::Dissolution` switched on) — **and the
    CONDUIT PAIRING RULE** (ratified into (c) 2026-07-25): a karst conduit is confined by **its own
    void geometry**, not by a depositional horizon nor a potential surface, so it pairs by **void
    connectivity** — a **third mode** that flow.md §11.5's two-mode green explicitly does NOT cover.
    So (c) ships **three** obligations, not two;
    **(d) fluid identity** (lava/ice/brine on the same atom, own competence curves);
    **(e)** retiring `pregen/hydrology.rs`, `RiverSeg`/`carve_rivers`, and
    `Cell::{flow_to,river,discharge}` — **and as of MFD this should DELETE, not supersede**:
    `recv()` is now the *argmax share*, a projection that exists only because two consumers want
    one arrow per cell, so its spines row was upgraded **SUPERSEDED → A-1**. Expect **`area` to be
    the harder half**, since MFD makes exported drainage area a *dispersed* quantity whose peak is
    15× smaller. **Do not close the arc when the first slice lands.**
  - **NAMED LIMITS (honest, from the 2026-07-25 foreclosure sweep — flow.md § 5):**
    oscillatory/tidal flow nets to ≈0 and needs a **gross-energy** term · **episodic**
    catastrophes (turbidite/jökulhlaup/lahar) average away inside a chapter though the
    graded bed IS the signature · **sub-cell meander migration** is refinement-tier ·
    **evaporites** precipitate because the *carrier left*, a distinct trigger from a
    falling competence ceiling.
  - **ACCEPTANCE (the prize, un-gameable by statistics):** cut a cliff face and read
    *channel gravel with a placer streak → floodplain silt → an unconformity where the
    river left → carbonate dissolved into a phreatic tube, now dry → collapse breccia on
    its floor → a spring line downslope.* Each is the same atom at a different slot.
    **If any of it needs a landform-specific code path, it is not faithful.**
    **⚠ WHO OWNS CLASTIC FACIES — CONTRADICTED with the genesis-passes arc; under the user's call, see `docs/audits/2026-07-25-roadmap-staleness-sweep.md` row C-2.**
  - **NOT A CONSTRAINT (user, 2026-07-25, emphatic):** socia/civ/eco/bio consumers
    (settlement siting et al.) are **stubs and baggage to be replaced** — they do not
    exist as designed systems and **must not constrain the flow design at all**. Future
    designs compose with flow; flow does not bend around unavowed stubs.

- **🔴 SLICE (a) SHIPPED 2026-07-26 (journal/0110) AND ITS ACCEPTANCE PROBE RETURNED A NULL
  THAT REROUTES THIS ARC — read before dispatching any continuation below.** The load is now a
  multiset of `(lithology, quantity)`, identity travels source → receiver, deposition is a
  falling competence ceiling, and `DepUnit` carries the material that **arrived** rather than
  one looked up from a tag. Mass closes **per species** (0109's residual rule re-derived, with
  the shared-total hazard pinned by a constructed falsifier — apportioning species from a
  shared denominator leaks *per species*, invisibly, while the total stays perfect). Pass
  purity: **zero exceptions**, and A-7's diagnostic dissolved the one constant the pass wanted.
  - **THE FACIES GRADIENT DOES NOT EXPRESS.** Identity reaches **0.000006 %** of the archive;
    headwater→trunk grain ratio is **0.921 before and after**, unchanged to four decimals.
  - **The measured cause is bigger than hybrid-`p`, in three layers (corrections #55):**
    **(1) fluvial transport is 0.109 % of this world's sediment routing** — rivers pick up
    **659.5 m** over the run, in-place weathering makes **256,886 m**, and hillslope creep
    moves **605,117 m**: **creep does 918× what the rivers do.** *A record built by in-place
    weathering and creep has almost nothing for a river to have sorted — and that is equally
    true of the scalar solve, so it is a fact about the landscape, not about the slice.*
    **(2) No cell on this world can carry sand** — max competence ceiling **0.2832** against
    coarse clastic's **0.840** threshold; 55.5 % of land holds mud, **0.0 %** holds sand;
    **92.25 % of everything picked up is set straight back down in the cell it came from.**
    Corroborated independently: the world's max transport capacity is **6.74 × 10⁻⁴**, *three
    times below* the `energy_band` boundary `COMPETENCE_SCALE` is anchored on — **the
    calibration anchor sits outside the range the world occupies.**
    **(3) Then hybrid-`p`** — a real blocker, but **third**, because a term moving 0.1 % of the
    sediment has no purchase on the archive even with channels restored.
  - **Nothing was tuned.** Lowering `COMPETENCE_SCALE` until sand travelled would have worked
    and would have been *a number pretending to be a mechanism*. The probe prints the anchor,
    the world's actual maximum, and the refusal.
  - **⚠ NEEDS RATIFICATION (user-owned):** `material_transport` **on by default** (it moves the
    goldens — same class as the erodibility/tectonic/MFD flips); and **`COMPETENCE_SCALE = 420`**,
    the knob deciding *which grain sizes this world can move at all*, ~~whose measured answer today
    is **"mud, sometimes."**~~
    **⚠ THE RATIFICATION MAY STILL BE OWED; ITS SUPPORTING TEXT IS NOT CURRENT — annotated
    2026-07-29 (baseline sweep S5/F6). Nothing is decided here.** Two of the three sentences
    have moved under it:
    (i) `material_transport: true` is **already the shipped default**
    (`crates/dc-worldgen/src/deeptime/grid.rs`), so this asks the user to ratify a flag that is
    on — *and no ratification record for it exists in `ROADMAP-history.md`, the close blocks, or
    journal/0110–0112, so either it shipped default-on ahead of its ratification or the record
    is somewhere unread.* **Worth putting to the user as such.**
    (ii) *"mud, sometimes"* is superseded: under the joint calibration **sand now moves on
    0.095 % of land where it was zero** (`journal/0114`).
    (iii) `journal/corrections.md` **#59** falsified *"the competence ceiling is fixed by an
    anchor that already ships, and is not a tuning knob"* — `energy_band`/`competence_ceiling`
    are now relative to `REFERENCE_KT`.
  - **Appearance: announced, and a NULL IN THE VIEWPORT for the second slice running.** Surface
    elevation identical to two decimals. **No walk is owed** — a tour-map would find nothing to
    stand in front of.
  - **🔀 THE CONTINUATION ORDER IS REROUTED BY MEASUREMENT, and this is the user's call.** The
    corpus sequences fluvial refinements next; the numbers say otherwise:
    ~~**(b) the GRAVITY/MASS-WASTING member of § 13.2 — newly promoted to FIRST.** Creep routes
    this world's sediment and **carries no identity**; making it material-aware is where a
    visible facies signal actually lives.~~
    **✅ (b) SHIPPED 2026-07-26 as `journal/0112` — "Movement 2b, continuation (b) —
    material-aware hillslope creep". Struck 2026-07-29 (baseline sweep S5/F5.)** It is the
    shipped default (`grid.rs`: `material_creep: true`), and this document elsewhere cites it
    as done (*"provenance in the archive 0.000006 % → 65.206 %"*). **It is not an open user
    call.** The reroute *judgement* below stands as the record of why (c)–(f) are ordered as
    they are.
    **(c)** Hjulström's U-curve entrainment (§ 13.4) — armouring, desert pavement, cohesive
    persistence, selective winnowing. **(d)** the eolian load — the wind agent has a genuine
    load and still reads its species off the tag. **(e)** lineage/provenance (§ 13.8) —
    `as_deposited` loses the parent at deposition. **(f)** solutes, **blocked** on `flow.md`
    § 2.5 fluid identity (see `material-genesis-notebook.md` § 3).
    **Cross-arc:** hybrid-`p` is FLOW's, and **whoever recalibrates `k_transport`/`k_bedrock`
    should read corrections #55 first — it is the EROSION BUDGET that is small, not the
    sorting rule.** **stubs #23** (the settling law that cannot see buoyancy) is new, with
    fluid identity as its heir and the shipped placer pass as its co-consumer.
  - *Also measured and worth knowing: **no dense-mineral species exists** in the 7-class
    deep-time roster, so **placers cannot ride the load at this tier at all** — the gold-in-
    gravel payoff § 13.5 promises is unreachable until the roster grows.*

- **MOVEMENT 2b — MATERIAL-AWARE TRANSPORT** (material-behavior.md § 13, ratified 2026-07-24;
  **RESHAPED 2026-07-25 by the FLOW arc — read `flow.md` first**). *Given a Sequenced home
  2026-07-25 by the staleness sweep (row R-1): until today this arc existed **only inside a close
  block**, so a wrap rewrite could have silently dropped the thing a whole session pointed at.*
  The load multiset, Hjulström entrainment and settling deposition **stand**.
  - **WHAT CHANGED.** § 13.1 routes transport *"in downstream order along the **pinned
    receiver**"* — and that receiver is the spanning tree FLOW retires (flow.md § 2.1; deletion is
    continuation (e)). Transport must instead walk the **face-flux record** (`DeepField::flux`),
    which **can diverge**, and descend **potential** (`dc:field/head`), **not elevation**
    (flow.md § 2.4). § 13.2's wind / ice / gravity family is the same thing FLOW calls
    `Cause` = **the mover** (flow.md § 7), and 2b discharges **stub #18's constant `cause`
    field**, which already names 2b as its heir. The **load-exchange half is explicitly claimed by
    the FLOW arc's UNIFIES clause**, so the two are one design, not two.
  - **SEQUENCING — DECIDED (user, 2026-07-25): 2b NOW, on today's faces. NOT after MFD.** The
    sweep stated this as an open fork (*"2b after MFD, or 2b on the faces as they stand today?"*)
    and the user closed it the same day.
    - **The consequence, taken with eyes open:** 2b ships **unable to express *concurrent*
      distributaries.** Every divergence in today's record is **temporal — avulsion**, produced by
      the aggregation window (the terrain moves under the flow and the steepest-descent receiver
      *switches*), never **simultaneous** (flow.md § 2.6). A delta with two channels flowing *at
      once* is not representable at any cadence setting. Avulsion is the honest physical origin of
      braid plains and fans, so what 2b can build on today is real, not a placeholder.
    - **And it gains the rest for free when MFD lands.** MFD is a **SOLVE change, not a record
      change** (flow.md § 2.6, § 9 Q8) — a head field partitions flux across several receivers
      where steepest descent cannot. The record shape 2b consumes does not move, so simultaneous
      divergence arrives as *more entries in the same faces*, with no migration on 2b's side.
      That is precisely why "now, on today's faces" costs nothing later.
  - **Provenance, stated because it matters for how much weight the reshape carries:** the
    *diagnosis* (2b's ratified mechanism walks the structure FLOW retires) is **assistant-
    originated** — the 2026-07-25 staleness sweep. The *sequencing call* is the user's.

- **THE AGGREGATION WINDOW IS A DECLARED AXIS** (flow.md § 11.1, **RATIFIED by the user
  2026-07-25**; given a Sequenced home 2026-07-25 by the staleness sweep, row D-3, which found it
  **absent from this board entirely** — a ratified architectural decision living in one paragraph
  of one design doc). § 5's cadence grows a third axis — **ORDER × RATE × WINDOW**.
  - **A window that decides an acceptance number must be declared, not assumed.** Slice 1's
    divergence count — **175,320 divergent `(cell, chapter)` pairs, 7.378 %**, the number the
    slice was *accepted* on — was produced by an **implicit** 25-epoch chapter. At a one-epoch
    window that count is **zero** and the tree structure reasserts; set `K` and you set the
    count. The user's reasoning, recorded: *"freedom to future mods / ourselves (we are the first
    modders)"* — a mod authoring a pass must be able to state its own record granularity the same
    way it states order and rate, and **an undeclared constant is exactly the surface a third
    party cannot reach.** This is the north-star's *"authored in a uniform, self-declaring shape
    and tuned by data"* applied to the **time** axis.
  - **Consequence (§ 11.2): chapter-vs-epoch resolution becomes a shipped DEFAULT, not an engine
    property.** Once the axis is declared, per-chapter vs per-epoch flow facts stop being an
    architecture question. **Default cheap** (the coarser window, for dev-iteration speed);
    **expose the knob** for stress tests. *(User, confirmed: "cheap end for dev iteration,
    precisely.")*
  - **RIDER (§ 11.3) — the self-describing-record contract, a STANDING constraint on every future
    record.** *Any mode that changes what an **absent** entry means must be carried in the
    record*, never held as external knowledge — otherwise absence is ambiguous across worlds and
    every consumer must know how a world was generated in order to read it. **S-9 one level up:
    the answer carries its resolution.** Its first customer is the **marine-sink lever** (79.38 %
    of slice-1 entries, 31.37 MiB, sized and deliberately not pulled — default KEEP): the drop is
    not forbidden, it is **gated on this contract**.
  - **SPEC HOME.** `material-behavior.md` § 5 — now **"Cadence: order × rate × window"** — is the
    spec of record for the scheduler; it was amended 2026-07-25 to carry the third axis and to
    cross-reference flow.md §§ 11.1 / 11.3 (`ideas.md § Pass cadence`, the sketch it reconciles,
    was corrected to match). **Sequenced, not built** — no `Pass` declares a window today, and
    today's one aggregating record (`DeepField::flux`) buckets by chapter as an implicit constant.

- **STRUCTURE-AWARE FINE EXPRESSION — the sub-resolution the collapse randomizes but
  physics structures** (filed 2026-07-24 at the user's direction; unifies the flow-biased-fill
  deferral of material-behavior.md §13.8 with the journal/0055 + journal/0010 within-voxel
  order residual and stubs.md § 12's residual).
  - **WHAT.** The collapse expresses a coarse record at fine resolution by **addressed
    stochastic rounding** (journal/0055) — mass-honest but **spatially unbiased**. Two scales
    of one gap:
    - **Within-VOXEL order (sub-0.9 m):** a mixed voxel loses the vertical ORDER of what it
      mixes, so a sharp contact renders as **journal/0010 speckle, not a band** (stubs.md § 12
      residual; ratified non-blocking, "belongs to the forms presentation work").
    - **Within-CELL distribution (0.9 m–460 m):** a deep cell's deposited material is spread
      ~uniformly across its column, **not biased** to where it physically settled — coarse near
      the paleochannel, fines in the distal lows, cross-beds dipping downflow
      (material-behavior.md §13.8).
  - **WHY THEY ARE ONE ITEM.** Both are **S-4 "coarse cause → fine expression"** where the fine
    expression **randomizes where the physics would structure**. Same class, two axes (vertical
    bed order vs horizontal flow bias), same eventual home — the **forms / presentation pass**.
    The fix reads a **structuring signal** (contact orientation for order; ~~the flow field `recv`~~
    **the flux record** — `DeepField::flux`, per-chapter directed flux on faces, journal/0096 —
    plus local topography for placement) instead of the current unbiased sieve.
  - **INPUTS: the FLOW half already exists** (corrected 2026-07-25, sweep row S-3; was *"inputs
    already exist"* naming `recv`). The flux record shipped 2026-07-25 and carries **direction and
    magnitude per chapter**, which `recv` structurally never could — one out-edge cannot express a
    fan, and `recv` is the spanning tree the FLOW arc retires (deletion is continuation (e)). So
    **do not re-derive "we must wait for Movement 2b" for the flow half.** What still rides
    Movement 2b is the **composition** half — the cell's material multiset that biased fill places.
    The record also still needs to carry the within-voxel order the sieve currently drops.
  - **STATUS: non-blocking, and the two scales differ in value** (user, 2026-07-24). The
    within-VOXEL order loss is **ratified-acceptable** — "granted micro-scale lossiness, still
    reads realistic for most purposes"; **not a defect to fix.** The within-CELL flow-biased
    placement is an **interesting-but-unsettled enhancement** — "super interesting… not sure
    about that one, food for thought"; NOT a committed direction. This entry keeps both
    discoverable and honestly-statused; the biased-fill inputs already exist (§13) so nothing is
    lost by leaving it open.

- **MIGRATE EVERY "NOT REAL" FIELD INTO A REAL DECLARED FIELD PASS** (reminder, user-directed
  2026-07-24). As the §5 **field-pass half** lands (geotherm = the first, this session), the
  ad-hoc / stub / unconsumed proto-fields must each migrate into a **real declared field pass
  writing to the condition-field vocabulary** — not stay ad-hoc planes.
  - **⚠ TARGET LIST REWRITTEN 2026-07-25 (sweep row S-1) — three of the four named targets moved,
    and two of them are not migrations at all.** What survives as written: **`exhum` + `t_crust`
    exported planes** (spines § 3, still unconsumed — their heir is **metamorphic grade**,
    stubs #4; see the Metamorphism entry below). Plus any field computed inline that a formation
    predicate or cellular pass ought to read by id.
    - ~~the **degenerate `burial_temp_c` geotherm** (stubs.md #14)~~ — **STRUCK: retired
      2026-07-24** by the real geotherm (journal/0093), which took it out of the provider set
      entirely rather than migrating it. stubs #14 is headed RETIRED.
    - ~~drainage `recv` / `area` / `lake`~~ — **STRUCK: these are a DELETION target, not a
      migration target.** spines § 3 marks the row *"SUPERSEDED 2026-07-25 (journal/0096, FLOW
      slice 1), retirement sequenced"*; the heir is **`DeepField::flux`** and the disposal is
      **FLOW continuation (e)**. Migrating them into a declared field pass would be building a
      declared home for a plane we have already agreed to delete.
    - `dc:deep/drainage` has been **a declared pass since Movement 1** (journal/0090; it is named
      in the 17-pass order in `ROADMAP-history.md` § Shipped, journal/0104) — the unconsumed thing is the exported **plane**, not the
      pass. The entry's original framing conflated the two.
    - **Vocabulary the migration did not have to open:** `dc:field/temperature` (0093) and
      `dc:field/head` (0098) both landed as first-class field passes on their own arcs — two rows
      this reminder can stop carrying.
  - **Each migration empties a §3 row and grows the SDK's condition-field vocabulary.** The
    condition-field vocabulary IS the formation-predicate SDK surface (§12 formation is
    output-owned; the predicate is plain data over field-ids — crossing constraint satisfied).


- **WEATHERING IS ONE PROCESS — SAPROLITE IS A STATE ALONG IT, NOT A SLICE**
  (**USER'S STRONG LEANING on the destination, 2026-07-25** — *"strong enough that it should pop
  into sequence when the requisites are met"*. ~~Not scheduled; **gated on the requisites below, and
  it enters Sequenced the moment they are met.**~~)
  - **✅ REQUISITES MET 2026-07-25 — R1, R2 and R3 all (see the marked requisites below). THE GATE
    IS OPEN AND THE BLOCKER MOVED: this is now a DESIGN question, not a prerequisite question.**
    (Corrected 2026-07-25 by the staleness sweep, row S-2; the header still read "gated on the
    requisites" after commit `160858b`'s own message said *"the gate is open, and the blocker
    moved."*) Before it can be built, the arc must answer **what is persisted** — the **chapter**
    axis, the **agent** axis, or a **per-slot scalar** (R3's three levers, below) — which is
    exactly the *"a summary must be derived from the authority, never become it"* question this
    project already has a doctrine for. The numbers that frame the call: **gen time is affordable
    (25.7 s → 46.4 s); residency is not (`LedgerField` 17.45 MiB → 973 MiB, 55.8×, resident).**
    The S20 spike costed four options for that decision, including paged facts.
  - **WHAT.** Retire saprolite as a bespoke thing. Weathering becomes **one declared process**
    acting on whatever is exposed to reactants at a rate set by *material susceptibility ×
    driver × access* — the gradation **emerging from the rate**, not imposed by a shape function.
    Saprolite is then simply the region of the continuum where parent fabric is still recognisable.
  - **WHY (the user's reasoning, recorded).** *"Saprolitification is a slice of a more general
    process of weathering that ends in totally crumbling — are we splitting that one process
    between multiple slice owners for any good reason?"* Today it **is** split three ways:
    `dc:deep/weather_inventory` (declared, deep tier, produces a **scalar**) · `dc:deep/weather`
    (the scalar `R`/`H` height authority, still unreconciled) · `geology.rs::emplace_weathering_front`
    (**not declarative** — ordinary collapse code that *invents* the vertical distribution). Only
    one of the three is authored in the shape everything is obligate to converge on.
  - **THE THREE ARGUMENTS THAT CARRY IT.** **(1)** Nature has no saprolite *process* — it has
    weathering, and saprolite is a **named state along a continuum** (intact → fractured → saprock
    → saprolite → residual soil); modelling a stage as a mechanism is the same category error as
    S18's one-shot. **(2)** The gradation would become **emergent**: fronts are exponential
    *because* first-order kinetics consume a downward-advecting reactant, so modelling the cause
    makes `WEATHERING_PROFILE` and **stubs #20 DELETED, not tuned** — the disposal the stub doctrine
    wants. **(3)** The **7/8 cap is a definition wearing physics' clothes** — weathering does not
    halt at 7/8 retained fabric, so the cap makes complete weathering (laterite, oxisol, total
    crumbling) **unrepresentable**. Also: today the pass only weathers **bedrock**, because the
    materialized seam is the only `Structure` in the inventory — nature weathers sediments, soils
    and transported clasts too (that narrowing is stub #16's shadow, not a physical claim).
  - **IT DISSOLVES THE PROFILE-vs-NORMALIZATION PROBLEM** (the user's own conclusion, and it is
    right): *a scalar only needs a profile invented for it when the process forgot to record depth.*
    This is the same finding as journal/0099's PLEA, reached from the other direction — the collapse
    can only impose a **universal** shape while the physical controls already live in the deep sim.
  - **⚠ REQUISITES — the honest reason this is not scheduled yet (the counter-case is about
    ORDERING, not destination).** The exponential emerges from **reactant transport**, and we have
    **no vertical fluid flux**. A depth-resolved weathering rate would have nothing honest to read,
    so "emergent" gradation would emerge from a **fabricated depth term** — the same shape function
    with better camouflage (**A-1** with a disguise). It pops into Sequenced when:
    ~~**(R1)** the **potential/head field** lands~~ **✅ MET 2026-07-25 (journal/0098)** and
    ~~**(R2)** **vertical flux is real** rather than the honest zeros of flow slice 1~~
    **✅ SUBSTANTIALLY MET 2026-07-25** — 307,364 vertical entries across 44.3 % of cells, **with
    one honest caveat that survives into the arc**: that flux is **recharge-free** (stubs #19), so
    a depth-resolved rate reading it reads **gravity drainage with no real precipitation depth
    behind it** — relatively shaped, absolutely uncalibrated. ~~**(R3)** measure the cost first:
    per-depth × per-cell × per-epoch over 297 k cells with multi-slot columns is far larger than
    today's per-cell scalar, and the deep run is already 35–85 s.~~
  - **✅ R3 MET 2026-07-25 (journal/0106, `examples/perdepth_weathering_cost_probe.rs`) — and it
    moved the blocker.** Measured on the shipped world (1337 / Medium, 297 025 cells, 200 epochs,
    8 chapters), with the pass timed OFF vs ON so its own wall clock is a *difference*, not a share:
    - **The multiplier is 59.5× in tuples and 5.6× in TIME.** A per-depth firing visits **59.5 slots
      on average** (20 487 597 `(cell, slot, chapter)` visits against 344 410 `(cell, chapter)`
      firings; epochs-per-chapter cancels, so this is the per-epoch multiplier too). But today's
      firing is dominated by **fixed** cost — it re-derives a working inventory and drains a commit
      log for a single span — so 552 ns → 3 066 ns. **The work that multiplies is the cheap work.**
    - **The causal triangle is real and worth 38.7 %.** A slot deposited in chapter `c` cannot
      weather before `c`; honouring that is 20.5 M visits instead of 33.4 M, and +20.7 s instead of
      +37.8 s. It is **free** — a pass walking the live record gets it automatically, and you have to
      work to lose it. **Say so in the design so nobody builds the rectangle.**
    - **GEN TIME IS NOT THE BLOCKER.** The pass is 4.5 s of a 25.7 s deep run (17.6 %); per-depth
      projects to 25.2 s, taking the run **25.7 s → 46.4 s**. Against the standing doctrine (*gen
      time is not a constraint*) that is affordable.
    - **🔴 RESIDENCY IS THE BLOCKER.** The `LedgerField` sidecar goes **17.45 MiB → 973 MiB**
      (55.8×), and it is **resident** — it ships in the `DeepField`. 61.5 M facts = 20.5 M visited
      slots × 3 agents × 16 B. The itemisation reconstructs today's measured footprint **exactly**,
      so the projection scales a validated model.
      - ~~**The levers are axes, not micro-optimisations:** drop the **chapter** axis (÷ ~4.8);
        drop the **agent** axis in the persisted form (a further ÷ 3); or persist a **per-slot
        scalar**~~ — **✅ ANSWERED 2026-07-25/26. The user chose S20 option 3 + 2c, and NO AXIS IS
        DROPPED.** S20 priced all four families and the axis-drops lost on their own terms: the
        cheapest (4c′, 70.05 MiB) is still **~2× option 3's resident footprint** *and* has thrown
        away the provenance option 3 keeps, so **no axis-drop dominates paging on the numbers**.
        Dropping chapter would have deleted the derived-story layer; dropping agent would have
        retired, at the storage layer, the sum-not-product distinction ratified one day earlier.
        Both irreversible — the facts are not re-derivable without re-running the compile.
      - **✅ HALF BUILT (2026-07-26, journal/0108): the 2c encoding shipped.** `Fact` 16 B → 8 B,
        every axis retained; resident **17.45 → 9.57 MiB**, per-depth projection **973.40 →
        504.50 MiB**. **The blocker is halved but not cleared** — 504 MiB is still a resident
        half-gigabyte on a Medium world.
      - **🔒 RESERVED CONTINUATION — S20 OPTION 3, THE PAGER.** *This is what journal/0108 was a
        slice OF.* Target: **37.86 MiB resident / 468.91 MiB on disk** (halved by 2c), every axis
        retained, page-in **p50 ~100–220 µs cold, ~5 µs warm** — an inspect-one-voxel cost of 1–2 %
        of a 60 Hz frame, and *faster than the chunk generation the same click may trigger*. It is
        **S-9 with the overlay on disk**, so it is a shape already ratified rather than a new
        mechanism. **What it owes, all still unpaid (S20 § 5):**
        - an **injected `FactStore` port** — *not* `std::fs` in a headless crate. `dc-core`,
          `dc-sim` and `dc-worldgen` still contain **zero** `std::fs`/`std::io`/`File::` in `src/`,
          and a filesystem dependency there is a new capability class that is **easy to add and
          very hard to remove**. The honest shape is a trait `dc-worldgen` defines and does not
          implement.
        - a **versioned derived artifact** keyed by `(seed, extent, deep config, worldgen version)`
          with a **tested** staleness branch — a stale file paired with a fresh world produces
          confident wrong provenance, which is **worse than none**.
        - a **writer/reader split across process lifetimes**, and **two artifacts, not one**: the
          fold must persist too, because a process that did not generate the world has no fold, and
          rebuilding it from the paged facts means reading the whole file — the thing paging exists
          to avoid.
        - a **"provenance unavailable"** answer on S-9's resolution flag. The hot query is
          unaffected (*"what is this rock?"* reads the fold); the cold one must be **allowed to say
          nothing** — a shape we already own (journal/0101, `Identity::Unrecorded`).
        - it also **discharges stub #21** (the positional mixed-radix edge id), which it must build
          anyway the moment a ledger is written to disk.
        - **One residual S20 named and did not price:** all 37.86 MiB is **addressing**, not
          payload — 4.5 M CSR rows at 8 B. Coarsening the per-slot *fold* (fewer, thicker slots for
          the fold than for the facts) would cut it further while the facts on disk keep full
          resolution. **Nobody asked for this; recorded so it is not lost.**
      - **CROSS-REF (added 2026-07-25, sweep row A-1): the `DeepField::strata` collapse is the next
        lever on the SAME budget.** Observed *"the SAME lever, one record over: `DeepField::strata`
        is `Vec<DeepStrata>`"* — **9.06 MiB of struct headers over an 84.47 MiB heap, 11.3 % of
        cells holding an empty record**. It was filed as a residency item in its own right; R3 has
        now put **residency, not gen time**, on this arc's critical path, so the two entries are
        competing for one budget and should be read together.
    - Incidental re-confirmation: **72 006 of 297 025 cells (24.2 %) ever weather** — journal/0102's
      75.8 %-never figure, re-measured from the other side; the CSR layout it motivated is why today's
      sidecar is 17 MiB and not 31.
    - **Nothing of the change was built.** The probe's "per-depth firing" is a cost model in
      `examples/`, written against the substrate's existing public API. It prices the **cellular** half
      only, which is correct: reactant transport is a *field* pass (§5's split, below), i.e. today's
      per-cell-per-epoch cost class — it changes the constant, not the exponent.
  - **ONE CONSTRAINT ON "ONE PROCESS" (integrator, agreed at the same time):** *one process must
    NOT mean one pass.* §5's split still binds — reactant **transport is a FIELD**, the **edge is
    CELLULAR**. Collapsing them would rebuild the monolith the pass-runner exists to prevent.
  - **WHEN IT LANDS:** stubs **#20 deleted** (not tuned), the 7/8 cap gone, `WEATHERING_PROFILE`
    deleted, the collapse fold demoted from *inventing* a distribution to *expressing* a recorded
    one, and the two-authorities split (material vs height) becomes derivable rather than
    maintained — which is Movement 2a's stated direction anyway.


- **GEOTHERM: nonlinear / mantle-heat enrichment** (followup, user-directed 2026-07-24). The v1
  geotherm (material-behavior.md §14) is a per-cell **linear** gradient `T(depth) = surface_T +
  gradient·depth`. Enrich later: non-linear `T(depth)` + a mantle-heat contribution (deep
  thermal structure, not just a surface-anchored line). Non-blocking; the linear v1 is the
  honest first field pass and the vocabulary/consumer shape it lands is unchanged by the
  enrichment.

- **THE HONEST IDENTITY SURFACE — retire the stored `Block` summary; one
  `identify(pos)`** (arc opened 2026-07-24; priors: materials.md
  block-is-material DECIDED, the block-consumer recon, ARCHITECTURE § "a summary is
  not an authority", S-3/S-9).
  - **✅ SLICE 1 SHIPPED 2026-07-25 — journal/0101. THE ARC STAYS OPEN.**
    `HostWorld::identify(pos) -> Identity` (`dc-api/src/identify.rs`): untiered,
    position-addressed, payload uniformly a mixture, with **`Identity::Unrecorded`
    as a first-class value** distinct in the TYPE from `Mixture(EMPTY)`. Wired into
    `dc:world/get_contents` (both lying fields fixed), `character_sense_raycast`,
    and the F3 HUD (its `(no contents record here)` branch is now reachable).
    Enabler: the stored `Block` already disambiguates — empty record + `Air` ⇒
    genuinely empty, empty record + anything else ⇒ unrecorded — so **no
    dc-worldgen change was needed**. A *non-empty* record still wins outright, so
    the block/classified divergence that signals an edit stays legible. Census
    through the real query path (`dc-client/examples/identify_census.rs`): phantom
    air **702 → 0** (of 10 985 solid voxels over 169 columns; the 702 land in
    `UNRECORDED` beside the 4 211 that were already honest, and the 6 072 recorded
    mixtures do not move). **Still owed by this arc:** everything in
    the continuation slot below — nothing was drained, deleted, or migrated, and
    the path remains **edit-blind for composition** (an edit writes a `Block`; no
    mixture is stored, so none can move).
  - **WHAT.** The honest answer to "what is this voxel" is its full `VoxelContents`
    (up to 8 partials), never one arbitrary component. One **position-addressed**
    function `identify(pos) -> payload`, where the payload is **uniformly a mixture**.
    Raycast composes it (`raycast(camera) → pos → identify`) — a separate step, because
    "what is at X" is a world question, not a camera question.
  - **⚠ UNTIERED — DECIDED 2026-07-25 (user), SUPERSEDING the Near/Mid/Far tier design.**
    This entry previously specified `identify(pos) -> { tier, payload }` with tiers
    derived from the LOD ladder. **Retired.** The user's challenge: *"if this is a world
    query then why tiered at all when we can inspect chunk and read the voxel?"* — and
    the provenance matters: **the tiering was assistant-originated**, an artifact of one
    request ("a way to get voxel composition by looking at it") being split into several
    instruments, which then hardened into architecture without ever being re-challenged.
    - **The defect it encoded: two different questions were CONFLATED.** *"What is at
      world position X"* is a **world** question with one true answer, to which distance
      is irrelevant. *"What is the renderer showing at X / which rung is resident"* is a
      **render** question — legitimately per-viewer, legitimately vaguer, and the only
      one with any business reading `LodLadder`. This entry's own line already said so
      ("a world question, not a camera question") and then built a camera concept into
      the world query anyway.
    - **Cost was the strongest counter-argument, and it fails.** Gen is **pure-of-position**,
      so the true answer is *always* derivable — there is no fundamental barrier, only a
      latency one. Cost must change the **policy or the latency, never the ANSWER**;
      returning a vaguer truth because the honest one was expensive is precisely the
      summary-wearing-an-authority's-clothes this arc exists to retire.
    - **THE SHAPE:** `identify(pos)` returns the honest full mixture at `pos`, **always** —
      **resident chunk first (INCLUDING dirty/edited state)**, derived if absent,
      **`UNRECORDED`** where no record backs it. **No tier flag, no `LodLadder` coupling,
      no distance thresholds.** Dropping tiers also dissolves the "whose ladder?" problem
      (per-viewer dc-client state vs a headless world query) — there was no good answer
      because the question was malformed.
    - **What survives unchanged:** payload is **uniformly a mixture** (so far-field
      *speckle* falls out by construction — the renderer speckles, the query reports the
      mixture; there is no far tier left to special-case), and **`UNRECORDED` as a
      first-class answer** (corrections #49's measured 6.4 %).
    - **"Reads the dirty rails edits too" (user) is the sharp half**, and it lands on a
      known gap: contents are re-derived from worldgen and are **edit-blind** today
      (journal/0088 — the runtime `Chunk` stores only the 1-byte `Block`, so a mixture does
      not survive an edit). That *is* the **runtime edit-fact overlay** already in this
      arc's CONTINUATION SLOT — contents as *derivable base + edit facts* — and it is what
      makes "break gives you the real mixture" true.
  - **THE F3 INSTRUMENT ALREADY EXISTS, and is fine as-is** (user, 2026-07-25). The
    look-at contents HUD (`dc-client/src/inspector.rs`, journal/0088) fulfils some version
    of the original request: *"I just use F3 to know what's in front of my face when doing
    the walks."* It **rides the looking-at-voxel highlight box, so it has a limited range**
    — and that is **honestly fine for now**. **NOT A PRIORITY (user, explicit):** when
    `identify` is better, the HUD *could* raytrace and ride that too — which would mean
    generating a chunk from a **LOD octree node intersection** and then finding the voxel
    intersection from the same ray. Recorded so the idea is not re-derived; **do not
    schedule it.** *(Note: this HUD carries the corrections #49 defect identically — its
    correct "no contents record here" branch is unreachable — so the `UNRECORDED` fix
    repairs it for free.)*
  - **INFORMATION-AVAILABILITY IS A GAMEPLAY LAYER, NEVER A QUERY LIMITATION** (agreed
    2026-07-25; **no plan, not scheduled**). If the game ever wants "you cannot identify
    distant strata without a survey instrument", that rides **on top of** an honest query.
    User: it *"would come through layers of honest obscurity due to how player perception
    and knowledge will be modeled"* — i.e. it belongs to the (undesigned) perception /
    knowledge model, **not** to the world query. Recorded specifically so nobody
    re-derives distance-tiering later by mistaking a game rule for an engine constraint.
  - **FIRST CONCRETE REQUIREMENT, now measured rather than argued (corrections #49,
    2026-07-25):** the tier flag must be able to say **"UNRECORDED"** as a first-class
    answer, distinct from both *"air"* and *"recorded"*. Today `has_contents` is answered
    **per-chunk**, so an unrecorded basement voxel reports `has_contents: true` +
    `classified: dc:air` **over solid stone** — **6.4 % of near-surface solid voxels**,
    globally. `Option::None`, the only channel that could have meant "no record here", was
    already spent on a whole-chunk condition inherited from the mesher. **A summary is not
    an authority — including when it is a `bool` named after the thing it is not
    measuring.**
  - **WHY.** "Nobody wants one arbitrary material component of a mixture" (user). The
    summary existed for the **surface-only** far field; the far field went **volumetric**
    (FF2b), so its justification **expired — A-2**. The renderer already re-derives full
    contents (`chunk_contents`) and splats them; the stored 1-byte `Block` is a
    storage/sim relic the render **bypasses**, and it loses the mixture on edit.
  - **UNIFIES.** = the block↔material collapse tail (storage palette · far-span ·
    ~80 solidity · mesher layer pick · player-facing name) + ~~the **distance pyramid**
    (near/mid/far *are* the `identify` tiers)~~ **— CORRECTED 2026-07-25 (sweep row A-5): that
    framing died with the tiers.** `identify(pos)` is **untiered**, so the pyramid does not "have"
    the tiers; **the distance pyramid is a RENDER concern only**, and it is what the palette-quant
    station and the far-LOD material split converge into. + **S-3** (`classify` demoted to a derived
    rung + the pyramid's coarsest tier, never a stored authority) + **S-9** (self-labeled
    honesty).
  - **FIRST SLICE — ✅ SHIPPED 2026-07-25 (journal/0101), see the slice block at the top
    of this entry.** As dispatched: `identify(pos)` as one **untiered** surface behind
    `world_get_contents` / `sense_raycast` / the F3 HUD, with `UNRECORDED` first-class.
    **Not** done, and deliberately: draining the `Block`-token consumers (recon list) —
    that moved to the continuation slot with the rest of the retire. Contents stay
    re-derivable (unedited).
  - **CONTINUATION SLOT** (this is a slice OF *"the runtime canonical is contents, not a
    summary"*): after `identify`, the arc continues with the **storage/wire migration**
    (edited voxels carry contents/edit-facts, not a `Block`), the **runtime edit-fact
    overlay** (break/place = move-facts — the north-star's *first runtime-process
    milestone*, which also makes break-gives-the-real-mixture true), the **far-span
    `Block`→material** migration, and **legacy-S1 retire** (Crux 2). Do not close the
    arc when the first slice lands.

- **GENESIS-PASSES DRIVE ROCK DISTRIBUTION — retire class-member into hierarchy +
  property-driven passes** (arc opened 2026-07-24; priors: materials.md "the class
  system's fixed-constant roster is scaffolding" + transformation-axes DECIDED, north-star
  declarative-materials, the entry-species probe, the seam-inventory `[S2]`).
  - **⚠ WHO OWNS CLASTIC FACIES — CONTRADICTED, the user's call; see `docs/audits/2026-07-25-roadmap-staleness-sweep.md` row C-2.** *(Three ratified things claim one output: this entry's "physical facies", the FLOW arc's un-gameable facies acceptance test, and material-behavior.md § 13.7's "transport **IS** clastic sedimentary genesis". Probably complementary — genesis = which rock, flow/transport = where the clastics went — but nobody has said so, and this arc's first slice converts `deep_class`/`dithered_member`, the same seam transport would move.)*
  - **🔖 OPEN EDGE — READ [`docs/design/material-genesis-notebook.md`](docs/design/material-genesis-notebook.md) BEFORE DISPATCHING THIS ARC.** A 2026-07-26 design
    conversation put the arc's central noun in question. **Ratified out of it:** the
    creation/transformation discriminator is **resolution, not phase** (*can the prior be
    named as a material we track?* — the material ontology is a **sieve**, and that is the
    whole case for genesis); a **four-way** test in which *"transform whose driver is not
    built yet"* is its own category and **filing one of those as a genesis is
    irreversible**; the **completeness test**; and — settling row C-2 — **genesis makes
    the *parent* honest · weathering the *loosening* · transport the *destination***, so
    clastic facies belong to transport and this arc **relocates** to *"which rock was
    emplaced here in the first place"*, inheriting **stub #16** as its first customer.
    **Left OPEN, and it bears on this entry's first slice:** if genesis is (a) a pre-loop
    initial condition + (b) a name for a seamed driver + (c) folded competitive
    precipitation, then *"genesis pass"* may not be a pass **type** at all. The user
    ratified only that **pre-loop is not a pass**, was *"NOT sold"* that ongoing formation
    is untractable as material-as-pass, and held **emplacement** open as possibly
    *transform → transport → transform* with no genesis at all. **Do not re-derive this
    conversation; read the notebook.**
  - **WHAT.** Retire the class-member-fitness abstraction. Rock **distribution + physical
    facies** come from **deeptime genesis passes** that select a **parent material**, march
    its **leaf** materials, and derive fitness **purely from properties stored on each
    material** (grain size, solubility, density…). Adding a leaf → it participates
    automatically. Diagenesis/metamorphism = **transform edges** (thermo/pressure agents)
    evolving the distribution across deeptime (the fact-ledger, applied to genesis). Output:
    where materials are · how distributed · the facies · **octave**-materialized → runtime.
    The material **hierarchy** (parent→leaf) serves as ~classes where needed.
  - **WHY.** The class system is self-admittedly *scaffolding* and **measured near-flat**
    (entry-species probe: formation context barely changes which member). North-star wants
    materials+passes+hierarchy; property-derivation makes it **modder-extensible** (leaves
    free); and it is the honest model — a rock's identity is set by its **genesis, at
    genesis, by the pass**.
  - **UNIFIES.** This **IS the member-selection/octave arc done right**: the genesis pass
    produces the distribution (property-derived, physical), the octave dither materialises
    it — so "retire class-member", "fix the member squares", and "physical facies from
    passes" are **one arc**. Rides the material-behavior model (cellular passes, edges,
    agents, hierarchy).
    - **⚠ STRUCK 2026-07-29 — this bullet also claimed it *"fixes the palette-quant + LOD
      root cause (coarse facies point-sampled)"*. It structurally cannot, and the claim was
      never ratified.** Verified in code, not from either doc: `interp_select_draw` selects a
      **member** *after* `run_strata` has already fixed the chunk's **class stack** from
      `record_at_voxel(cx*32+16, cz*32+16)` (`collapse.rs:1409` — *site retired 2026-08-02/03,
      journal/0129 + 0145; historical citation, staleness F12*). Octaves applied to the member
      dither run **downstream** of the class boundary the deep sample drew and cannot move it.
      The same-day audit said so plainly and nobody reconciled the two:
      `docs/audits/2026-07-24-palette-quant-generation-diagnosis.md` § 2 — *"the per-voxel
      dither only walks the member around **inside the class the chunk already fixed**; it
      cannot cross the class boundary the deep sample drew."*
    - **The damage was OWNERSHIP, not just accuracy.** This clause made the genesis arc the
      sole apparent owner of the palette-quant fix, and the cure the user actually ratified —
      the CoarseField adoption, *"solved by construction, approximately once"*, *"this is
      foundational"* — was left with **no live entry, no owner, no slice**. Revived as its own
      § Sequenced entry, 2026-07-29. **The facies-driver half of this arc is still correct and
      still belongs here**; it is genuine content work (a declared field pass). Only the claim
      to discharge palette-quant is withdrawn. *corrections #65 shape: an assistant
      reconciliation narrowed a ratified user-facing cure into a clause of an entry the user
      does not re-read.*
  - **STEELMAN (recorded so it isn't re-lost):** keep the **distribution concept** (a rock
    unit *is* a mix of related materials — honest, housed in the hierarchy); class-fitness
    is a genesis **model** the pass *formalises*, not deletes; the **richness lives in the
    pass's physics** (property→distribution), which is near-flat today, so re-housing alone
    buys nothing — the win is encoding real depositional/petrological physics; **leaves are
    free, a new genesis PROCESS is a new pass**.
  - **FIRST SLICE.** **Seam-first, byte-identical:** convert the `DepTag→material` /
    `deep_class`/`dithered_member` determination (the seam-inventory `[S2]`
    `material_properties`) into a declared pass/provider that reads material **properties**
    — no behavior change, goldens hold.
  - **CONTINUATION SLOT** (this is a slice OF *"genesis is a property-driven pass
    pipeline"*): after the seam conversion — (1) **enrich the property→distribution
    physics** so energy/depth/thermo genuinely swing the mix; (2) **octave materialisation**
    (multi-octave + physical driver, retiring the single-octave chunk dither — the member
    arc); (3) **diagenesis/metamorphism as transform edges** (thermo/pressure agents, on
    the fact-ledger); (4) the **modder-authors-a-leaf** path (properties only, no bespoke
    code). Do not close the arc at the seam conversion.

- **The distance pyramid: consume the mixture LOD by band, drop the dither
  with range** (SEQUENCED 2026-07-21 at the user's direction; design captured
  in visuals.md § "LOD colour cascade"). Three bands, knobs on the dropoffs
  (user doctrine: *"where there's a cell range, there's a knob"*):
  **near** = texture / the 4×4 world-anchored dither over the real mixture;
  **mid** = the *averaged* mixture, one blended colour per coarsened voxel;
  **far** = the winning material only, one flat colour.
  - **Most of the data machinery already exists and is unrendered.** S8's
    `MixtureDownsampleRule` / `DominantClassDebrisAware`
    (`dc-core/src/materials/lod.rs`) already reduces a 2×2×2 cell of child
    `VoxelContents` (64 eighths) to one parent (8 eighths) — occupancy voted in
    eighths, class by volume with the losing class *folded in rather than
    lost*, slots by largest remainder, mirroring the block LOD's octant layout
    voxel-for-voxel with a proved invariant. The far band's "winning colour" is
    `dominant_material`/`classify` (journal/0052). **What is missing is a
    renderer that consumes the pyramid by distance**, not the pyramid.
  - **The perf case is measured, not assumed**: journal/0010 clocked a
    fully-mixed chunk at **16× the triangles** of a uniform one (196 608 vs
    12 288), purely because each mixed face becomes 16 dither quads. Journal/0055
    made mixed faces world-wide, so that ceiling is now being paid at range.
    Dropping the dither by band saves exactly on the faces that just got
    expensive.
  - **Enabling piece, cheap, do it first**: cache the blended colour **per
    interned `MixtureId`** rather than recomputing per voxel — the region
    `MixtureTable` is tiny (journal/0055 measured 325 mixtures / 3 160 bytes),
    so it is a rounding error and is computed once. (The user's instinct was
    per-material precompute; per-material albedo is already registry pack data
    from 3c-2, so per-mixture is the version that actually buys something.)
  - **Sequencing**: lands after the in-flight `meshing.rs` side-face fix
    (same file). Pairs naturally with **FF2b** (coarse volumetric summaries)
    and with the Observed *"far field is boxier than the near field"* item —
    a banded pyramid is the obvious place to carry sub-voxel height too.
    Band distances are an **appearance call: the user picks from pictures**,
    so ship the knobs before the values.
- **Edited chunks still accumulate for the session — the save-layer heir**
  (surfaced 2026-07-21 by journal/0051, which bounded everything *else* in
  `HostWorld.chunks`). Eviction deliberately pins edited chunks, so a session
  that edits 10 000 distinct chunks holds ~640 MB until quit (64 KB/chunk,
  measured). Correct-as-designed and now *visible* (`host_edited` in the
  `DC_MEM_PROBE` line, asserted by a dc-api test), but the real answer is the
  decided save/persistence layer: spill an edited chunk and let the slot go
  back to being evictable. Sequenced behind persistence itself.

- **Distance-evict re-derivable resident data — a knob for later** (user, 2026-07-24).
  The world is massive; a player 100 km away is holding resident chunk + warm-LOD data
  that is **re-derivable** (no edits) and just taking space (S-2: store only what the
  derivation can't predict). *Untouched* `HostWorld.chunks` already evict (journal/0051)
  and the far pyramid has a budget (`FAR_PYRAMID_L0_BUDGET`); the ask is a **distance
  knob** letting unedited resident data (near chunks AND the warm-reduced LOD nodes) be
  reclaimed as the player recedes, while edited chunks stay pinned (→ the save-layer
  above). Not now — a knob to add later; couples to the LOD warm-cache lifetime the
  haunted-LOD diagnosis flagged as a needs-live-probe.


- **METAMORPHISM — the grade axis: `exhum` = P, `dc:field/temperature` = T → grade**
  (**UNBLOCKED 2026-07-24** by the geotherm, journal/0093; given its own Sequenced entry
  2026-07-25 by the staleness sweep, row D-1). *One job that had **three** ROADMAP homes — the
  layer-cake redemption's (b), consume-the-ledger's (b), and a close block — and whose only
  "sequenced" home was **a close block a wrap rewrite was about to overwrite**. Both surviving
  homes now point here.* The two axes exist and are read by nothing: **`exhum` is the pressure
  axis** and **`dc:field/temperature` (the geotherm, journal/0093) is the temperature axis**; a
  grade is the pair. Landing it **retires stubs #4** and **empties the `exhum` / `t_crust` row of
  spines § 3 "Built, and nothing calls it"** — the planes ship in `DeepField` explicitly labelled
  *"the metamorphic-grade axes the collapse tier reads"*, and have shipped unread since S12.
  Two things worth carrying into the slice: the geotherm entry itself named metamorphism as its
  **real payoff** (its effect on coal is ~surface-temp-thresholded, because burial is shallow —
  the deep crust is where a `T(depth)` field earns its keep); and **coal rank and metamorphic
  grade are one thermal-maturity ladder** (S-8, *one quantity, many names*), so this is the same
  machine as the coal-rank item and should not grow a second one.

- **Tectonic expression at the collapse tier — the layer-cake redemption**
  (promoted 2026-07-21 after the user's callout: dip/fold non-expression
  "slipped by without my understanding or ratification" — an integrator
  sequencing failure, acknowledged; the Observed LAYER-CAKE line was filed but
  never surfaced during the tectonics ratification). The ratified architecture
  always intended it: `DeepField.chapters` ships ~5 KB of plate state per
  chapter EXPLICITLY so per-unit deformation (dip, fault traces) re-derives
  analytically at collapse resolution — built in S12, consumed by nothing.
  One family, one slice-group, all UNGATED as of the U8 flip (landed 2026-07-21):
  (a) chapters → strata dip/fold/fault expression in cut faces (the
  "coal seam dead-ends at a fault" line of things-that-will-happen);
  (b) `exhum`/`t_crust` → metamorphic-grade classes — **this half now has its own
  Sequenced entry: see "METAMORPHISM — the grade axis" just above** (one job, three homes;
  consolidated 2026-07-25 by sweep row D-1);
  (c) drainage export (`recv`/`area`/`lake`) → the 3e-2 macro drainage
  consumers — **⚠ note 2026-07-25: those planes are now a DELETION target, not a
  consumption target** (FLOW slice 1 superseded them; heir `DeepField::flux`, disposal
  continuation (e)). What survives here is the *expression* need, not the input.
  Note: this is the CUT-FACE sin, not the silhouette sin — terrain
  shape flatness is the separate S13/roughness thread.
- **Consume the ledger terms the runtime throws away** (geology.md § Expression
  of the ledger, DECIDED 2026-07-21). Four concrete, independently shippable
  pieces: (a) *(**carry `H`: SHIPPED** 2026-07-21, journal/0053 — see `ROADMAP-history.md` § Shipped.
  Both consumers read the recorded plane; stubs.md § 3 retired whole.)*;
  (b) **consume `exhum`/`t_crust`**, which ship
  explicitly as "the metamorphic-grade axes the collapse tier reads" and are
  read by nothing — **now owned by "METAMORPHISM — the grade axis" just above**
  (2026-07-25, sweep row D-1: this was one job with three ROADMAP homes and no
  Sequenced entry of its own); (c) **derive material FORM** (loose / pore-partial / whole /
  inclusion) from provenance rather than leaving it implicit — sub-voxel facies
  express as inclusions (the charcoal sieve: 0 of 158 310 beds survived 0.9 m).
  **(c) is now the priority piece and much larger than the charcoal framing
  suggested**: journal/0053 measured the sub-voxel sieve deleting **~75 % of
  the whole sediment pile** (mean `H` 4.75 m, only ~1.15 m surviving as
  whole-voxel strata; the tour's dune field records 379 units summing to
  7.99 m and expresses *zero*). Filed as **stubs.md § 12**, heir named:
  amalgamate adjacent sub-voxel units **inside the record**, not in the
  veneer. It needs no user decision and rides the landed contents-authority
  machinery — the best next forms-adjacent slice;
  (d) **per-voxel provenance query** — read a voxel's ledger: started as X,
  heat/pressure did Y, moved because of Z. **RESOLVED 2026-07-24 by the
  commit-as-facts semantics** (material-behavior.md § Commit semantics DECIDED):
  provenance = `base + facts`, addressed to the material portion's lineage (a move
  is itself a fact that travels with it) — it falls out of the deep-cell inventory
  keystone and needs no separate design pass. The earlier "needs a design pass,
  PROPOSED not ratified" framing is retired.
- **Roughness recalibration — three costed candidates, USER PICKS FROM PICTURES**
  (S13 § 6, 2026-07-21; nothing flipped on). Binding constraint measured: the
  `s7_walk` seam test prints max interior step **2 of 6 voxels**, so there is
  **3.0× headroom** on the finest levels.
  - **A. Uniform re-anchor** `AMP_DECAY^(L−2)` (×3.31). Predicts 18/51/66 m at
    100 m/250 m/1 km at the walk site. Zero gen cost — but **measurably fails
    the seam test** (2 × 3.31 = 6.6 > 6); a ×2.5 variant fits with nothing to
    spare.
  - **B. Band-limited re-anchor**, boosting levels 6–10 by `[4.0, 3.4, 2.8,
    2.2, 1.6]`, unity from 11. Predicts 14/58/75 m. Zero cost, **seam-safe by
    construction**. Risks: 58 m over 250 m ≈ 23 % grade may hit character slope
    limits, and it is five hand-chosen numbers — *a fractal answer to a
    simulation question*.
  - **C. Refine the deep tier to 230 m cells.** ~62 s / ~210 MiB at Medium (from
    15.5 s / 52 MiB); needs `DEEP_MAX_WIDTH` raised and **breaks the ratified
    `pregen_time_vs_extent` < 60 s budget** (note: U3 already relaxed the ritual
    ceiling to "5 min if that's what it takes", so this may be cheaper than it
    reads). No predicted relief — S13 states plainly it cannot predict this
    without running it. **The only candidate that makes the *mountain* legible
    rather than the *ground*.**
  A and B garnish the ground; only C touches the 460 m–7.4 km band the `L_DEEP`
  override discards. Sequence the steep-site re-walk FIRST so the pick is made
  against the world's real terrain.
- **A 5th/6th far LOD level — for horizons past ~10 km** (found by the
  journal/0042 measurements, 2026-07-21). The 4-level ring scheme's honest
  ceiling is ~10 km / 172 MiB / 13.7 s to fill, because cost is quadratic
  inside the stretched L4. Journal/0023 already named the right answer — *add
  rings*, don't lengthen the last one. Only worth doing if the design target
  wants vistas past 10 km; note the haze limit below may bind first.
- **The forms/partials EMISSION slice** (user directive, journal/0049 station
  2; materials.md DECIDED 2026-07-21). **The forms DESIGN is now done —
  `material-behavior.md` §§ 2–3 IS that pass** (the closed form set structure/loose/
  pore-fill/fluid/void, the machine-complete transition graph, agents as folded
  rate-terms). What remains is the **emission slice**: world-gen emitting partials —
  sand as the first spawned loose material — which is a **rider on the deep-cell
  inventory keystone** (Crux 1), NOT a standalone design pass. The 0010 dormant loose
  renderer and S8 mechanics are the substrate.
- **Tectonics SPIKE** (per tectonics.md § SPIKE, architecture ratified
  2026-07-20). **[VALIDATED as FIELD-SOLVERS, 2026-07-24 (user-confirmed; north-star/
  material-behavior): the SOLVER half — tectonic deformation/uplift, drainage,
  climate, thermal — is a FIELD pass (compute+plant a field, native-core); do NOT
  recast *that* as cellular. But each has a cellular EXPRESSION pass that reads the
  field (ctx channel 2) and moves material — erosion/transport reads drainage; an
  **upheaval** pass reads the tectonic field. That split IS the design (material-
  behavior §5 dual-process split), NOT a recast. Same stamp covers erosion-supply,
  3e-2, and drainage below.]** implement `DeepConfig::tectonic_history` behind the flag and
  produce the eight measurement groups (clamp stability under ramped
  repaints, recorder growth vs K, ritual wall/memory at 200/300/400 iters
  reporting tradeoffs not optimizing to a cap per amended U3, landform
  evidence with pass bars + negative controls, forcing-wavelength kill of
  the 50 km artifact, byte-identity off, relief distributions for the
  re-sequenced amplitude call, drainage-export fidelity). **Dispatch after
  the eolian agent merges** — write-sets collide in `deeptime/erosion.rs`.

- **Retire `client_player_pose_set` into `dc:character/pose`** (API.md
  Decisions #5, ratified 2026-07-20): make the player a dc-api character
  and route the player controller through controller-verb commands — the
  work that makes player input replayable (decision 2). Explicitly
  sequenced AFTER the session-4 console and edges agents merge: it rewrites
  `mcp.rs` and `player.rs`, both in those agents' write-sets.


**NEEDS RATIFICATION (user-owned — this CHANGES TERRAIN SHAPE):**
2. **The contrast is currently modest at the shipped erosion amplitude** (relief
   +2 m aggregate) because the world barely erodes against its uplift; the
   headroom test shows the model produces real cliffs (44.7 m) the moment erosion
   is allowed to cut. Whether to raise `erodibility_contrast`, raise the global
   erosion rates (cause 3, "conservative amplitude"), or both, is the user's
   amplitude call — the lever now exists and is a knob.
   *(2026-07-22: the user noted they have never SEEN a cranked erosion
   budget — the headroom world exists only inside `erodibility_probe`
   experiment B; no launch path walks one. Note the term collision: the
   journal/0040 "amplitude call, answered neither" was TECTONIC amplitude
   (`thickening_scale`, --amplitude), a different lever. Enabling slice
   filed: extend 0040's deep-config launch plumbing with a dev erosion-budget
   flag (weathering / k_transport / k_bedrock ×N, optionally
   erodibility_contrast) so THIS call can be made from vistas on a same-seed
   pair.)*
   **ENABLING SLICE LANDED 2026-07-22 (journal/0076): `--erosion-budget <mult>`
   on dc-client rides the 0039 `DeepOverrides` door as `erosion_budget:
   Option<f64>`, multiplying `weathering`/`k_transport`/`k_bedrock` together at
   world build (experiment B's exact semantics; relative rates fixed). `1.0` is
   byte-identical to no flag (falsifier `erosion_budget_one_is_byte_identical_to_no_flag`);
   goldens did not move. The walk is UNBLOCKED — the amplitude call can now be
   made from vistas on a same-seed pair (`--erosion-budget 3` vs `10`). The knob
   is the erosion budget only; `erodibility_contrast` was left for a later slice
   (the budget alone is what experiment B moved). The DECISION is still the
   user's; this slice only makes it seeable.**
   > ## 🔴 THE BLOCK BELOW IS THE LOSING SIDE OF A MEASUREMENT AND IS STILL MARKED SETTLED
   >
   > *Banner added 2026-07-29 (baseline sweep S9/S9-1). **Nothing below is deleted** — its
   > sweep is empirically reproducible on the shipped world, and journal/0079's probe was
   > faithful to the launch path it tested. What it was **blind to** is `diffusion`.*
   >
   > **`journal/0111` (2026-07-26) measured the opposite on three load-bearing halves:**
   > 1. *"graded to base level … **neither supply-limited**"* → 0111: *"98 % of every metre of
   >    bedrock this world detaches leaves the land system… **at the shipped calibration the
   >    landscape is supply-limited** — the weathering constant *is* the denudation rate."*
   > 2. *"the rate only sets approach-to-grade"* → 0111's sweep: budget 100× **+ creep 10×**
   >    buys **132×**, *"59× more than their separate gains multiplied."* **`erosion_budget`
   >    scales `weathering`, `k_transport` and `k_bedrock` — it does NOT scale `diffusion`**,
   >    and creep does **918×** what the rivers do. *0079's null was an artifact of the knob's
   >    reach, not of grade.* **"A knob that cannot move the thing it is named after"** —
   >    corrections #56, `stubs.md` #24.
   > 3. *"relief is bottlenecked on the GENERATING side — not erosion"* → `journal/0114`
   >    measures relief **+4.6 % at 45×, +18 % at 100×, +52 % at 300×**. Erosion moves relief;
   >    it was never allowed to.
   >
   > **⚠ AND DO NOT RESTATE IT AS "SUPPLY-LIMITED TODAY."** 0111's finding holds of the
   > **calibrated** world; `EROSION_CALIBRATION = 45` sits behind `calibrated_rates`, **OFF in
   > production**, blocked on the incision clamp (`journal/0116`, `stubs.md` §§ 27/29).
   > `journal/0114` then refines 0111 in turn: *"the rates were never the binding constraint."*
   >
   > **WHAT THIS BANNER DOES NOT DO: reopen the axis.** That is a **user call**. What it
   > withdraws is only the closing clause — see the strike at the end of this item. *0079
   > opens by invoking the journal/0030 trap — "a null from an instrument that cannot see the
   > question is worth nothing" — and CLAUDE.md's "corollary for reading a null", written for
   > `journal/0110`, fits 0079 word for word and had never been applied to it. See also
   > `journal/corrections.md` #41, which carries 0079's mechanism.*

   **ANSWERED — NOT BY A WALK — 2026-07-23 (journal/0079; RATIFIED). The
   erosion budget is not the amplitude lever.** A faithful probe (1× asserted
   byte-identical to shipped `build_field`) swept `--erosion-budget` 1×/3×/10×/30×
   on the client world: relief **1287 m at every budget**, mean |Δsurf| vs 1×
   climbing only to **0.35 m at 30×**, strongest hard-bed contact gaining 3.5 m,
   lithologic separation unmoved. A mechanism probe pinned *why*: the surface is
   **graded to base level** — mean lowering **41 m (max 1.4 km), flat across the
   whole sweep** — so it is neither supply-limited (it erodes a lot) nor
   iteration-starved (flat, not growing); the rate only sets approach-to-grade,
   capped by `inc_pot.min(room).min(max_inc)` (`erosion.rs:1149`). Weathering ×30
   thickens regolith 5→13 m but does not lower the surface. This is the third leg
   with journal/0040 (tectonic amplitude bought no relief) and S13 (5 % of
   roughness reaches the ground; the summit is a genuine plateau): **relief is
   bottlenecked on the GENERATING side — deep-field elevation structure + uplift —
   not erosion.** Cranking erosion is pushing on a rope. `--erosion-budget` stays
   a dev tool (a faithful lever on regolith / approach-to-grade). **Cause 3
   "conservative amplitude" is REFRAMED, not retired: it is a deep-field
   relief-generation problem** (couples to S13's owed 100–500 m landform band and
   the plateau) — its own design pass, Sequenced below. Instruments committed:
   `examples/amplitude_tour.rs`, `examples/amplitude_mechanism.rs`.
   corrections #41. ~~**Nothing further to ratify on the erosion axis.**~~ **🔴 WITHDRAWN
   2026-07-29 (baseline sweep S9/S9-1) — see the banner at the head of this item. Whether the
   axis reopens, and on what terms, is the USER'S call and is not presumed here.**
3. **The four rate coefficients** in `resistance_of_material` (how smash /
   solubility / permeability / cohesion map to each agent's resistance) and the
   contrast/clamp defaults ride **plausible-not-tuned**, same status as S9's
   physics constants and S10's rate constants (no-bandaid). Engineering, rides
   as-built unless the user wants a different look.


**Far-field range knobs** (user-requested 2026-07-20 at the FF2a
ratification): expose the draw-distance geometry as adjustable settings —
`FULL_DETAIL_RADIUS_M`, the ring edges, `FAR_MAX_M` (today compile-time
constants in farmesh.rs) — so the user can push the horizon and feel the
FF2a scale-checkpoint numbers (~10 km ≈ 376 tiles / ~43 MiB, meshing stays
budget-bounded) instead of reading them. Small; a dev-console/config
surface question more than a rendering one; the ring-membership hysteresis
and coverage tests must hold at any setting.

**Sim light — what the voxels know** (user-sequenced 2026-07-20.
**DESIGN PASS DONE 2026-07-20 → `docs/design/light.md`**, which is now the
doc of record; visuals.md holds the dated DECIDED entries it consolidates.
**Next step is the SPIKE**, per the ratified shape design → spike →
milestone; § 10 of light.md states exactly what the spike must measure.)
This is a new sim field, not a renderer feature, and deliberately NOT
PBR-2.

- **What it is**: a deterministic, seed-driven, propagated per-voxel light
  level — Minecraft-blocklight-grade, "the simulation level of our light,
  not crisp dynamic shadows etc, just what the voxels know" (user).
- **Why it unlocks things**: it is the missing axis for photosynthesis and
  plant growth at the collapse tier (S10's biology gates on moisture and
  temperature but has no light term, and it is what makes caves lightless
  *in the simulation*); for creature/spawn behaviour; for stealth and NPC
  vision; and for **embodied-agent perception parity** — an MCP-driven
  character must not see better than a human player (visuals.md). It is
  also what makes "darkness is a gameplay material" true rather than
  decorative.
- **Renderer light and sim light are two different things with two
  different consumers, neither derived from the other** (DECIDED,
  visuals.md). The shader may do shadows/HDR/godrays; the sim carries a
  coarse level. They need not agree.
- **PRIOR — S3 already wrote the skylight query contract** (S3-results.md
  § Skylight query contract; also `column.rs` module docs). It solves the
  cubic-chunk problem (a cube chunk cannot know what is above it) without
  vertical scans: (1) a sky/light query consults ONLY resident chunks and
  cached column summaries — it never loads, generates, or walks "the
  column above"; (2) every query is bounded to a caller-supplied
  `[y_min, y_max)`, and no unbounded vertical scan exists anywhere;
  (3) **unknown volumes are non-occluding — "optimistic sky" — and poison
  the answer's `fully_resolved` flag**, and a consumer needing certainty
  schedules LOD derivation rather than forcing loads from inside a light
  query; (4) answers carry the resolution they were derived at. Column
  summaries build lazily at ~20 µs, so they are effectively free.
  **S3 OQ 6 explicitly defers a decision to this work**: "optimistic sky
  is a policy, not a truth... a consumer that ignores the flag will light
  caves as if open to sky until data arrives. **The lighting spike must
  decide re-light-on-load.**" That decision is this milestone's to make.
- **Open design questions** (answer in the design pass): resolution (per
  voxel? per column?); whether sky light and block light are separate
  channels (Minecraft separates them so a day/night cycle need not
  re-propagate block light — likely the same reason applies here);
  propagation/update cost on edits, which is the same dirty-rail shape as
  remeshing and collider tiles; and persistence vs re-derivation.
- **Noted structural parallel** (assistant, unproven): a propagated light
  field and water.md's bound-water saturation field are the same
  computational shape — a bounded local relaxation over the voxel grid
  attenuated by a per-material property (opacity vs permeability), with
  sources and sinks. S11 measured that relaxation as genuinely local
  (4–11 cell halo); the machinery may serve both. Check before building
  either twice.
- Per the knob doctrine: any propagation radius/level count ships as a
  knob, not a baked constant.
- **SHAPE RATIFIED 2026-07-20 (user: "go with your suggested shape"),
  mirroring how water went**: (1) a **design pass** producing the doc —
  including the heavenly-body *path* abstraction; (2) a **spike** measuring
  derive-per-chunk cost and proving the relaxation's order-independence by
  construction; (3) the **milestone**. No implementation before the doc.
- **Day/night needs no separate system** — it emerges from body paths, so
  the ephemeris IS the path primitive (user, 2026-07-20). Paths must be
  parameterised over world time AND observer latitude, with a long cycle
  that changes the path itself, so **seasons, poles and tropics can land
  later without a rewrite** — build the path capability now, not the
  features. `lat_deg` already exists on pregen cells.
- **Block light is unblocked from the item system** by a **dev-light block**
  (user): a point-light emitter that never spawns naturally, so propagation
  can be built and tested before torches are items. Plant light budgets
  deferred.
- Determinism requirement for the spike: light is a **max-plus relaxation**
  (`light(v) = max(emission, max over neighbours(light(n) − attenuation))`),
  monotone with a unique fixpoint, so run-to-convergence is
  order-independent **by construction**. An ordered flood-fill queue would
  diverge under replay/parallel — same family as S9b and S11.
- Note in favour of deriving: a stored model needs *un-propagation* when a
  source is removed (the classic stuck-light bug class in MC-likes); a
  derived model has no removal path at all — re-derive the halo and it is
  correct.

**PBR-2 — lit-world completion** *(**DEFERRED by the user 2026-07-20** — not
next, despite wanting it: "although i want this: it's going to gum up the
pipeline for the actual game and gen stuff. so not yet on pbr2." The
integrator's earlier "PBR-2 gates whether the underground is lookable at"
claim was based on a misreading of the 0027 walk and is **withdrawn** — the
underground IS currently lookable at; one face orientation is black
everywhere, surface and depth alike, and depth attenuates nothing. The
darkness TARGET is DECIDED — visuals.md: real darkness underground,
reference modded Minecraft with shaders — but its renderer half waits.
**SIM light is sequenced ahead of it** (below): it unlocks gameplay, PBR-2
polishes appearance.)* (the deferred half of the renderer, opened by
PBR-1 shipping): sun **shadows** (Bevy cascades + our knobs, the chasm-shaft
signature shot), **colored point lights** (lava/forge/bioluminescence via Bevy's
clustered path), **tonemap/HDR** (`Camera::hdr` + exposure — real darkness and
earned firelight), **POM** (the LabPBR height channel is already packed),
**light shafts/godrays**, **water**, and **weather/wetness** coupling
(sim-driven porosity darkening — the specular B channel is already sampled).
The `opaque.terrain` shader-pack HOOK (PIPELINE.md § 5, format ≥ 1) also lands
here: PBR-1's material follows the § 5 texture + vertex-attribute contract but is
not yet runtime-overridable via a pack (needs a `HOOK_FORMAT` bump + prelude
composition, like the `post` stage). Never: GI (doctrine).

**S10 follow-through — the biotic layer's five open calls** *(the spike
itself SHIPPED 2026-07-20, journal/0025 + docs/spikes/S10-results.md; it
answered the gate question — all four signals present and legible, cost
+10 s once and +6 MiB kept. What remains is **user decisions**, and one
implementation slice.)*

**NEEDS RATIFICATION (user-owned, from S10-results.md § Recommendation):**
3. **Signal abundance is aesthetic, not correctness**: coal 0.55 % of
   columns, paleosols 22.9 %, charcoal 20.2 %, retrogression 23.5 %. Is a
   fifth of the world carrying a fire record the texture we want?
   *(Amended 2026-07-20, journal/0026 — the question is now narrower than it
   looked. Measured against the 0.9 m voxel, the **fire record is entirely
   invisible** to a player (0 of 158 310 beds survive quantization), so the
   "fifth of the world burning" texture does not reach the eye at all today.
   What DOES reach the eye is organic **soil**: 185 km of surviving thickness,
   making carbonaceous mudstone the second most abundant rock in cut faces.
   That, not charcoal, is the appearance call to make.)*
   **⚠ THE NUMBERS ABOVE ARE HISTORICAL — measured 2026-07-20, on the reference seed
   (2026-07-25, sweep row S-8).** Coal has moved **twice** since they were taken: the
   susceptibility blend made near-surface coal recessive (journal/0072), and the geotherm
   re-sited it onto warm crust (journal/0093). On the world the client actually boots
   (**1337 / Medium**) coal is now **0 % — 0 units across 297,025 cells** (corrections #51),
   not 0.55 %. **Re-census before treating any of these four figures as an appearance call.**
4. **The 7-species roster and ~25 rate constants** ride as
   plausible-not-tuned (no-bandaid: they are the mechanism's calibration,
   not a patch) — same status as S9's physics constants.
5. **Pack-addition blast radius** (ecology.md § 5 fork 2) **has stopped
   being hypothetical**: biology is now demonstrably an erosion term, so
   adding an organism pack would change *terrain*, which materials packs
   never did. The doc filed this as "needs deciding before biology couples
   to erosion" — it now does. Options unchanged: bake biotic erosion at
   world creation (pack-add affects ungenerated regions only, matching the
   DF-like seed policy) or accept landform drift.


**Coal rank when burial deepens** (journal/0026): `dc:stratum/organic-coal`
ships one member because rank transitions (lignite → sub-bituminous →
bituminous → anthracite) live at ~1–2 km of burial and our deepest recorded
overburden is ~100 m — a rank ladder now would be a discriminator on an axis the
data never visits. The class's **depth axis is the rank axis** and the
class-share invariant means members can be added later without moving an
existing seam. Revisit when the record carries kilometres (3e-later / thicker
basins).
**⚠ RE-BLOCKED 2026-07-25 (sweep row S-5) — the stated blocker is no longer the real one.**
Coalification moved onto a **temperature** axis when the geotherm landed (`COAL_ONSET_C`,
journal/0093): `promote_coal` now reads the geotherm at seam mid-depth against an onset
temperature, so *"the class's depth axis IS the rank axis"* is no longer the whole rule —
P and T are separate axes and rank is the pair (same thermal-maturity ladder as metamorphic
grade, S-8). And the harder half: on the shipped world (**1337 / Medium**) there is **0 coal**
(corrections #51), so a rank ladder would have **nothing to discriminate on any world a player
can open**. Revisit after the coal-content call (a)–(d) in Observed, not after the record
carries kilometres.

Also filed from S10 (not blocking): parent-material phosphorus from pregen
provenance instead of a uniform pool; individual plant placement from the
community vector by addressed hashing (ecology.md's derivation-chain
terminus — note the community vector is currently **dropped** after the run,
so C-refinement would have to re-derive it); vegetation → channel planform
("vegetation invented meandering rivers") needs 3e-2's finer corridor to
have any planform to bend.

**3e-2 — C refinement** (DECIDED 2026-07-19 — earth-processes.md § 3e-2
decisions. **RECONCILED 2026-07-25 (user) — the three contradictory stamps on this
entry were never actually fighting; they were stamps on different clauses. The
record of decision is earth-processes.md § 3e-2's `⚠ SUPERSEDED IN PART` banner,
which already said all of this; this board was the stale party.**): drainage
coarse-at-A ~~with the river-conditioning mechanism (corridor wander toward refined
lows + descent-along-flow as hard constraint, no divide-crossing)~~; width cap
permanent; 0015-mechanism elevation stitch + interior-commit records over
the 16–24-cell halo; proximity approach trigger, order-independent by
construction; **contact-softening in scope per method rule 5** (no
grid/analytic boundary reaches the eye); calibration RATIFIED — the
Phanerozoic register (~500 Myr recorded span, basement ages procedural
— "procedural hacks for the boring billion"; knob deferred).

- **WHAT SURVIVES — decisions 2, 3, 4, 5 untouched, plus decision 1's FIRST
  clause.** None of the width cap, the stitch, the approach trigger or the
  calibration is about how a river is *made*, so FLOW does not reach them. And
  *"drainage is advective and decided-once-coarse"* stands on the halo theorem
  (corrections #8) — that is precisely the clause the **2026-07-24 sweep stamped
  VALIDATED** (*"drainage is decided-once-coarse and is the sole advect"*).
- **WHAT DIED — decision 1's SECOND clause, the river-conditioning mechanism.** The
  wandering channel **line** with the refined surface **nudged** around it is a
  *drawn* carve: an operator that deforms terrain to imitate a result. flow.md § 3
  forbids exactly that — **the channel is what REMAINS when the material the erosion
  passes actually moved is subtracted along the recorded path**, an expression of a
  mass budget, never a deformation. (flow.md § 6 lists § 3e-2's expression half under
  "What this retires".) Refinement must now get its channels from `DeepField::flux`.
- **WHAT NARROWED — the no-divide-crossing rule binds the FREE/surface regime ONLY.**
  Not a supersession but a correction: **bound (groundwater) flow genuinely crosses
  surface divides** — artesian basins, karst piracy. Carried over unqualified the rule
  **forecloses regional groundwater**. Also per flow.md § 2.4 the driving field is
  **potential/head**, not elevation — and `dc:field/head` now exists (journal/0098).
- **🔗 SEQUENCING FACT, recorded 2026-07-25 because it was written down NOWHERE: 3e-2
  is now DOWNSTREAM of FLOW.** It was free-standing when ratified; it no longer is. It
  cannot be dispatched until the flux record can supply channels at the refined tier,
  which puts it behind MFD and Movement 2b. *The four surviving decisions remain
  implementable in themselves — it is the drainage-expression half that has acquired a
  dependency.* Do not dispatch this as "nothing open" work.

**S11 follow-through — the water model's four open calls** *(the spike itself
is SPIKE COMPLETE 2026-07-20, journal/0028 + docs/spikes/S11-results.md; it
answered the gate question — bound water's halo is 4–11 cells and the body
graph does not grow with edits. What remains is **user decisions**.)*

**NEEDS RATIFICATION (user-owned, from S11-results.md § Recommendation):**
1. **The two body characters — `Finite` vs `Pinned` — as a design commitment.**
   This is the load-bearing one and it is not a performance question. Pinning
   is the claim that *some water has a level set by the world rather than by
   its own volume*. The measurement says the distinction is necessary (a finite
   sea drops 13.18 m when breached into a large void) and that pinning is also
   ~415 ms cheaper per resolve. Seas and fed river reaches obviously; **where
   the boundary sits — a big lake? a spring-fed pool? — is the user's call.**
2. **The ~12-cell bound-water derivation halo**, if it becomes a constant. Wide
   margin over everything measured, but it is calibration on a model whose rate
   constants are plausible-not-tuned — same status as S9's physics constants and
   S10's rate constants (no-bandaid: rides as measured).
3. **Body identity is not stable across a merge** (S11 design choice 2): bodies
   in one air component merge, lowest id surviving. If lakes are ever to be
   named, findable, or referred to by quest/ledger state, that needs deciding
   **before** the graph ships.
4. **Whether the connectivity index is truly never persisted.** The spike
   asserts it is derived and proves reload identity from it (6 ms rebuild for a
   1.77 M-voxel world), but at real world scale that becomes a streaming cost
   nobody has measured. The alternative — persist it beside the S3 region files
   — is the *same* open question the far-field summary store carries, and the
   two should probably be answered together.

*(Not decided by S11 and still open in water.md: the bulk-flow octree — though
note the spike found free water in equilibrium is **static data with a level**,
which is what the notebook's "creates no new blocks so long as its outlet
connects" predicted; sub-resolution water; capillary action; which cave family
ships first; and where the deep-time water field lives.)*

**⚠ READ `docs/design/flow.md` FIRST (ratified 2026-07-25) — this entry has been half
delivered and half superseded (2026-07-25, sweep row S-4).** `water.md` itself now opens with
a supersession banner. **The river / drainage / channel-expression half is superseded.** The
**groundwater** half is **partly delivered at the DEEP tier** by `dc:field/head`
(journal/0098): transmissivity, vertical conductivity and **confinement** are derived from the
strata record's own permeabilities — *a marine mud over a fluvial sand **is** a confined
aquifer, with no landform code path* — and **artesian occurs naturally** (60 columns, max
excess 2.94 m). Recharge is still open as **stubs #19**, so those excesses are metres rather
than the hundreds a real basin gives. **What this pass still owes is the PRESENT/RUNTIME
tier:** visible and flowing water, ponds and sub-resolution water, speleogenesis, and the
free-water body-graph coupling. Caves ride **FLOW continuation (c)**.


3a. Form archetypes + drop distributions (materials.md § forms) — implement
   with the first inventory/interaction milestone.
3c-2. Geology client integration, material tier (visuals DECIDED
   2026-07-19, visuals.md § mixture road): sidecar channel through the
   seam + interim world-anchored dither materialization (registry albedos,
   eighths-weighted), partial-height loose-material rendering with binary
   threshold collider; subtle ore. Runs after body steps 1–2. Watch the
   order-dependent MixtureTable ids at the seam (journal/0008).
   **(SUPERSEDED 2026-07-22 by block-is-material):** there is no separate block
   registry — block IS material (one namespace), so the atlas/enum is fed by
   **material definitions**, never a `define block_type`. A1 (journal/0087) already
   routes the render through material identity and deleted the block twin.
3d. Geology post-v1 slice (geology.md §§ formation-context +
   roster/inclusions/slots, DECIDED 2026-07-19): strip climate windows from
   igneous class sheets (formation-context shim correction — fitness goes
   province/depth-driven); chunk-line family cutover diagnosis + smoothing
   (context interpolation / boundary dither); two-layer class-satisfiability
   enforcement in pipeline validation; accessory inclusions as pore
   partials; then the rich vanilla mineral roster. Carbonate follows.
3e. Epoch-indexed formation context (the deep-time turn): paleo-context
   axes from pregen history feeding deposition/metamorphism; igneous from
   tectonic setting + emplacement depth. Orogeny-repo recon 2026-07-19 as
   idea quarry; couples to S2 checkpoint-facts design (Observed).
4/5. **Ecology + organisms** — designed 2026-07-19, docs/design/ecology.md
   (supersedes the old biomes/ecology seeds and the agenda doc): species
   is the primitive, biome is a diagnosis; the six-process biotic layer on
   the deep-time tier; biology as an erosion/weathering/rock-forming term;
   a "biome pack" is an organism pack. Evolution recorded as the user's
   teleology design (org defs pin a form at a horizon; the lineage between
   is simulated; ahistorical override; fossils = mid-horizon pins) —
   post-v1, but v1 must not foreclose it.
   - *(**S10 biotic-layer spike — the gate — PASSED** 2026-07-20,
     journal/0025 + docs/spikes/S10-results.md: community vector + all six
     processes on the A-tier, all four read-quality signals present and
     legible, cost +10 s once / +6 MiB kept, determinism intact. Evolution
     stayed out of scope as specified. The substrate evolution will ride is
     proven; the five ratification calls are above.)*
6. Body plans implementation staircase (docs/design/bodies.md § staircase,
   ratified 2026-07-19) — six steps from joint-tree skeleton to authoring
   editor; not yet scheduled against the geology track.

## Observed (undiagnosed or deliberately unfixed)

- **🔴 THE FOOT-IK CANCELS 92 % OF THE DERIVED SWING CLEARANCE, AND INVERTS THE KNEE PROFILE —
  DIAGNOSED 2026-08-04** (`docs/audits/2026-08-04-longleg-swing-probe.md`; found by the user at the
  station-2 walk: *"the longer legged one has a little weirdness in the forward swing… a mini bend
  and correction back to straight and back to bent"*).
  - **Mechanism, measured.** `character.rs:313`'s seated guard (`adjust.abs() > 1e-3`) sits on the
    two-bone solver's **singular point**: the posture bake stands every body at chain reach, so a
    grounded sole is at `d = reach` *exactly*, where `dφ/dd → ∞`. The gait's sole crosses ±1 mm
    **six times a cycle**; the decision flips `Seated ↔ Applied` at each crossing and the knee jumps
    **5.02°** (longleg) while **the sole does not move** — rendered envelope ±1 mm, but the knee
    joint translates **21.8 mm in one sample**. Closed form predicts 5.17°, measured 5.02°.
  - **The bigger number is not the artifact.** The gait derives **+12.66 mm** of mid-swing
    clearance; **+0.99 mm** is rendered. The IK also **inverts the knee profile** — the gait's knee
    is a triangle most-bent at mid-swing, the rendered track is a **W** that reaches **−0.5°
    (nearly straight) at mid-swing**. *The rendered gait is not the derived gait.*
  - **Ruled out with numbers:** the half-voxel window by **33×**; B7's `d_min` by **5.2×**
    (`BeyondFlexion` never returned); quantization is **not the cause but is the amplifier**
    (5.02° continuous → 12.26° @60 fps → 19.21° @12, and it decides how many of the six crossings
    survive: 4 @60, 2 @12).
  - **⚠ THE COMPARATIVE DID NOT REPRODUCE, and the probe said so.** Stout's jumps are **larger**
    (7.31° continuous, 20.74° @60). The phenomenon is **universal, not longleg-specific**; the
    probe could not determine from geometry why the user saw it on one body. Its inference —
    legibility (longleg's swing lasts 0.52 s in 31 poses against stout's 0.29 s in 17, with 2.3×
    longer bones) — is **flagged as inference, not measurement**.
  - **Speed-invariant** across the walk band (<0.03 cycle drift while the period changes 24 %),
    which is what rules out aliasing. Above the walk/run transition the root stops bobbing, the
    sole never goes below ground, and the jump collapses **31×** to 0.16°.
  - **NOT DIAGNOSED: why the IK corrects a SWING foot toward the ground at all.** The hypothesis
    that ground-following belongs to the stance foot only is the **integrator's, unmeasured**.
    That is the next probe, not the next fix.
  - **Not sequenced as work** — the user's verdict was *"it's non blocking at any rate."* Banners
    stamped on `stubs.md` #43. Probe retained: `crates/dc-client/src/swing_probe.rs`, gate cost
    **0.14 s**.

- ~~**ONE COUNT, TWO VALUES: the record's total unit count**~~ **✅ RESOLVED 2026-08-04
  (S0 § g): STALE, NOT WRONG — the settled count is 7,304,581, measured by BOTH
  instruments in agreement** (including the one that produced 10.95 M). The drop
  decomposes: slice 2's identity rederivation −30.4 %, slice 3's sub-quantum carry
  −3.4 %, drift −0.8 % (journal/0141's 7,622,541 at slice 2b is the dating middle
  figure). All artifacts stamped (0136, 0141, the correlation + clock audits, the
  fluvial priors). *Historical body: journal/0136:280 said 10,951,030; the correlation
  design said ~7.4 M; a 48 % gap with neither pointing at the other — doc-topology
  shape 4, caught by a third audit reading both.*

- **LATERAL QUANTIZATION OF MATERIAL IDENTITY IS UNRULED, AND THE CORRELATION DESIGN
  ASSUMES IT** (user, 2026-08-04, in discussion — deferred to next session, written here
  because deferred means written). The correlation design's M-B/M-C "cut" is a **per-voxel
  winner-take-all draw**: one identity per voxel, inverse-CDF against the blended shares.
  The user challenged it against the ratified **distribution-first expression** rule
  (`materials.md` DECIDED 2026-07-21 — integrate the column in metres, then slice; partials
  survive to the voxel boundary; beds in the fixture world render **mixed**, not snapped to
  whole voxels): *"i disagree that we want to dither on the full-voxel scale for materials
  in our pack. we have ratified against that previously."*
  - **The precise gap:** that ruling was made about the **VERTICAL** case (a bed ending
    mid-voxel yields a fractional voxel). The cut is **LATERAL** quantization (two
    materials sharing a bed at one depth, one winning the voxel) — technically unruled.
    **Whether the principle extends laterally is the user's call**, and the integrator's
    reading is that it must: not quantizing a contact crossed going down while quantizing
    one crossed going sideways is an axis-dependent inconsistency.
  - **The alternative construction, assistant-proposed, not ratified:** use the octaves to
    shape the **share field** rather than to pick a winner — the local shares wander around
    the smooth trend with the variogram's texture — and let every voxel express its local
    shares through the existing eighths machinery. Inside a body the field goes near 1.0
    (pure voxels, no fictional half-rock); at a body's margin it passes through the middle
    (mixed voxels, honest at 0.9 m); a true gradational pair renders as mixtures throughout.
    The mixture-vs-cut *branch* then becomes a continuous parameter — **how sharply the
    share field is shaped** — still derived from what the materials are.
  - **If ruled, it revises `2026-08-03-stratigraphic-correlation-design.md` § 3.2** (banner
    owed there) and adds a sentence to the walk criterion: read **margins**, not contacts.
    **S2 must not wire the winner-take-all cut before this is settled.**

- **E4-2's MEASUREMENTS SAY K2 IS SLOWER THAN WHAT IT REPLACES — a decision is owed**
  (2026-08-04; `docs/audits/2026-08-04-e4-2-convergence-study.md`, rescued to main; code
  unmerged on `worktree-agent-a787b0d7fdeb61b53`). Monotonicity is **not** broken by the
  limiter-as-projection (that chain is monotone to a = 1e8, with a closed form) — it is
  broken by the **Picard relinearization at a measured a = 2.0**; the *binding* bound is
  the **anisotropy** one at **0.85**, where the arm runs **2.2× slower than explicit** at
  `picard = 4`; and `picard = 1` is not an escape (it overshoots the max principle by 150 m
  on a supply-limited chain). **Fork for next session: re-price K2-alt vs K3 on these
  numbers (U-1 pre-committed to neither) or park E4-2.** § 5's world runs are unrun.

- **🔴 THE BUILD-MUTEX HOOK SILENTLY DISCARDS NON-CARGO WORK BUNDLED WITH A DENIED CALL —
  DATA LOSS, not just a stall** (E4-2 agent, 2026-08-04, caught by proofreading): when a
  Bash/PowerShell invocation contains a cargo command **and** other commands (an edit, a
  file write, a python heredoc), a denial discards **the whole invocation** — the non-cargo
  side never runs, and nothing says so. The E4-2 agent lost two python edits this way and
  found them only by re-reading its own files; **one would not have compiled.** Interim
  rule for every agent and session: **never bundle an edit with a cargo call** — separate
  invocations, always. Fix shape (hook-owned, unowned): deny with a message naming what was
  discarded, or scope the denial to the cargo command alone. Sibling of the text-match
  false positive below; same file, same afternoon, both found by use.

- **THE BUILD-MUTEX HOOK DENIES NON-CARGO COMMANDS THAT MERELY MENTION CARGO IN TEXT**
  (geo session, 2026-08-03, hit live): a `git commit` whose **commit message** contained
  the word "cargo" was denied as a build-slot claim (`scripts/cargo_mutex_hook.py` —
  evidently a substring match over the whole command string, which includes quoted
  message text). Cost: one reworded commit; the failure mode is worse for wrap commits,
  whose messages routinely cite gate commands. Fix shape: match the command's leading
  token(s)/invocation position, not the raw string. Small, mechanical, hook-owned;
  owed to whoever next opens the hook file.

- **FIELD REPORT, slice 3's interfingering, LIVE VIEW (user, 2026-08-03, during the
  bodies session's game session — formal walk verdict pending, no pose yet, corrections
  #48 gap to close at the acceptance walk): "as predicted, i do not like this,
  visually."** Two halves, both mechanism-confirmed at desk: **(a)** the contacts are
  still SHARP — *"sharp differences between regions, they just don't follow the cardinal
  directions"* — because the membership dither assigns each column exactly ONE deep cell
  (forced by F2's mass coupling: record + regolith H + ledger must name the same cell),
  so the contact is column-granular, jagged instead of straight, never gradational;
  **(b)** worse and load-bearing — *"the difference is not only at the surface: it
  extends all the way down the column. So we have this finger of one material
  distribution running through another, and it has an entirely different stratigraphy"*
  — a finger is a full-depth transplant of a neighbouring cell's whole geological
  biography, where real facies interfinger BED-BY-BED. This is the slice-3 audit's F6
  risk realized (the near dither inherits the far tier's cell-wide-blend semantics the
  user rejected 2026-07-29, "the whole cake is swirled"); P-6 said the walk is the
  ratification and the live verdict is trending REJECT. **The design question it opens:
  gradational bed/event-level contacts WITHOUT breaking the F2 mass coupling that
  forced one-cell-per-column. A design conversation, user-called; do not build against
  an assumed answer.** Neighbours: the far-register heir (refinement-operator budgets),
  MM-3/SubCell (the shipped mechanism), the acceptance walk (P-7, owed).
  - **STATION RECORDED 2026-08-03 (geo session, in the kept-open game; corrections-#48
    gap closed):** contact zone at the x≈82,483 m cell edge on station 1's transect —
    mudstone-presence flip bracketed 82,400→82,480→82,560 m at z = 13,334 m (a
    four-material mix appears at the contact and vanishes 80 m to either side). Bench
    cut voxels (91620–91690, 238–264, 14816–14830). **Wall pose: feet (82490, 215.5,
    13346) m, yaw 0, pitch +0.12** — assets `0149-interfinger-contact-wall.png` (+
    `-benchcut` rim view). **And the wall sharpened the finding: the contact is
    recorded-record vs UNRECORDED BASEMENT at the same elevation** (left: banded
    sandstone/conglomerate strata, `has_contents: true`; right: `dc:stone`,
    `has_contents: false`, floor-to-rim razor-vertical) — the membership dither
    transplants not only stratigraphy but **record EXTENT**, so a column boundary can
    be a full-height cliff between "the world has history here" and "the world has
    none", which no bed-level blending of identities alone would dress. The design
    conversation must cover the record-depth discontinuity, not only the mix.
  - **THE DESIGN CONVERSATION OPENED 2026-08-03 (user), AND ITS ANCHOR IS A USER SKETCH
    + A USER PRINCIPLE, both verbatim in the dispatched design pass:** *"a deep cell is
    a construct… wouldn't we just join layer to layer across boundaries? layers created
    at the same time in two different deep cells should… blend? find a midpoint and
    smoothly grade their thickness, as well as whatever the physical drivers etc does
    to facies"* — stratigraphic correlation; deep cells are boreholes 460 m apart. And
    the principle: *"ideally our default case, barring any physical drivers, is utterly
    smooth interpolation between all deepcell boreholes. **non-smooth detail is
    refinement content, and for the default deepsim plugin pack, it must model a
    process honestly.**"* Smoothness is the unearned default; sharpness is bought by an
    honest process — the positive half of the 2026-07-19 "I'll scream" ruling. **Design
    pass DONE 2026-08-03 (`docs/audits/2026-08-03-stratigraphic-correlation-design.md`,
    harvested same day).** Recommended: R-C, the shared-clock proportional rule (chapter
    partition + cumulative-thickness fraction; arity-free, chunk-seam-free; pinch-outs
    and truncation wedges fall out). **The mass argument HOLDS** (blend-of-sums exact in
    f64; F2 coherent; and the key simplification — correlation is READ-side, so
    quantum/carry/merge-key are untouched and every deep-time golden must be STILL, the
    strongest tripwire). Retires: `cell_of`, the near `sample_source_cell` call site,
    the `NearRecordMembership` domain, F6/P-6's framing. Survives: `SubCell` + records
    (the boreholes are the input), `cell_bundle`, the accessor layer, `Octaves`, the
    packed unit (chapter bits become the correlation clock). **User picks pending in its
    § 9: P-1 identity treatment (mixtures vs octaves-cut vs hybrid — carries the
    two-user-rulings tension: 2026-07-19 "dress every contact with noise/dither" vs
    2026-08-03 "utterly smooth default") · P-2 the inverted acceptance criterion · P-3
    R-C vs R-C′ · P-4 the onlap feather at the record edge (the 0149 wall becomes a
    wedge) · P-5 formal supersession ratification.** Side finding fixed in the harvest:
    two `recorder.rs` doc comments still claimed M-2 while the code ships M-1.

- **🟢 RESOLVED same night — the bodies session's arc gate (`0e344ca`) caught and fixed it
  as its defect #4 (a hand-maintained pinned tool list vs their `character_clear_look`
  addition; full workspace 962/0 after). Exactly the batching ruling working as intended:
  the debt was recorded at the time and collected at the arc gate.** ~~**INHERITED RED ON
  MAIN — `dc-client` `mcp_character::tests::character_session_is_embodied_and_attenuated`
  (filed by the geo session 2026-08-03; belongs to the BODIES thread).**~~ Assertion
  `left == right` fails at `mcp_character.rs:444`, **verified on clean main** (single-test
  run, compile line cites the main checkout — not a worktree artifact). Entered via the
  2026-08-02 `[UNGATED]` bodies-thread commits (gravity step 1 / gait work; exact culprit
  undiagnosed from the geo side, and the assertion's *intent* is bodies-session design
  territory, so geo did not guess a fix). Its sibling — a non-exhaustive grant match in
  `dc-host` missing `Payload::ClearLook` — was fixed in-flight by the slice-3 builder and
  merges with slice 3. Until this red is fixed, **every full workspace gate on main runs
  `--no-fail-fast` with exactly this one documented red**; a second red is a real defect.

- **OWED-SMALL from the 2026-08-02 geo session (deliberately not done, reasons attached):**
  the sweeps' unapplied minor findings — doc-topology **F7** (stale `GOLDEN_SURFACE` hex in
  4 docs: dated qualifiers owed, do NOT update the hex), **F9's § 4 re-cut** of
  `dependency-graph.md` (rows were refreshed; the startable-today list re-cut is partial),
  **F13** (graph's "last full pass" date — left honest at 2026-07-29 since no full graph
  pass ran), staleness **F4** (ROADMAP :3203/:3447 dither notes — mechanism closed for deep
  history by P11, veneer heir = slice 3) and **F5** ("vanilla" mentions grew 162→177 in
  crates/ as default-pack prose; the strike-'vanilla' trigger has fired — a sweep-and-rename
  pass is owed, ~~cheap, mechanical~~ **and the debt ACCELERATED while owed — 2026-08-03
  staleness F8: crates 177→215, docs 56→89, and FS-A shipped a source file NAMED for the
  struck word (`crates/dc-core/src/materials/release_vanilla.rs`), so the rename now
  includes a file path; still mechanical, no longer cheap**). Pointers: `docs/audits/2026-08-02-doc-topology-sweep.md`
  / `2026-08-02-roadmap-staleness-sweep.md`. Plus one from the erosion split (journal/0139):
  **broken rustdoc intra-doc links** where doc comments reference symbols no longer in
  module scope — `cargo doc` warnings only, invisible to the gate; fix if the corpus ever
  gates on `cargo doc`, or opportunistically when those files are next open.

- **FIELD REPORT, the P11 walk (user, 2026-08-02, post-walk):** two halves, opposite
  signs. **(a) POSITIVE — "the chunk-sized class-member quantization appears to be gone,
  naturally."** The 28.8 m member stepping that drove journal/0129's octaves work and
  P11 itself is no longer visible to the user's eye — corroborates slices 1–2 at the
  appearance level. **(b) OPEN — "there are larger regional borders still… a straight
  line where mudstone is in the mix on one side, and not on the other."** CANDIDATE
  mechanism (integrator hypothesis, NOT diagnosed): the **~460 m deep-cell record tile**
  — every column currently reads exactly one deep cell's record, so per-cell member
  presence produces ruler-straight borders at cell boundaries; this is exactly what
  P11 slice 3's per-column cell-membership dither is sequenced to dissolve (ruling 5,
  "the ~460 m near tile dies here"). The cheap discriminating check when someone is at
  a border: is the line aligned to the deep-cell grid pitch? **→ CONFIRMED AT DESK
  2026-08-02 (geo-2 session), no probe needed** *(citation corrected same day by the
  slice-3 design pass, F1: the record read is `record_at_voxel` at `collapse.rs:1483-1487`,
  resolved NEAREST by `field.rs:836-844`; `:1459` is the provenance half of the same
  chunk-centre addressing)*: one `run_strata` per chunk skins all 1024 columns from the
  chunk-centre's nearest deep cell, so the recorded mix is uniform per chunk and
  a member-presence border can ONLY fall on a deep-cell grid line, quantized to 28.8 m
  chunk edges — ruler-straight at ~460 m pitch, exactly as observed. Nothing else in the
  near path moves recorded-mix composition (the veneer dither selects among members a
  span already holds; it cannot add or remove mudstone). **This observation RESOLVES
  into slice 3's scope and its acceptance criterion** — the mudstone-mix border at a
  cell edge must stop being a straight line; the slice-3 design pass (in flight same
  day) drafts the criterion + its instrument. **→ ANSWERED BY THE USER'S OWN LIVE VIEW
  2026-08-03 (stamped by staleness F13): slice 3 shipped, the ~460 m tile is dead in
  code and the ruler-straight border is gone — but the dither that replaced it was
  REJECTED at the same view (journal/0149); the border question's live successor is
  the stratigraphic-correlation build (its § 7 inverted acceptance criterion is this
  entry's criterion, generalized).**

- **GRADE AND FORM LEGIBILITY — how does the player KNOW what they're looking at?
  (user, 2026-08-02, raised at the U4 ruling; explicitly ruled NOT to bear on U4.)**
  Two nested opens, the second older than the first and never previously written down:
  **(a)** the five grain grades just ruled (scree/gravel/sand/silt/clay) need a
  presentation answer — what tells the eye sand-grade from silt-grade on a surface;
  **(b)** the general form question — how does a player know they are looking at
  LOOSE versus STRUCTURE of a material at all? **The user has thoughts; this is a
  user-led design conversation to schedule, not a slice, and nothing should be built
  against an assumed answer.** Collides first with FS-A's walk acceptance (a stripped
  upland vs distal basin cut face is only a verdict if the grades READ). Neighbours:
  `visuals.md` (SPLAT_N, palette), the appearance cluster, the octaves-member-field
  surface-consumer question (this list's own entry above).

- **THE OCTAVES MEMBER FIELD IS INVISIBLE AT EVERY SURFACE OF THE SHIPPED WORLD — the
  U3-dominance verdict is now a DESK question** (2026-08-02, the P11 walk's tour map —
  journal/0143; probe `examples/appearance_tour_p11.rs`, gated). 0 of 4,792 sampled land
  columns carry a `Single` top span in a multi-member class; 99.1 % of surface top spans
  are `Mixed`, whose expression uses the undithered member by design. Control proven:
  the identical classifier finds 343,625 movable spans BURIED (73.6 % of land columns),
  and every both-members case is andesite/basalt — albedo twins. So journal/0129's "the
  member dither is inert at U3's pose" was never an unlucky pose; it is universal. The
  open question (undiagnosed, deliberately unfixed): what surface consumer, if any,
  should the octaves member-selection field get — or does its value live underground
  (mining faces, caves, cuts) where the spans actually are? Sequencing input for the
  visuals road and P11 slice 3's per-column work, not a defect.

- **🟠 GEO-ARC FINDINGS FROM THE 2026-08-02 SPINE-AUDIT (FULL, read at `3cf8778`) — filed by the
  BODIES session for the geo thread, which had already wrapped when they landed.** Full record:
  `docs/audits/2026-08-02-spine-audit-full.md`. Nothing applied; the arc adjudicates.
  - **F1 🔴 (verified at source by the integrator before filing): `outcrop_shares` lost a SECOND
    customer and three comments still describe the call.** Slice 2 replaced the
    `outcrop_shares(&[])` empty-section call with a direct `bedrock_axis = [axis.basement_slot()]`
    / `bedrock_sp = [1.0]` (`erosion/transport.rs:174-175`, `erosion/weathering.rs:419-422`).
    **`grep -rn "outcrop_shares(&\[\])" --include=*.rs crates/` returns ZERO at `3cf8778`** —
    confirmed independently. The value is identical; the *claims* are not: `transport.rs:356`
    still says *"Nothing is named here"* and `weathering.rs:419` still says *"the same walk asked
    with an empty section."* Anti-shape **A-2**, and it also kills **A-7's own worked-forwards
    instance** (`spines.md:2066-2072`), which now describes deleted code. **This is corrections
    #90's rule landing twice in one slice** — the slice caught the rate customer, filed the
    correction, and missed the bedrock one.
  - **F2 🔴 USER-OWNED — A-7's enumeration is short by three, and it collides with a ratified
    ask.** `REFERENCE_MATERIAL=MUDSTONE` (`lithology.rs:731`), `DEEP_BASEMENT=GRANITE` (`:748`),
    `ANCHOR_MATERIAL=SANDSTONE` (`transport.rs:86`) — two of which the **ratified**
    members-into-history § 6b explicitly asked for, against A-7's *"naming is not a ratifiable
    carve-out… a defect."* Ratified design ask vs read-first anti-shape, in direct conflict: a
    loud plea, not an edit. The audit's own diagnostic points at the resolution — the
    `GeologySet` declares neither its basement nor its reference sheet, while
    `SpeciesAxis::new(geology, basement)` already takes basement as a **parameter** that
    `set_species_axis` hands a constant.
  - **F4** A-2 partially applied: the `Litho::of_material` guard claim was fixed on the function
    (`lithology.rs:353-355`) and left verbatim on the test (`:1174-1176`).
  - **F8** A-4's extraction entry cites `split_by_shares`, which no longer exists (successor
    `species::split_row_into`; substance survives). **F10** eight citations into `erosion/` and
    `tests/providers_common/mod.rs` drifted through the split-then-re-move.
  - **F6** the content-door § 3 row's **severity rose without its status changing** — the species
    axis is now content-derived, so a custom pack gets a vanilla axis too.

- **🟠 A FIXED 12 fps ALIASES A DERIVED CADENCE — the stout hitches, and it is the smallest body
  that shows it first (user-sighted 2026-08-03 at the derived-gait walk, station 3; mechanism
  computed from the shipped formula, journal/0148).** `ANIM_FPS = 12.0` is fixed while cadence is
  now **derived per body**, so frames-per-cycle is no longer a constant anyone chose: at 4.5 m/s
  the biped gets **6.97** frames/cycle (essentially 7 — near-integer, so its samples nearly repeat
  and read stable), the longleg **7.73**, and the **stout 4.29** — shortest legs, fastest cadence,
  fewest samples, barely above Nyquist for the fundamental. The samples drift through the cycle
  and beat against it. **Anti-shape A-1, fourth instance this arc has retired**: 12 fps was chosen
  when there was ONE clip at ONE cadence and "frames per cycle" was a number nobody had to think
  about. **The user's leaning is FORFEIT and it is NOT YET A RULING** — *"probably we just forfeit
  it… i just don't care enough"* — but the diagnosis inside it is worth keeping either way: **the
  fault is quantizing per SECOND against a cadence that varies per BODY; quantizing per CYCLE (N
  poses per stride) gives every body the identical stop-motion look regardless of leg turnover.**
  ⚠ **This is the deferred taste call from the quantizer removal (journal/0133), and its stated
  precondition — *"a body whose feet actually reach the ground"* — was met at this walk.** Owner:
  the user; adjacent to B3's 20 Hz vs 12 fps question (`posture-gait.md` § 7 member 2).

- **🟠 THE WALK LOOP HAS NO *FAST* STOP CHANNEL — the user built a WALL to compensate for agent latency
  (2026-08-03, derived-gait walk).**
  > **⚠ CORRECTED 2026-08-04 — corrections #100. This entry's title said "NO STOP CHANNEL" and the
  > body said the observer has none; both are false.** The in-game dev console is registered
  > **unconditionally** (`dc-client/src/app.rs:342`) and carries the dc-api command surface — the
  > user can press **T** and zero a move intent today — and `Authority::freeze_character`
  > (`authority.rs:583`) is already the freeze verb, wired only to session teardown. **Note which
  > way this cuts: the USER'S OWN WORDS below are narrow and accurate** — *"no way for me to tell
  > **you** to stop them"* is a statement about the **round-trip to the driver**, which is real and
  > unimprovable (≈4.5 m of travel). **The assistant-authored doctrine generalised it into "the
  > observer has no channel at all," which is a different and false claim.** Same shape as the
  > CLAUDE.md ecology widening: a user statement broadened by an assistant transcribing it, then
  > read as fact by everything downstream. Inventory + priced options:
  > `docs/audits/2026-08-04-walk-stop-channel-design.md`.

  *"i blocked them off, by the way, because there's no way for
  me to tell you to stop them before they reach a ledge. claude time vs realtime."* The loop's
  premise is that Claude drives while the user observes, but **every intent commits several
  seconds of world motion before the driver can react, and the observer has no channel at all.**
  Evidence is a wall the user placed in the world. Costs a real station: this walk's station 2 was
  also mis-framed (all three on one line, occluding each other) and had to be rebuilt as lanes.
  Candidates, none designed: a **leash/tether radius**; a **duration- or distance-bounded intent**
  (*walk 3 m then stop*); a dev **freeze-all-characters** verb. **Will bite every future bodies
  walk identically.**

- **🟠 SUBAGENTS DIE WHEN THEY BACKGROUND THEIR OWN GATE — two for two in one session
  (2026-08-02, both bodies builders).** Each implementer wrote its code, backgrounded a
  long cargo run (or armed a monitor for the build slot), ended its turn "waiting" — and
  never resumed when the background work finished; both harvests happened by hand from
  their worktrees. The work survived (worktree commits + Tee'd logs are the durable half,
  exactly as the wrap doctrine says); the agents' completion machinery did not.
  **Interim rule until the mechanism is understood: a subagent runs its gates in the
  FOREGROUND of its own session and waits — never `run_in_background` inside an agent.**
  FOLDED into session-workflow § Delegation 2026-08-02 (greenlit).

- **🟠 THE BUILD-SLOT MUTEX IS DEAD AS A MECHANISM — both failure directions in one evening,
  from two sessions that both know the doctrine (2026-08-01, bodies session + geology
  session, both testimonies in hand).** The bodies session's full test gate (this
  machine's single build slot, lock written first) died externally at **58/90 suites,
  802 passed, 0 failed, exit −1 mid-suite** — two foreign cargo processes were alive when
  the corpse was found. The geology session's own account, volunteered unprompted: it
  *waited* ~90 min for the foreign gate and did not touch it, **but** its cleanup was
  `Get-Process cargo,rustc | Stop-Process -Force` plus an **unconditional `Remove-Item`
  of `.agent-build.lock`** — *"ownership-blind by construction… I deleted a mutex without
  reading it back, which is the exact failure CLAUDE.md names. No harm done, wrong shape,
  my error."* **Mechanism of the gate kill: uncertain** between shared-`CARGO_TARGET_DIR`
  artifact collision from a concurrent build and something else; commit `f96f12c`'s
  message attributes it to the concurrent cargo and this entry is the banner on that
  attribution (both accounts preserved here; the substance — externally terminated, not a
  test failure — is not in doubt). **The observation:** an advisory lock file has now been
  ignored-or-unseen during a live gate AND deleted unread, by competent sessions, in one
  night. *"Do not answer 'the gate cannot see X' with a rule asking people to remember X"*
  applies verbatim. **✅ HEIR BUILT 2026-08-02 (user greenlit):**
  `scripts/cargo_mutex_hook.py`, PreToolUse on Bash|PowerShell — denies cargo while build
  processes are alive (own session included) or a foreign claim is <3 min old; stamps the
  lock itself on allow; **sessions no longer touch the lock by hand** (CLAUDE.md § Build
  rules updated same commit). Deny and allow paths both proven live in-session, including
  a real denial of this session's own cargo call against a planted foreign claim.

- **✅ RESOLVED 2026-08-04 — DISSOLVED, NOT FIXED, by the per-cycle quantization slice
  (journal/0155).** This entry asked that the wrap-before-quantize order be corrected *"with the
  slice that makes the pose sim-visible, not before."* **Per-cycle quantization is phase-domain
  quantization, so there is no absolute-time grid left for a loop to drift against** — the defect
  has no expressible form in the new scheme, and the builder reports it had no alternative to
  choose. `looping_wraps_to_the_same_phase` is back to an exact `assert_eq!` and now also covers a
  **0.7 s clip**, the exact case this entry named. *Recorded because a side-effect resolution is
  the easy kind to leave rotting on the board: nobody who fixed it was looking at this entry.*

- ~~**🟠 `quantize_time` FLOORS ON AN ABSOLUTE GRID THEN WRAPS, SO A LOOPING CLIP'S FRAMES ARE
  ANCHORED TO t=0 RATHER THAN TO THE LOOP**~~ (`dc-client/src/body.rs`; found 2026-08-01 while
  reviewing the quantizer removal's one flagged judgement call). `let stepped =
  (t * ANIM_FPS).floor() / ANIM_FPS;` runs **before** `stepped.rem_euclid(duration_s)`.
  - **Consequence:** for a looping clip whose duration is **not a whole number of frames**, each
    cycle samples a *different* set of sub-frame phases — a slow, subtle drift with no visible
    cause. **Latent today and measured so:** every *looping* shipped clip is frame-aligned at
    12 fps (`idle` 2.0 s = 24 frames, `walk` 1.0 s = 12), and the only non-aligned clip (`jump`,
    0.6 s = 7.2 frames) is a **one-shot**, so `rem_euclid` never runs on it. **A pack author
    writing a 0.7 s looping clip triggers it immediately.**
  - **The integrator's verdict on the flagged call:** the agent retargeted
    `looping_wraps_deterministically` from `assert_eq!` to a derived 1e-12 tolerance, and that
    was **locally right** — bit-identity was unachievable *by construction*, so the old
    assertion was false-by-construction and only ever passed because the 11.25° snap rounded
    `rem_euclid`'s last ULP away (anti-shape **A-3**, and the agent found and reported it
    honestly). **But it fixed the TEST to match the IMPLEMENTATION.** The one-line alternative —
    **wrap first, then quantize** — makes the original bit-identity assertion *true*, and
    removes the drift above at the same time.
  - **Deliberately not fixed now — BUT THE REASON FIRST GIVEN WAS VOID** (corrections #83). The
    original justification was *"the full gate measures ~9 hours, so a one-line change costs a
    working day"*. **The gate is ~35–40 minutes**, so cost is not the argument. **The surviving
    reasons are:** the bug is **latent** (no shipped looping clip is frame-misaligned), and the
    fix belongs **with the slice that makes the pose sim-visible**, where the assertion is
    re-tightened to exact in the same change. **Rides as-built on those grounds, not on cost.**
  - **⚠ IT STOPS BEING LATENT IF THE POSE BECOMES SIM-VISIBLE.** `posture-gait.md` § 5 proposes
    resolving damage against the **nominal pose**, which would make pose sampling
    **replay-critical** — and a tolerance-based test is not adequate for a replay claim. **Fix
    the ordering as part of that slice, not before**, and re-tighten the assertion to exact
    when it lands.

- **🟠 MEASURED 2026-07-29 (journal/0130 + journal/0131): BODY PLANS ARE A REAL SEAM NOW, AND
  THE IK'S RATIFIED SENTENCE IS HALF TRUE.** Two experiments, no engine constant moved.
  **Confirmed:** one unmodified clip set drives `dc:body/biped` and `dc:body/stout`
  (legs 0.50×, arms 1.60×) — derived leg rig exactly 0.500×, `idle` poses byte-identical
  across plans, walk artifacts *shrinking* 0.102 → 0.068 m, and the second body reads as a
  coherent creature. `bodies.md` § IK's *"one clip serves every mutation of a plan across
  differing proportions"* is **measured true**. **And the solver works:** `dc:body/longleg`
  (+0.12 m leg slack, pack-side content) plants 86/88 flat samples with a **−56.2°** knee —
  the first joint this project has seen bend under IK.
  - **⚠ THE HOVER IS NOT FIXED, and it has TWO causes.** *(Both causes since DELETED from
    the tree — stamped 2026-08-03, staleness F10: the rotation quantizer came out before the
    0140 walk, and `root_bob_m` left the schema in journal/0147; this bullet is history.)*
    Slack removed the *offset*
    (20 mm → 4 mm) and did nothing to the **15 mm oscillation**, which is the clips'
    `root_bob_m` surviving `ROT_QUANTUM_RAD` (11.25°) — the solver computes the 5 mm
    correction and the quantizer rounds it away. **Amplitude identical on both plans.**
    User, on seeing it move: *"the hover looks bad… it's not a bob, it's a hover."*
  - **⚠ FOUR absolute-metre constants sit in a pipeline whose premise is that proportions
    vary** (count corrected from three, corrections #78): hip height · the clips' root bob
    (jump 0.120 m = 13 % of the biped's hip, **26 %** of the stout's; both peak at *exactly*
    +0.125 m) · the **half-voxel** foot window (**1.02× the stout's whole leg**) ·
    `CROUCH_ROOT_DROP_M` (**97.8 %** of the stout's hip — a crouching stout has a 0.010 m hip
    and a degenerate 180° knee). **With one plan a length IS a ratio**: anti-shape **A-1**
    under the body builder's foundation.
  - **The window can never admit a terrain step — 2:1 by construction at every N** (window ≡
    voxel/2; smallest real relief ≡ one voxel). Foot placement structurally only ever sees
    *sub-voxel* offsets. Now asserted.
  - **Two gates nothing reconciles:** `longleg` plants standing and **refuses crouching**
    (4/88); `biped` plants crouching and refuses standing. The plan built to make the solver
    run **broke the posture that already ran.**
  - **✅ CLOSED 2026-08-01 (corrections #81) — THIS WAS NEVER A LIVE USER CALL after
    `posture-gait.md` was ratified**; § 1 dissolved it the same day and it was re-presented to
    the user twice regardless. Hip height, root bob and crouch drop are **deleted, not
    corrected** (outputs of the bake). User rulings: **no artificial gap between a load-bearing
    segment and the ground** (so the 20 mm is a defect, settled forward, not by archaeology),
    and **anything presumptive about size/proportion is scheduled for refactor**. The
    half-voxel window survives as an engineering fix — size it to the relief it must admit.
    *Downstream may now assume ground contact is the intent.* Struck original below:
  - **~~🔴 OWED — USER CALL, and nothing downstream may assume ground contact until it lands~~**
    (`bodies.md` § IK banner): is the 20 mm hip/reach gap intentional (feet clearing terrain
    seams) or an off-by-a-half-thickness? It decides whether all four constants become
    **ratios of the plan** — an engine change that invalidates the authored clip bobs and
    moves how every body looks — or whether *"feet to actual ground"* is **retired** as
    never-intended. A measured slack-vs-bend trade curve is in journal/0131 as evidence for
    the call, not a resolution.
  - **⚠ CORRECTED 2026-08-01 (corrections #80): the bullet below is right as arithmetic and
    WRONG as a diagnosis — preserved as dated testimony.** The **operative** cause of the
    hover is a space layering: `character.rs:219` excludes `root_bob_m` from the hip the IK
    solves against, while `:257` adds it to the rendered root — so the solver returns a
    correct, **constant** leg pose and the renderer hovers the whole body under it. The
    quantizer is the wall **behind** that one. The tell was free and misread: **knee angle
    constant while the gap tracks the bob**. Found by the user watching motion (third
    instance; corrections #77, #78, #80).
  - **🔴 THE SECOND WALL ~~DEEPEST CAUSE~~, quantified 2026-07-30 and known to NEITHER journal: THE
    STOP-MOTION IDENTITY AND PLANTED FEET ARE IN STRUCTURAL CONFLICT.**
    *(⚠ `ROT_QUANTUM_RAD` is GONE from the tree — stamped 2026-08-03, staleness F10: the
    quantizer was removed before the 0140 walk; the numbers below are history of the
    removed mechanism, and the surviving form of this conflict is the 12 fps
    cadence-aliasing finding, journal/0148 + the bodies close block's item 2.)*
    `ROT_QUANTUM_RAD = TAU/32 = 11.25°` (`dc-client/src/body.rs:35`). **One quantum of hip
    rotation moves the ankle 172 mm** on the biped's 0.88 m leg (199 mm on `longleg`,
    86 mm on the stout). The corrections at issue need **1.30°** (the 20 mm standing gap),
    **0.98°** (the 15 mm bob oscillation) and **0.33°** (`longleg`'s residual 5 mm) — so the
    quantizer is **9×, 12× and 35× too coarse respectively.** The solver computes each
    correction exactly and the quantizer rounds it to **zero**.
    - **No choice of leg slack can fix this.** It is an angular-resolution wall, not a
      proportion problem — which is why `longleg` bends a beautiful knee and *still* floats.
    - **It widens the user call.** `bodies.md` § stepped animation is **DECIDED 2026-07-19**
      — frame-stepped ~12 fps with quantized rotations, *"an identity, not a workaround."*
      Planted feet at sub-voxel precision is **inexpressible under it.** Both are available,
      but somebody must choose the mechanism: exempt the **IK correction chain** from
      quantization while keeping clip rotations stepped, or quantize the foot's **position**
      against the ground rather than the joint's **angle**. **This is a fork nothing in the
      corpus has posed, and it is upstream of the hip/reach question.**
  - **Consequence for the params conversation:** a plan parameter must declare its **UNITS**
    (ratio-of-plan vs absolute-metres) as well as its firewall side. The units axis is
    undetectable at one plan and load-bearing at two.
  - **Discharged:** the A-4 instance — the default body pack now loads through the registry
    door (`authority.rs::load_body_packs`) instead of the client calling the authored source
    directly, so first-party content ships the route a mod would take. `vanilla_body_pack` →
    `default_body_pack`.

- **🟠 "VANILLA" IS NOT THIS PROJECT'S WORD FOR THE DEFAULT PACK — STRUCK IN THE BODY FILES,
  220 OCCURRENCES REMAIN** (user ruling, 2026-07-29: *"the default pack is not elsewhere
  referred to as 'vanilla'… I would strike it"*). The premise that it was confined to
  `bodies.md` was **false** — it was already corpus-wide before bodies ever used it. Struck to
  **0** in the body files; residual **220**: crates **145** (`dc-api/src/classes.rs`,
  `dc-worldgen/src/pipeline.rs`, `dc-core/src/materials/geology.rs` and worldgen
  tests/examples — the geology cluster is the bulk **and is under active edit**), docs **37**
  (7 non-body in `API.md`), ROADMAP+history **8**. **Journal's 30 are append-only history and
  must NOT be swept.** Sequenceable as one mechanical sweep once the worldgen agents land.

- **🔴 USER FIELD REPORT + RULING, walk 2026-07-29 evening (station 2 of the member-#0
  far-slice walk): THE CELL LINES ARE GONE, AND THE WHOLE CAKE IS SWIRLED.** Reference
  poses (corrections #48 — a prose landmark is not a pose): ground **feet (106263, 21.85,
  −5506), yaw 0, pitch −0.05**; top-down **feet (106263, 1686, −5506), yaw 0, pitch
  −1.55**. Assets `0127-station2-alt2-topdown.png`, `0127-station2-ground-north.png`.
  Surface voxel ground-truthed: recorded sandstone+siltstone+carbonaceous-mudstone mix.
  - **(a) CONFIRMED BY EYE: no 460 m cell lines anywhere** — *"welp, there are certainly
    no cell lines anymore."* U22's line criterion is discharged and stays discharged.
  - **(b) REJECTED BY EYE: the cell-wide blend's SEMANTICS.** *"the slice of
    chocolate cake in the middle of the vanilla cake is gone. the whole cake is swirled
    now… the interleaved fingers like this, while being organic-ish and no longer having
    square boundaries, do not resemble the material distribution of nearground chunk
    rendering (which doesn't do the bilinear interleaving thing)."* The near ground skins
    from the record's per-voxel mixture + member dither; the far interior now interleaves
    *neighbour cells* everywhere (the E[w_home]=9/16 finding journal/0125 flagged — the
    eye and the near/far agreement drop 0.8225 → 0.7922 point the same direction).
    It also does not resemble the warm-derived LOD.
  - **(c) RULING: the far draw's blend rides for now and is REPLACED EVENTUALLY** —
    *"that's going to have to be replaced eventually and has been discussed before. i let
    this continue since you said we'll use this for more than just LOD"* — i.e. member #0
    (the K1 fine-read kernel) stands; the far field's *use* of it as a cell-wide blend is
    the interim.
  - **(d) USER DESIGN SKETCH, recorded verbatim and NOT reconciled (corrections #65):**
    *"once we have the refinement layer declarable operators over primitives, we should
    maybe be able to know what the "budget" of materials per operator is, and coarsely how
    the operators will shape the far field - we may be able to place textures more
    faithfully to the near field. perhaps."* The direction this names: the far register
    becomes a **summary derived from the refinement operators' budgets** (S-3: derived
    from the authority, never beside it), instead of an independent draw over shares. Kin:
    `summarize`'s octree-contract home (member-#0 MM-4 ruling) and corrections #39's
    honest-heir note. **Heir: the refinement arc, post-operators — a design conversation,
    not a slice.**

- **The tripwire sweep's residues** — the two parents are ✅ RESOLVED 2026-07-29 by
  journal/0126 and archived verbatim to [`ROADMAP-history.md`](ROADMAP-history.md)
  § *Observed — archived*: *"which shipped artifacts have no golden?"* (filed by
  journal/0124, record-side) and **🔴 *"no golden hashes the far field"*** (found by
  journal/0125's member-#0 slice, expression-side). **They were one finding meeting itself
  from both sides of the collapse tier, and they closed together** — 17 artifacts, 12
  covered / 4 newly goldened / 1 not shipped, `tests/artifact_tripwires.rs`. What is
  genuinely still open is only this:
  - **🟠 A FAR-FIELD GOLDEN ON A WORLD WITH LAND.** `GOLDEN_FAR_SURFACE` is captured on the
    `providers_common` fixture, and that fixture is **99.6 % ocean**: of 36,864 sampled
    columns, **160** front with anything but the ocean block, and the highest ground in a
    ±56,000-voxel scan is **+7 voxels**. So the far *height* field is fully pinned while the
    far *surface-class draw* — the thing journal/0125 changed, and the reason the hole was
    found — is pinned by 0.43 % of the sample. Home: `contents_contract.rs`, whose Medium
    seeds are the worlds with land. Cheap; it is a second capture, not a second instrument.
  - **`Pregen::grid` and `Pregen::pipeline` were not walked.** The sweep enumerated
    `DeepField` exhaustively and added the far surface; nobody has yet said, per member,
    which golden catches a change in the *coarse* grid the deep run is built from. Same
    enumeration, one tier up, and the same cost (cheap).
  - **The `GOLDEN_LEDGER` debt is armed, not owed.** `DeepField::ledgers` is empty in every
    shipped world (`weather_inventory` off), so it has nothing to hash; the commit that flips
    the flag on owes the golden in the same diff. That obligation is carried where it will be
    read — the failing assertion in
    `artifact_tripwires.rs::every_deepfield_member_is_classified` says it — rather than only
    here, per read-first item 5's *a one-directional pointer is not a pointer*.

- **Poke-through geometry check (re-filed 2026-07-29 — the APPEARANCE WALKS OWED parent
  moved to history; this residue was live inside it):** whether near-field geometry pokes
  through the far sheet at tier boundaries has never been visually checked; ride it along
  the next appearance walk's stations rather than launching for it alone.

- **⚠ THE "MEMBER STEPPING DOMINATES U3" CLAIM DOES NOT SURVIVE THE SITE — measured
  2026-07-29 (journal/0129), and this is the third time this question has been answered
  with a different answer.**
  > **⚠ CONTESTS A POSSIBLY USER-ORIGINATED CLAIM — RECORDED AS A MEASUREMENT, NOT AS A
  > RESOLUTION, AND IT NEEDS THE USER'S RULING.** The 2026-07-24 diagnosis (*"the member
  > squares are one octave of value noise at chunk wavelength — fix = octaves, not
  > resolution"*) traces back to a conversation the user remembers and may have authored;
  > corrections #73 is the receipt that user memory is what protects this thread. Per
  > CLAUDE.md § *a user-originated design may not be superseded by an implementation
  > slice*, **an implementation slice does not get to retire it.** What is below is the
  > *evidence* (three measurements at the pose, one of which refutes the slice author's
  > own replacement hypothesis); what it is **not** is a decision about what the user saw.
  > The mechanism half of the diagnosis was **correct and is now shipped**; only the word
  > *dominant* is in question. The entry below is preserved because its *stepping* half is
  now shipped and measured; what is refuted is the word **dominant**. Three measurements
  at U3's own reference pose (`palette_quant_tour`, extended):
  1. **The vanilla set caps the member dither at a coin flip in 4 of 10 classes**, and
     nobody had ever printed the census. `clastic-fine`, `clastic-coarse`,
     `igneous-intrusive`, `igneous-extrusive` have **exactly two** members; the other six
     (every organic class, ore, accessory) have **one** — a constant field under any
     source. The loudest thing a member dither can draw is mudstone-vs-siltstone or
     granite-vs-diorite: **within-class pairs of similar albedo.** U3 was recorded as
     *"tan / grey / red-brown / dark-speckled"* squares — those are **classes**, which no
     member dither can produce.
  2. **At the pose the dither is inert for the surface anyway:** the surface top span is
     `Mixed` for all 1024 columns and `mixed_at` uses the **undithered** `event.member` by
     design; the topmost event's class (`organic-soil`) has one member.
  3. **The obvious replacement hypothesis also died in the same run.** A chunk's whole
     record comes from ONE `run_strata` over chunk-*centre* context, so it could step at
     chunk lines independently of any dither — measured, **0 of 56 adjacent chunk pairs
     differ in surface-class mix** over a 230 m block. Every chunk reads `o:0.80,S:0.10`.
  **Verdict: the 28.8 m stepping defect was real and is now measurably gone (on/off-lattice
  curvature 1.2e14 → 1.07); whether it was ever U3's DOMINANT signal is UNSETTLED, and the
  recorded pose cannot settle it because nothing there steps at 28.8 m today.** The fourth
  possibility is corrections #48's own hazard: **the pose is from 2026-07-24 and the world
  is not** — journal/0111's 1000× denudation recalibration and 0112's material creep both
  landed after it and both rewrote that surface. ~~**Owed before any game time: a tour map
  that finds a chunk whose surface top span is `Single` in a two-member class.** Nothing has
  ever searched for that, and it is the only station where this fix is visible.~~
  **✅ ANSWERED WITH A PROVEN NULL 2026-08-02 (stamped 2026-08-03, staleness F5 — the
  answer sat in the audit while all three asking sites stayed open):**
  `docs/audits/2026-08-02-appearance-tour-p11.md` searched the whole shipped world —
  `Single`-movable surface top spans number **0 of 4,792** (99.1 % `Mixed`), with a
  positive control (73.6 % of columns carry a movable `Single` span BELOW the surface,
  so the instrument sees the category). **The station this walk needed does not exist
  on the shipped world; the walk should not be launched for it.** *(The correlation
  redesign moots the near-membership question anyway — its § 5 retires the dither.)*

- **The U3 checkerboard's dominant signal is the 28.8 m MEMBER STEPPING, settled
  2026-07-24 — and the answer never flowed back into the audit that asked it** (corrected
  2026-07-29, after the member-#0 design pass re-derived the question from the audit's
  never-updated INFERRED section and the **user's memory** caught the re-derivation).
  **⚠ ITS "DOMINANT" HALF IS NOW REFUTED — see the entry immediately above.**
  Primary evidence, cited because a close block is a handoff not an authority:
  **corrections #45** (member dither is world-anchored, C0-continuous; defect =
  **single-octave**) + the 2026-07-24 haunting diagnosis (*"fix = octaves, not
  resolution"*). Consequence for member #0's continuation slot (b): **the near-path fix
  alone will not clear the checkerboard — the octaves `DitherSource` is a co-requisite**,
  and both defects (460 m facies point-sampling; single-octave member dither) are real.
  Residue: a confirm re-shoot at U3's reference pose rides the next appearance walk.

**⬇ 94 → 91 entries, 2026-08-03 (archive pass 3).** Three entries whose own bodies declared
them resolved BEFORE 2026-08-03 left this section for [`ROADMAP-history.md`](ROADMAP-history.md)
**§ Observed — archived 2026-08-03 (pass 3)**: the look-ownership ruling + follow-travel slice
(resolved 2026-08-02, journal/0142), the three-artifact-tripwires expected-red (resolved
2026-08-02, corrections #92), and the 9-hour-gate falsification (resolved 2026-08-01,
corrections #83). **Every entry stamped resolved ON 2026-08-03 was deliberately kept live one
session for the user's review of the day's staleness/doc-topology stamps**, as were entries
whose bodies say they are kept on purpose (charcoal-premise, build-slot-mutex banner). § Sequenced
was swept and yielded zero movable entries. Verbatim moves, poses and assets intact.

**⬇ 129 → 85 entries, 2026-07-29.** Forty-five entries left this section for
[`ROADMAP-history.md`](ROADMAP-history.md) **§ Observed — archived**: **8 user field reports the
user personally struck** (*"plenty of these are going to be stale and were made in the context of
old implementations and we didn't close the loop as the project moved on"*) and **37 whose own
bodies already declared them ✅ DONE / RESOLVED / FIXED** and were kept live anyway. Archive by
**status, not age**; every entry moved **verbatim**, poses and assets with it. One entry was
*added*: a "no volcanism" observation that had been sitting inside another entry with no header
of its own since 2026-07-20 (S6 finding F5).

- **👁 THE COLD TIER DOES NOT LOOK LIKE WHAT LOADS IN — and unbiasedness is not the axis**
  (**user field report, 2026-07-29**). *"The bilinear noise does not actually approximate what
  loaded chunks look like well, it sticks out poorly. So there's a more foundational issue to
  that."*
  - **WHY THIS IS ITS OWN ENTRY AND NOT PART OF corrections #39.** #39 corrects the **bias sign**
    of the coherent bilinear source — the claim was it pushes class splits *toward* 50/50; the
    measured truth is it pushes them *away* (a 0.6 majority renders ≈ 0.67; the source
    **sharpens** the mix). Its two named heirs — far-`summarize` and a CDF-corrected source —
    are both answers to *"does the draw reproduce the true share?"*. **The user is asking a
    different question:** even a perfectly unbiased draw with the wrong **spatial structure**
    does not look like the near field it is standing in for. The cold tier's job is to *predict
    the near tier's appearance*, and **statistical agreement is not visual agreement.**
  - **Consequence for the board: ratifying or rejecting #39 does not touch this.** #39 is a
    correct correction to a question the user is not asking, and it must not be allowed to carry
    this observation into a ratification as a rider. *(#39 has sat `NEEDS RATIFICATION` for 7
    days; it stays open **and decoupled**.)*
  - **This is the octaves argument arriving from the other side.** Structure at every scale is
    exactly what a single-wavelength bilinear source cannot give, at any bias. Couples to the
    CoarseField adoption entry and to the refinement-primitive design pass (§ Sequenced), and
    corroborates the user's own 2026-07-24 design (journal/0088): *"a finer grid just makes
    smaller squares."*
  - **Scope check — this is FAR/COLD ONLY, and that is load-bearing.** journal/0074 removed the
    near ground's class consult (`collapse.rs:881-883`), so the surface-class draw #39 corrects
    is far-field exclusively. **Do not conflate with U3**, the near-field per-chunk checkerboard
    formerly at `collapse.rs:1409` — different site, different tier, same root *(that site
    retired 2026-08-02/03, journal/0129 + 0145; staleness F12)*.
  - **UNMEASURED, and deliberately so.** No probe has compared a cold tile against the same
    ground loaded. That comparison **is** the instrument this entry wants, and it is cheap:
    render both, diff. *Do not close this on reasoning.*

Source: [`docs/audits/baseline-2026-07-28/S6-roadmap-observed.md`](docs/audits/baseline-2026-07-28/S6-roadmap-observed.md)
— every finding re-verified at source before it was applied, because *an audit finding is a
hypothesis, not an authority*.

**What this section was, measured, and what to avoid repeating:** 43 % of it was not an
observation. The dominant failure was **not** staleness-by-decay — it was
**closure-in-the-wrong-place**. Every one of the audit's top five stale entries had its refutation
*already in the corpus*, and three of them **inside this same section**: *"the HostWorld never
evicts chunks"* sat 700 lines below *"eviction landed"*; *"we cannot see where runtime goes"* sat
1,000 lines below a complaint **about the instrument it said did not exist**; two **UNWALKED**
tags sat 1,160 lines above the walk that discharged them. The missing act is discharging the old
entry **in the same commit as the new fact** — and here it would not even have required opening a
second file.

- **🟠 THE GATE CANNOT SEE BROKEN DOC LINKS — and doc comments are how this repo routes readers
  to its own doctrine** (surfaced 2026-07-28, **booked for a design conversation with the user**;
  recorded now rather than remembered, per *defer = write it now*).
  - **The observation.** `rustdoc::broken_intra_doc_links` is a **rustdoc** lint. `fmt`, `clippy`
    and `test` never run rustdoc, so **no gate stage can see it** — it is not a false green, it is
    a register the gate does not have. Found while verifying a doc comment written the same day:
    `cargo doc -p dc-sim` reports **two unresolved `[`draw_domains!`]` links at
    `crates/dc-sim/src/statistical/rng.rs:89`** (the macro exists in the crate but is not in scope
    at the link site). **Pre-existing, unrelated to the change that found it, and left in place**
    — it is a warning by default, so nothing has ever failed on it.
  - **⚠ SIZED 2026-07-28, and it is bigger than two:** `cargo doc -p dc-worldgen --no-deps` with
    the lint promoted reports **16 unresolved links** in that crate alone (`Litho`, `DepUnit`,
    `InvCtx`, `litho_of_tag`, `Fact::Move`, …). **Two crates checked so far, 18 broken pointers.**
    *One was instructive enough to fix on sight:* `providers/mod.rs:307` linked
    `Self::burial_temp_c` — **the seam that RETIRED into a field pass.** The doc rot is the
    seam-dissolution story leaving a dangling pointer behind, in the paragraph explaining the
    dissolution. **That is the failure mode in miniature: a pointer survives the thing it points
    at, and no gate can see it.**
  - *Method note for whoever takes this: a naive `cargo doc … | Select-String` reports **exit 0**
    while printing errors — the pipeline swallows cargo's code. Capture the exit status
    separately. Same "did it run?" vs "did it pass?" trap as the Tee'd-log rule.*
  - **Why it is more than lint hygiene here, which is the part worth the conversation.** This
    project deliberately puts **load-bearing doctrine in doc comments** — the module note that
    keeps the S2 tier from being deleted or adopted, `spines.md` § 3's *"record what consumed
    it"*, the seam-first practice of *"a doc comment naming its heir"* (**651 `heir` uses**). A
    doc comment is a **pointer meant to be followed by a cold reader**. A link in one that
    silently does not resolve is **the same defect class as the one-directional pointer decided
    today** — the reader arrives and the edge is not there — except this one is *mechanically
    detectable* and currently undetected.
  - **⚠ Do NOT treat "add `cargo doc` to the gate" as the decided answer.** It is the obvious
    move and it is not obviously right: the gate already runs in stages because it outgrew one
    tool call, rustdoc would rebuild documentation for the workspace, and **CLAUDE.md § Gates'
    own rule is that a probe nobody runs is silently wrong** — so the question is which register
    is missing and what the cheapest honest instrument is, not whether to bolt a fourth stage on.
    *There may also be a wider version of the question: how many other advisory-by-default
    diagnostics is the gate structurally blind to?*
  - **Blast radius: none today.** No behaviour, no world output. The cost is a cold reader
    following a pointer that goes nowhere, which is exactly the failure the corpus spent this
    week measuring.

- **🔴 FIELD REPORT (user, on the 2026-07-26 walk) — THE FAR TIER AND THE NEAR TIER DISAGREE
  ABOUT WHAT THE ROCK IS, ON STEEP SLOPES.** *"The cold LOD is showing granite and diorite
  textures, but in moving closer the surface is andesite and basalt. Maybe an artifact of the
  slope here, since granite and diorite are beneath the surface. The surrounding pit edge of
  sloping sandstone and conglomerate does not have this issue, also has a smoother slope."*
  - **Pose** (corrections #48 — a prose landmark is not a pose): the calibrated world
    (`dc-client --calibrated-rates`), seed 1337, `Extent::Medium`. Observed while flying
    between the station-A pit at world **(41870, 12433)** and the station-A3 lake basin at
    **(43709, 933)**. A later aerial from feet **(43709, 1180, 1420)**, yaw 0, pitch −0.62
    caught it at scale: a **hard rectangular seam** with different rock on each side, too
    straight-edged to be fog — `journal/assets/0115-station-a3-lake-basin.png`.
  - **The user supplied the negative control in the same breath**, which is what makes this a
    diagnosis-ready report rather than an impression: the *gentler* sandstone/conglomerate
    slope beside it does **not** show the disagreement. Steepness is the candidate variable.
  - **It is NOT `stubs.md` #15.** That stub (`far-node-synthesis-paints-the-column-with-the-
    surface-block`) predicts a far mesa's flank wearing its **cap** material. This is the
    opposite: the far tier showing the rock that lies **underneath** the near surface.
  - **Candidate mechanism (assistant, unverified):** `coarse_surface` is a floor-quantized
    summary; on a steep slope the quantized sample may land **below a thin cover** and report
    basement, while the near ground expresses the record's real top span through `ColumnFill`.
  - **Why it is more urgent than it was this morning.** `coarse_surface` is held to a
    **statistical** agreement test against `ColumnFill` — and a statistical test passes
    comfortably while a specific steep-slope class fails systematically (*"a summary is not an
    authority"*, with slope as the term nobody checks). **And the erosional calibration
    produces exactly the triggering geometry**: more relief, thinner cover on steeper ground.
    A pre-existing defect the erosion arc will amplify.
  - **First step:** re-observe under `--fullbright`, which is flat albedo with **no distance
    fog at all** (journal/0031), so a tier disagreement shows as pure colour with no confound.
    Deliberately not done during the walk — the user called the relaunch unnecessary.

- **DESIGN QUESTION PARKED BY THE USER MID-WALK (2026-07-26), recorded here so it is not
  lost — WHY DOES WEATHERING LOWER HEIGHT?** *"Wondering why weathering lowers height. What
  operations does weathering do that are not transform in place? Are they something that
  should happen prior to the river pass?"*
  - **Not answered, not investigated.** Filed verbatim under *defer = write it now*; the user
    explicitly deferred it to stay on the walk.
  - **Why it is a good question and not a detail.** Weathering converting rock to regolith
    *in place* should be mass-neutral in the column — it moves material between `R` and `H`,
    not out of the cell. If it is lowering surface height it is doing something that is **not
    a transform**, and that operation has a **phase-order** consequence: the re-scoped stubs
    #29 turns precisely on which phases lower a cell **after** incision has clamped it, and
    weathering is named as one of the four. **So this question sits directly inside the top
    blocker**, and answering it may narrow the blocker's scope.
  - **Related:** `material-behavior.md` § 12's transform/transport/genesis four-way test — a
    lowering that is not a transform is either transport or an unrecorded loss, and the four-way
    test is exactly the instrument for saying which.

- **🔴 THE ONLY TEST DEFENDING "A PLAYER CAN FIND AND DIG A COAL SEAM" RUNS ON A WORLD NO PLAYER CAN
  OPEN** (found 2026-07-25 by the ROADMAP staleness sweep, row S-7; **this is corrections #51 one file
  over**). `crates/dc-worldgen/tests/organic.rs::the_measured_coal_seam_is_coal_a_player_can_dig`
  builds `medium()` from **`const SEED: u64 = 0x0D5E_ED57_2026`** (`organic.rs:31`) — the warm
  reference world. `dc-client` can only ever open **`BENCH_SEED = 1337`** (an `i32`), and the
  reference seed **does not fit** it, so the world this guard measures is structurally unreachable
  from the game. Its threshold `MIN_DIGGABLE_COAL_VOX` (`organic.rs:78`) has been re-baselined
  **15 → 10 → 6**, twice under a **NEEDS RATIFICATION** flag, every time on that world.
  - **Why journal/0106's sibling audit missed it** (the part worth keeping): that audit asked *"is
    the claim seed-independent?"* and cleared `providers_common`, `rh_unification`,
    `s18_first_behavior_weathering`, `deeptime::production_config` and `water::coarse::production()`
    on that basis. `organic.rs` was **not on the list**, and unlike those its claim is emphatically
    **not** seed-independent: it is a claim about *what a player finds*.
  - **TWO HONEST OPTIONS, and they are the user's** — the same (a)–(d) content call corrections #51
    already put in front of them. **(i)** re-seed to 1337 and watch it fail, which is the true
    statement about the shipped world; **(ii)** rename it `..._on_the_warm_reference_world` and file
    the shipped-world diggability claim as **unguarded**, the way `coal_follows_the_warm_crust_…`
    was handled above. **Do not leave it named for a player.**

- **`DeepField::chapters` — built, exported, and called by nothing** (spine-audit
  2026-07-25; `field.rs:367`, populated `:421`). Its own doc comment has said *"today the
  table is exported and read by nothing"* since U8; readers are only tests. **Now listed in
  spines § 3, where it belongs.** The lesson is the reason it is here: **three consecutive
  sweeps added rows for its two immediate neighbours in the same struct and walked past
  it** — *a self-declaring comment is not an index*, demonstrated against § 3 itself.

- **A PROBE THAT CAN FAIL IS INVISIBLE TO THE GATE, AND THE ADVISORY RULE DID NOT HOLD FOR ONE
  DAY** (journal/0102, 2026-07-25). `examples/flow_cost_probe.rs` asserts its itemised residency
  equals `DeepField::resident_bytes()`. It broke when FLOW slice 1 added `flux` (caught
  journal/0100, fixed, and CLAUDE.md gained the rule *"re-run the probes by hand after any merge
  that changes what they measure"*). It broke **again the next day**, identically, when FLOW
  continuation (a) added `head` — the rule was already written and still did not fire. **The
  assertion caught it both times; the process caught it neither time.** CLAUDE.md's own remedy (a)
  is the real fix — *"an example that can fail belongs in the gate: put the assertion in a real
  `#[test]` that shares the code, and let the example print"* — and it is **not done**, because
  the assertion needs a production `DeepField` (~25 s of pregen) and whether that belongs in
  `cargo test --workspace` is a real cost call, not a mechanical port. **Scope fork, not decided
  here.** Cheapest honest version if the full test is too slow: a `#[test]` over a *small* extent
  that asserts the itemisation agrees — the term-drift this keeps catching is structural, not
  scale-dependent, so a 60-cell world would have caught both instances. The same exposure applies
  to every asserting example in `examples/` (`weathering_profile_probe`, `flux_record_probe`,
  `head_field_probe`, `identify_census`), none of which the gate executes either.
  - **⚠ STALE FOR THE FAMILY, STILL TRUE FOR ITS OWN SUBJECT — verified at source 2026-07-29**
    (S6 audit § 3, re-verified by this pass). The **mechanism shipped**: `test = true` on the
    example's Cargo target is now doctrine in CLAUDE.md § Gates (journal/0103, 35.0 s added), and
    `crates/dc-worldgen/Cargo.toml` carries **sixteen** named `[[example]]` targets — including
    `weathering_profile_probe`, `flux_record_probe`, `head_field_probe` and
    `contents_air_over_solid_probe`, three of the four this entry named — plus
    `crates/dc-client/Cargo.toml` for `identify_census`. So the last two sentences above are no
    longer true of the family.
  - **But `flow_cost_probe` — the probe the whole rule was earned on, named twice by name in that
    very Cargo.toml's doctrine comment as the cautionary tale — was never converted.**
    `crates/dc-worldgen/examples/flow_cost_probe.rs:412` still holds a bare `assert_eq!` inside
    `main`; the file contains **zero** `#[test]` and no `mod gate`; and `grep -n 'name = '
    crates/dc-worldgen/Cargo.toml` lists every other probe and **not this one** — the only
    `flow_cost_probe` token in any `crates/*/Cargo.toml` is the comment at
    `crates/dc-worldgen/Cargo.toml:21`. **The probe that broke twice, cost a CLAUDE.md rule, and is
    cited by name in the comment introducing the remedy is the one probe the gate still cannot see
    fail.** It would sit green through a full workspace run today exactly as it did then.
    **This is the highest-value single action left in this entry**, and it is now mechanical — the
    scope fork above was decided by journal/0103 (*size the test, not the report*: smallest extent
    that exercises the invariant, and this invariant — an itemisation equal to its own total — is
    scale-free, which the entry itself already argued).

- **OWED / the SAME lever, one record over: `DeepField::strata` is `Vec<DeepStrata>`**
  (journal/0102, 2026-07-25 — the successor the ledger collapse names). Measured on the production
  world: **9.06 MiB of 32-byte per-cell structs, 5.8 % of the whole flag-off field**, sitting on
  an 84.47 MiB `DepUnit` heap — and **33,680 cells (11.3 %) hold an EMPTY record**, paying the
  struct for nothing. The collapse is verbatim: one grid-wide `DepUnit` array with the cell as a
  CSR row. **Bigger than the ledger slice was**, for two reasons that are both known, not
  guesses: `DeepStrata` is written per-epoch during the compile (so it needs the same clock split
  — `DeepStrata` stays the gen-time accumulator, the resident record is grid-wide, compacted at
  the existing `shrink_to_fit` seam in `build_field_cfg`), and it is *read all over the collapse
  tier*, so `record_at_voxel`'s consumers need the same view treatment `ledger_at_voxel` got
  (which, on the evidence of journal/0102, can be zero call-site churn if the view carries the
  same read surface). Same family as `Vec<Vec<Fact>>` and `Vec<FactLedger>`, one record over.
  - **✅ VERIFIED STILL OPEN 2026-07-29** (S6 finding F6): `crates/dc-worldgen/src/deeptime/field.rs:473`
    is still `pub strata: Vec<DeepStrata>`. The CSR collapse is **not** done. Re-checked so the next
    reader does not have to.
  - **CROSS-REF added 2026-07-25 (sweep row A-1): this is now on the WEATHERING-IS-ONE-PROCESS
    arc's critical path.** That arc's requisite R3 named **residency, not gen time**, as its
    blocker (`LedgerField` 17.45 MiB → **973 MiB**, 55.8×, resident), so the strata collapse is the
    next lever on **the same budget the weathering arc has to fit inside**. Read the two together;
    freeing 9 MiB here is not decisive against 973 MiB, but the two entries are competing for one
    number and neither should be planned alone.

- **A front's parent alternates diorite/granite down a single column** (observed 2026-07-25 by the
  weathering-profile slice; **pre-existing, merely made visible**). `Single` voxels resolve their
  member through the per-voxel-column **member dither** while `Mixed` voxels use the **canonical**
  member, so a column whose voxels alternate plan kinds also alternates rock identity. Invisible
  until a front produced many contact voxels in one column. Same family as the palette-quant
  member-stepping thread (a *within-class member* choice reaching the eye), and it belongs with the
  genesis-passes arc that retires class-member selection.
  **CROSS-REF (2026-07-25, sweep row A-4): this is the member-dither family's first *measured*
  within-column instance**, and its untested-either-way twin is **"Mixed voxels carry no member
  dither — watch for the chunk-line cutover"** further down this section. One is now observed and
  one is still a hypothesis; they are the same mechanism seen from two sides, so a slice that
  touches either should settle both.

- **A column expressing ONE recorded voxel over 64 unrecorded** (measured 2026-07-25 by the
  contents probe at journal/0097's station: `dc:peat` at `y = 300`, unrecorded basement below).
  The **thin-record** family — one notch less extreme than the bare-cell zero below, and unlike the
  reporting defect above this one **is genuinely about the world**. Undiagnosed; likely the same
  root as bare-cell (sub-voxel record thickness falling off the record path).

- **`world_scan_region` / `world_get_block` are structurally blind to stratigraphy** (walk
  observation, 2026-07-25, journal/0097). Both answer with the stored **1-byte `Block` summary**,
  which collapses mudstone, sandstone, siltstone, carbonaceous mudstone and granite alike into
  `dc:stone` — so a column scan from veneer to basement returned `palette: ["dc:stone"]` and the
  walk would have concluded *"no band"* from an instrument that **cannot see one**. `world_get_contents`
  is the honest surface and showed all of it. This is ARCHITECTURE.md's *"a summary is not an
  authority"* **caught in the field, by an agent using the tool wrong** — and it is a live argument
  for the Sequenced **HONEST IDENTITY SURFACE / `identify(pos)`** arc: a tier-flagged answer is what
  a walk needs, and the untiered `Block` token is a trap for exactly the reader who does not already
  know it is one. *(CLAUDE.md § Agent walks updated in the same commit: use `world_get_contents`,
  never `scan_region`, for any material question.)*

- **`derive_regolith_at` derives `H` from an EMPTY ledger — Movement 2a's derived views
  cannot see `dc:deep/weather_inventory`'s facts** (spine-audit 2026-07-24, post-M3).
  `DeepField::derive_regolith_at` (`deeptime/field.rs:545`) builds its working inventory from
  `FactLedger::empty_with_bedrock(strata)`, so the "inventory is the authority; `R`/`H` are its
  materialized views" claim (journal/0092) currently holds for the **record half only** — the
  derived `H` structurally cannot see the **6.09 m of `Loose`** the M3 weathering process commits
  per epoch. This is the **concrete content of the R/H-unification residual** left by stub #17's
  discharge (the deferred half of its heir sentence). Not a defect *of* M3 — the two-authorities
  split was the briefed design — but it is the thing Movement 2 must actually close, and it is
  the honest reason "one authority" is not yet true. **Tracked:** ROADMAP Movement 2 /
  material-behavior.md §11 continuation slot + §13.6 / stubs.md #17 Residual.
  **✅ VERIFIED STILL OPEN 2026-07-29** (S6 finding F6): `derive_regolith_at` still builds from
  `FactLedger::empty_with_bedrock(strata)` — `crates/dc-worldgen/src/deeptime/field.rs:822` (the
  line moved from `:545`; locate it by content, not by number). The derived `H` still structurally
  cannot see `weather_inventory`'s facts. Both F6 entries are on the weathering arc's critical path
  and compete for one residency budget — read them together.


- **STATION — per-chunk material-palette quantization makes the chunk grid a
  visible checkerboard** (user field report + diagnostic station, 2026-07-24;
  `journal/assets/0088-palette-quantization-chunk-seams.png`).
  **UNTOUCHED BY THE 2026-07-25 WORK, and the mixture-representation arc it converges into is
  still held** (cross-ref added by sweep row A-5). One thing did move underneath it:
  **`identify(pos)` is now UNTIERED** (journal/0101), so the old framing *"the distance pyramid —
  near/mid/far **are** the `identify` tiers"* **no longer holds**. The pyramid is a **render**
  concern only, which is the tier this station and the far-LOD material split both live at.
  **THE REFERENCE STATION — EXACT POSE, recorded 2026-07-24 (user-recovered).** This is
  the standing reference point for the palette-quantization issue; re-shoot it to compare
  before/after any fix:
  > **feet (world m): `x 71291.7, y 372.1, z -2420.9`** · voxel `79213, 413, -2690` ·
  > **`yaw 21.9968`, `pitch -1.5475`** (looking ~straight down) · fly on.
  > Re-shot frame: `journal/assets/0088-palette-quant-reference-station.png` (matches the
  > original capture: tan/grey/red-brown/dark-speckled squares, yellow inclusion flecks).
  **CORRECTION (2026-07-24):** this station was previously described only as *"the east
  coast, ~110 km east of spawn"* — that distance was **wrong by ~39 km** (the true spot is
  **≈71 km east**, `x ≈ +71.3 km`), and the prose landmark sent a reconstruction attempt
  into grey single-class coastline at 108–110 km, where the checkerboard is absent. The
  pose was never recorded at capture time (the diagnosis audit § 7 even listed "record the
  camera pose" as an unfilled needs-a-live-probe item). *A prose landmark is not a pose;
  "defer = write it now" applies to camera poses too.* The user noted the weathering regime
  here reads visibly different from the plateau spawn — worth its own look.
  A top-down view shows the
  near/mid field as a **grid of chunk-sized squares, each a distinctly different
  overall tint** (tan / grey / red-brown / dark-speckled), while the **structure is
  continuous across the seams** — diagonal dune/wave bands of structure blocks (with
  yellow inclusion flecks) cross chunk boundaries unbroken, dithered mixed loose
  voxels between them. So the weathering/dune *pattern is continuous* across seams
  but the **materials are discontinuous** — an **S-4-square violation on the MATERIAL
  axis** (a chunk-resolution edge reaching the eye as a square; distinct from the
  far-field LOD material-identity split below, S-9, and the "far field boxier"
  geometry thread). **Two theories, to be measured:** (T1) the material *families*
  genuinely differ per chunk — same kind of mix, but a dominant family swapped
  (sandstone here, mudstone there) — a **generation-side** per-chunk material
  selection. (T2) the full contents vary *smoothly* (same materials, shifting
  weights) but voxels hold **more than the 4 materials a mixture can splat**, and the
  visible **winner reduction (>4 → 4) is salted per-CHUNK instead of per-voxel-
  position**, so the survivors are uniform within a chunk instead of jittered — a
  **render/materialization** bug (the >4→4 cut is the likely locus; journal/0055
  world-anchored dither, 0008 seam-order `MixtureTable` ids, the combinatorial mixture
  cap). **The discriminating measurement:** sample the *full* top-layer contents (all
  materials + weights, NOT the classified winner) across adjacent chunks at this
  station — smoothly-shared materials with jumping winners ⇒ T2; genuinely different
  material sets ⇒ T1. **This needs full-contents access, which the live
  `get_block`/`scan_region` do NOT expose** (they return the classified winner only) —
  see the look-at-contents dev slice below. **DIAGNOSED 2026-07-24**
  (`docs/audits/2026-07-24-palette-quant-generation-diagnosis.md`): **T1 confirmed, and
  it is the SAME ROOT CAUSE as the haunted-LOD blobs.** A chunk's entire strata
  composition is built from a **single point-sample of the deep record at the chunk
  centre** (`record_at_voxel(cx*32+16, cz*32+16)`, `collapse.rs:1474`), NEAREST at the
  **~460 m deep-cell grid** (`DEEP_CELL_M`); `run_strata` shares that one sample across
  all 1024 columns. So composition tiles are **~460 m deep cells with boundaries
  chunk-quantized to 28.8 m** (a staircase); climate/elevation/member-dither/mesher are
  all smooth/per-voxel — hence *continuous pattern, jumping composition*. The far LOD
  blobs are the **same coarse ~460 m NEAREST deep-facies field point-sampled instead of
  interpolated/area-summarized**, at a different site (near `record_at_voxel`→
  `run_strata`; far `surface_class`→`draw_class`) — one phenomenon family, two tiers.
  It is the **S-4 rule unmet**. ~~**One live-probe question left:** are the visible squares
  the 460 m deep cells or the 28.8 m chunk staircase~~ **✅ SETTLED 2026-07-24 — the dominant
  signal is the 28.8 m MEMBER STEPPING (single-octave member dither; corrections #45,
  "fix = octaves, not resolution"). Do NOT re-derive this as unprobed — that already happened
  once (corrections #73) and this clause was the stale end both times** *(struck 2026-07-29,
  doc-topology F2: the answer sat 345 lines up in this same file while this entry still said
  "planning held")*. **DIAGNOSTIC STATION — return here to
  check any fix:** feet `pos {x: 71291.7, y: 372.1, z: -2420.9}` / voxel
  `{x: 79213, y: 413, z: -2690}`, `yaw 21.9968`, `pitch -1.5475` (looking ~straight
  down), fly on. A correct fix dissolves the chunk-square tint grid into continuous
  ground with the diagonal structure bands unbroken. **Diagnose before touching.**

- **Far-field LOD reconstructs a coarse box's MATERIAL IDENTITY differently on
  cold-gen vs warm-regen** (user field report, 2026-07-23; **mechanism sharpened
  2026-07-23**). The coarse box for an area shows one material on its *first*
  (cold) generation and a *different* one after the area has been resident and its
  LOD is regenerated: the user watched mudstone coarse boxes gain **granite**
  patches on the second gen — **the sub-surface layer falls into the sample on
  warm-reduce but not on cold-synthesize.** So the two producers of the *same*
  coarse box pick a different winning material: **cold-synthesize (surface-sampled)
  vs warm-reduce (`dominant`/`classify` over resident voxels, subsurface
  included)** disagree — the reduce-vs-synthesize discrepancy (journal/0070
  measured a +0.938 coarse voxel bias). This is a **material-identity**
  disagreement, not a resolution/lighting one: it lives in **block-is-material /
  `classify` / `MixtureDownsampleRule`** and belongs with the **far-field span
  migration under Crux 1**. It is a live violation of **spines S-9's consistency
  law** (a collapse/reduction must agree with the base it refines). Couples to the
  albedo-at-range thread. **DIAGNOSED 2026-07-24**
  (`docs/audits/2026-07-24-lod-pre-post-visit-diagnosis.md`): it is ONE path with
  ONE distance gate — `REDUCTION_STANDOFF_M = 176 m` (`farmesh.rs:594`) sits *inside*
  the L1 ring (112–256 m), so L1's inner shell (112→176 m) synthesizes **cold**
  (surface-dither) while every band ≥ 256 m (L2–L4) **warm-reduces** (dominant
  subsurface) — the "band inversion" is **static/deterministic, not an eviction race**.
  Cold blobs = a coherent bilinear field point-sampled at the coarse stride (adjacent
  cells land on different members); post-visit faithful = the reduction sees real
  subsurface strata; the two producers were only ever held to tops/occupancy + class
  agreement, never material identity (the S-9 violation). Root: two unreconciled eras
  (FF2a synth-dither vs FF2b reduce) made *visible* by the 0087 block-collapse; the
  176 m standoff was geometry-motivated and never meant to flip *material*.
  **FIX DIRECTION (user, 2026-07-24):** (1) *never cold over warm-capable data* — the
  standoff is backwards; warm must reach the nearfield edge (loaded chunks → warm, no
  cold shell). (2) Cold's dither must be **replaced with a continuous method** (it is
  discontinuous with both the real chunks and the warm band). (3) The deeper truth:
  **both warm and cold fail to represent MIXTURE** — the real fix is the
  **blended-mixture-albedo cascade**: textured mixture (near) → averaged mixture
  albedo, one blended colour per coarsened voxel (mid) → flat winning material (far),
  pure mixed albedo becoming *necessary* the further out the band. This **converges
  the distance-pyramid Sequenced item + the palette-quant station (T1) + the contents
  inspector** into one mixture-representation arc — plan them together. **The hard part
  is representation, not data** (user, 2026-07-24): a blended mixture still comes out as
  **quantized colour patches unless dithered**, and the current dither technique looks
  bad on its own — so "draw the blended mixture" is genuinely hard to make look good,
  and the arc's real difficulty is the dither/blend method, not the plumbing.
  Implementation held pending that plan.

- **Perf: throughput ceiling at terminal velocity** (user field report,
  2026-07-23, on the 0083/0084 offload). Noticeably improved — the drop reaches
  the choking point *later* — but at terminal velocity it **still chokes, about as
  hard** once it does. So the meshing offload raised throughput/delayed onset but
  did not raise the *ceiling*: at max fall speed the streaming pipeline still
  saturates. Next perf targets from here: the `far_tile.snapshot` residue, the
  ~47 % un-instrumented `schedule` self-time (render/transform systems), and the
  streaming budget itself (`LOAD_BUDGET_PER_FRAME`) — but **measure with the
  per-thread-attributed instrument first** (below).

- **The perf instrument can't show the frame-thread envelope** (filed 2026-07-23,
  journal/0084). `PerfAggregate` sums self-time across the frame thread AND the
  task-pool threads, so after the offload the `%` table can't show spans "leaving
  the frame thread" — frame-count is the only clean signal. Owed: **per-thread-role
  attribution** (frame vs task-pool) in the aggregate, which is also exactly what
  the perf/debug overlay heir needs. Do this before the next perf slice so its win
  is directly visible.

- **Async-offload accepted corner — an edit to a chunk in its first ~1–2
  streaming frames meshes the PRE-edit snapshot** (journal/0083, ACCEPTED by the
  user 2026-07-23; must remain VISIBLE). Self-healing: any later edit to that
  chunk or a neighbour re-meshes it. Player edits on a just-appearing chunk are
  rare, but the real exposure is **simulation/NPC edits that fire the instant a
  chunk loads** — whose frequency is unknown until gameplay matures, and which
  will be **extremely hard to notice after the fact** (user: "we can accept it but
  we must know about it"). **Remedy is a DETECTION HOOK, not a doc note** (a note
  is exactly what gets lost): instrument the edit path with a counter on the
  perf/debug surface (journal/0080, the overlay heir) that fires when an edit
  targets an in-flight mesh task — so the corner self-announces the moment
  sim-editing begins, instead of being an invisible latent bug. Instrument-must-
  see-the-question applied to a deferred correctness risk. **Owed: build the hook**
  (small dc-client slice; sequence with or before the first sim-editing feature).

- **Mesh-buffer pooling: measured, deliberately NOT built** (2026-07-21,
  journal/0051 — the user asked for pooling; this is the numbered answer).
  New churn instrument in the `DC_MEM_PROBE` line: during a teleport storm the
  client builds ~130 near-chunk + ~32 far-tile meshes/second and spends
  **3.5 s of every 10 s (35 % of wall time) inside those builds** — but that
  window is per-voxel compute (32 768 voxels × 6 faces; far tiles dominated by
  coarse-summary sampling), not allocation. Decisive against pooling at this
  seam: Bevy's `Mesh::insert_attribute` **takes ownership** of each attribute
  `Vec` and moves it to the render world, so pooled scratch buffers would have
  to be **copied** in — zero copies per attribute becomes one. Residual idea
  that survives: reuse `MeshData`'s buffers *inside* `mesh_chunk` to kill the
  growth-reallocs — lives in `meshing.rs`, worth at most a couple of percent of
  that 35 %, unowned and unstarted.

- **The far field is now BOXIER than the near field** (user, live walk
  2026-07-21). Partial-height voxels give the near ground sub-voxel height
  variety; the far field is a whole-voxel column summary and cannot express it,
  so the LOD transition now differs in **geometry** as well as detail. Note
  what is and is not guarded: `coarse_surface_matches_near_column_height`
  asserts whole-voxel height agreement and
  `..._surface_block` asserts material agreement — **neither can see a
  sub-voxel height difference**, so this passed every test while being visible
  to the eye. Not yet diagnosed for severity; candidate answers range from
  "carry a partial-height byte in the far summary" to "accept it, the far
  field is a summary". Related: FF2b coarse volumetric summaries.
- **Organics render as solid boxes because form is keyed on CLASS**
  (found 2026-07-21 answering the user's question about why some surfaces
  quantize to boxes and others are partial). `fill::is_loose` returns true only
  for `clastic-fine` / `clastic-coarse` / `ore-placer`; everything else —
  including **organic soil, peat and coal** — goes to the structure bucket, and
  `meshing::height_frac` renders anything that is not loose-only at full
  height. So a carbonaceous-mudstone or peat surface is always a full cube
  while a mudstone surface can be a partial plate. Real soil and peat are
  loose. The user's instinct that this "has to do with form: structure vs
  loose, and provenance of how they got there" is correct **and is already the
  sequenced work**: consume-the-ledger piece (c), *derive material FORM from
  provenance* rather than from class. This is that item's first concrete,
  visible symptom.
- **Two more assumptions the partials-first world falsified** (journal/0057
  § i, recorded so nobody re-derives them as bugs). **(1)** Partial-height
  expression is now silently conditional on *which block the record produced*:
  a voxel whose block resolved to the 0055 fallback vocabulary (Dirt/Stone)
  renders full height regardless of a sub-8 loose contents value, because
  `block_uses_contents` gates it. Coverage stays consistent on both sides, so
  it is not a hole — but it is a decision nobody made deliberately.
  **(2)** The far field is block-tier and carries no contents, so the horizon
  draws every column full-height while the near field draws partial tops — a
  systematic sub-voxel step at the LOD boundary. Same root as the "far field is
  boxier" item; the distance-pyramid slice is where both get answered.
- **Mixed voxels carry no member dither — watch for the chunk-line cutover
  coming back** (journal/0055 judgment call 2, 2026-07-21; UNTESTED either
  way). The 3c-2 boundary dither (`dithered_member`) exists to wander material
  family contacts *off* the chunk grid; the agent deliberately did not apply it
  inside mixed voxels, on sound reasoning — the dither re-picks within a class,
  `classify` breaks ties on the lower material id, so dithering a 4–4
  cross-class split could flip the block and break
  `block == classify(contents)`. Single-event voxels keep the dither. The
  unexamined consequence: mixed bands may now show **chunk-aligned member
  selection**, i.e. the artifact 3c-2 was built to kill, confined to contacts.
  Nobody has looked. **Add it to the appearance walk**: stand at a cut face and
  check whether contact bands change member on chunk lines.
  **CROSS-REF (2026-07-25, sweep row A-4): the twin is now MEASURED from the other side** — see
  *"A front's parent alternates diorite/granite down a single column"*, where `Single` voxels take
  the dither and `Mixed` voxels take the canonical member **in the same column**, made visible by
  the weathering front producing many contact voxels at once. That entry is the observed half of
  this hypothesis; this one is still the untested half.
- **91.4 % of land skins to ONE block, and the diagnosis is not settled**
  (journal/0055, 2026-07-21). With the record deciding the surface, the
  world-wide skin goes Grass 81.8 % / Dirt 18.2 % → **Mudstone 91.4 %** /
  CarbonaceousMudstone 6.3 % / Coal 2.0 % / Peat 0.3 % / Granite 0.02 %. The
  agent filed this as a *block-vocabulary* question (the block tier cannot
  summarize the material diversity beneath it). **The integrator does not
  accept that as the whole diagnosis**, and the competing reading matters
  because it points at a different fix: the surface block takes the
  **metre-dominant class of the top 0.9 m**, so a thin organic soil horizon
  over thicker loess *loses the vote even when it is present in the voxel's
  eighths*. That is summarization hiding soil. The rival reading is that
  pedogenesis is simply under-modelled and soil is rarely generated at all —
  real Earth has soil nearly everywhere subaerial, and this record gives an
  organic surface to 6.3 %.
  **The discriminating measurement, cheap and unrun:** what fraction of
  surface *contents* contain organic material, versus what fraction of surface
  *blocks* read organic? If contents ≫ blocks, it is summarization (fix at the
  block/presentation tier). If contents ≈ blocks ≈ 6 %, the record itself
  lacks soil (fix in the biotic layer — adjacent to stubs 7/8).
  - **LARGELY ANSWERED 2026-07-21 by the user's live observation** (*"the world
    does not generate dirt anywhere apparently — was only a veneer placement
    previously"*), verified in the content set: **there is no soil material to
    place.** `MaterialId::LOAM` is defined but registered in **no** geology
    class member (its only other appearance in the tree is a unit test), so
    worldgen can never select it. And the `dc:stratum/organic-soil` class has
    exactly one member — `dc:geo/carbonaceous-mudstone` — whose material is a
    lithified **rock**. So every soil horizon the deep sim records is expressed
    as mudstone because mudstone is the only thing registered to express it.
    The monotony's two candidate causes were both downstream of this: there is
    nothing being hidden by summarization, because there is no soil substance
    in the world. `Block::Dirt` now survives only in the fallback paths
    (subaqueous, wilds, no-record columns) — the user's "veneer placement"
    read, confirmed.
  - **This is a CONTENT-SET gap, not a code bug**, and it is the missing
    substrate under an already-ratified design: the user's soil model (roots as
    porous *structure* holding soil in its pores; loose horizons above, packed
    below — materials.md § forms design pass, ideas.md § soil is loose but
    packable) has no material to operate on. It compounds with the form gap:
    `fill::is_loose` covers only clastic and placer, so organics fall in the
    **structure** bucket and would render as solid cubes even once a soil
    material exists. Two stacked gaps — **no soil substance, and organics
    formed as structure** — and the fix order is substance first, then form
    from provenance (consume-the-ledger piece (c)).
- **The eolian strata record costs +92.76 MB at Medium** (journal/0047
  measurement, 2026-07-21; UNDIAGNOSED in detail). The roster flip took the
  kept `DeepField` 52.8 → 145.5 MB — the record roughly triples because wind
  lays thin units across tens of thousands of cells per chapter. The clock
  cost is trivial (+2.5 s); the memory cost is the real bill. Perf-doctrine
  shape (recorder run-length merge / eviction tuning for eolian units is the
  obvious lever — the S10 soil-overprint precedent), but needs measurement,
  not a bandaid.
- **The 1-D wind march dumps residual load at the downwind land edge**
  (journal/0047, 2026-07-21). The raw strongest loess/deflation/frost cells
  all sit on grid col 544 — an edge-pileup artifact of 0034's march, real in
  every production world. The tour map excludes the border ring; the march's
  boundary handling is the defect. Small, mechanical, undiagnosed beyond the
  symptom.
- **The shared `CARGO_TARGET_DIR` can serve a STALE binary and produce a
  FALSE-GREEN gate** (found 2026-07-21 by the journal/0054 agent; the mirror
  of corrections #21's impossible *red*, and worse because it does not
  announce itself). Symptom: a full `cargo test --workspace --release`
  reported 489 passed / exit 0 while running **none** of the three tests just
  written — cargo reused a `dc_client-*.exe` timestamped minutes before the
  new source file existed, out of the target dir two agent worktrees share.
  Caught only by grepping for the new **test names**; `cargo clean -p
  dc-client --release` fixed it and the crate's count went 99 → 102.
  Remedy now practised by the integrator: clean every crate changed in the
  session before the merge gate, and verify by test name/count rather than by
  `test result: ok`. Undiagnosed in detail (which fingerprint input goes
  stale); see corrections #27.


- **The haze curve, not geometry, limits the usable vista** (journal/0042
  measurement, 2026-07-21). With `--horizon 8` the outer third of the field
  washes toward white and silhouette reading works only to ~5–6 km, even though
  the geometry is there and paid for. The fog *range* now scales with the
  horizon; the fog **curve** (its falloff shape) does not, and whether it should
  is a **user-owned visual call** the agent deliberately did not make. Cheap to
  change, needs the user's eye on a before/after.
- **Unlock ranking (assistant view, unratified, 2026-07-20)** — raised at
  the user's "what else do we need to get moving to unlock things?" and not
  yet answered, so recorded rather than lost. Ranked by how many downstream
  things they gate: (1) **persistence** — three systems have now
  independently filed "must persist" as an open question (far-field
  summaries, S11's water body graph, the generation-order-dependent
  `MixtureTable`), and edits still do not survive a scale switch; it is one
  answer being asked for three times, and everything built before it gets
  retrofitted after. (2) **inventory/items/interaction** (filed as 3a,
  never scheduled) — you can mine a coal seam and nothing happens; every
  content system terminates at a wall because nothing can be held.
  (3) **loose materials** — half-built already, since partial-height loose
  rendering shipped dormant in 3c-2 and only the content/sim half is
  missing. (4) **water present-tier** — S11 says GO.

- **Test-suite time is trending up: 382 s → 502 s → 541 s** across three
  2026-07-20 milestones (organics' biotic flip put a real ~14 s ritual
  behind every world-level suite; erodibility added its own). Each
  increment was individually justified and none was gated behind
  `--ignored` on the argument that gating would make the suite
  unrepresentative of the shipped world. No policy exists; the cheapest
  moment to set one is before it hurts.

- **An untold user reference: the "culture-language-balrog example"**
  (2026-07-20). The user cited it as load-bearing for how they picture
  darkness, danger and equipment — "you don't know about [it] but you'd get
  it" — then deferred telling it ("it'll cloud your context"). Recorded so
  the thread is not lost: it belongs in
  `docs/design/things-that-will-happen.md` when told.


- **"Our dismal mountains"** (user, 2026-07-20) — DIAGNOSED, four causes,
  all absences rather than bugs, which is why the terrain reads as flat in
  character rather than visibly broken:
  1. ~~**Erosion is lithology-blind**~~ **CLOSED AND SHIPPED ON 2026-07-20**
     (journal/0029 built it, journal/0030 flipped it). Erosion is lithology-aware
     (`deeptime::lithology`), with agent-specific resistance so the fix does not
     foreclose karst/glacial/littoral, and `production_config` now runs it.
     Differential erosion is proven present in the numbers (hard beds stand
     proud, basement shields for free) and bounded (stability clamp). **But the
     photographs say the flip bought no landform** — the ground-level surface is
     visibly re-terraced (real, measured in the lit pairs), yet the summit
     silhouette is identical, a 5 km lattice moved ±2 m on 2,615 m of relief, and
     no bench, ledge or resistant core is attributable to lithology at any
     vantage. **Cause 1 is closed and the mountains are still dismal**, which
     localises the remaining problem precisely: causes 2 and 3 are the
     load-bearing ones.
     **Remaining: 2 (no dip/fold), 3 (conservative amplitude), 4 (no glacial).**
     Cause 3 is now the highest-value next move and it is a **user decision, not
     a build** — the lever exists as knobs (`erodibility_contrast`, the global
     erosion rates), and 0029's headroom test (all rates ×10) already showed a
     hard bed standing 44.7 m proud. Nothing more should be photographed here
     until the amplitude call is made.
  2. **No dip/fold** (the layer-cake item below): even differentiated beds
     would only give horizontal benches and mesas; **dipping** hard beds
     are what produce hogbacks, cuestas and flatirons. Second in the
     user's ordering; S9 already classified fold-phase as a cheap *relax*
     term.
  3. **Conservative amplitude / no voxel-scale cliffs** (filed since S7).
  4. **No glacial agent** — cirques, arêtes, horns and U-valleys are the
     most dramatic alpine landforms and we have no cryosphere. Explicitly
     LAST (geology.md already sequences glacial as "later"); it needs ice
     as a modelled agent.
  Causes 1 and 2 multiply: hardness contrast supplies *what* stands out,
  structural dip supplies *how* it stands out.


- **The world is a LAYER CAKE — no dip, no folding, no tilt** (user,
  2026-07-20, from reading walk cross-sections: "the seams have to match
  orientation properly if this was pushed up in a tectonic event,
  subducted, etc."). **Confirmed by source read**: the only structural
  feature implemented anywhere is `DepUnit::unconformity: bool`
  (recorder.rs). Strata are per-column stacks of thickness, so every layer
  in the world is horizontal regardless of the tectonic history that
  produced it — a seam thrust up by an orogeny lies as flat as one that
  never moved. earth-processes § 7 (structural deformation:
  fold/fault/unconformity) is DESIGNED but only the unconformity flag was
  built; S9 classified fold-phase as a *relax* term (`fold-phase =
  f(uplift)` at sample time), so the cheap path was identified and not
  taken. Lateral variation exists only because adjacent columns read
  different 460 m deep cells. **This matters disproportionately for a game
  about reading strata**: dipping beds, folds and truncations are how a
  cross-section tells you what happened to it. Undesigned; pairs with the
  3e structural-deformation work.

- **Thick units render as flawless monoliths** (user, 2026-07-20; general
  case, noticed as house-sized unbroken pure coal). Two mechanisms, both
  ours: deep-time epochs are ~2.5 Myr so parting-forming events are
  sub-epoch and invisible, and the recorder merges consecutive like epochs
  into one horizon (the 665 k → 71 k unit optimization). Real thick seams
  carry mineral partings; real rock generally carries defects. Fix belongs
  at the **collapse tier as procedural detail** (method rule 5), not in
  deep time. Sketches in ideas.md § rock is not monolithic and § coal
  partings; pairs with the filed-not-built charcoal-as-inclusion item.

- **The sim must know about light** (user, 2026-07-20; NEW thread, see
  visuals.md § open thread). Distinct from render-side lighting: a
  deterministic sim-side light field is wanted for (candidates)
  photosynthesis/plant growth, spawn behaviour, stealth/NPC vision. No
  prior existed in the corpus. Noted parallel: a propagated light field
  and water.md's bound-water saturation field are the same computational
  shape (bounded local relaxation attenuated by a per-material property),
  so S11's locality result may transfer. Undesigned.


- **Walk report (2026-07-20, journal/0027): peat and carbonaceous mudstone are
  nearly the same colour.** `[0.24, 0.17, 0.11]` vs `[0.21, 0.18, 0.15]` — a
  difference you can measure and not really one you can see, and the fullbright
  control makes it worse rather than better (side by side and unlit they are
  nearly the same taupe; `0027-peat-coal-mudstone-section-fullbright.png`).
  Deliberately unfixed for now: peat is rare (17 cells world-wide on the seed
  walked), and the pair that actually matters — organic soil against ordinary
  mudstone — is well separated. Revisit when the placeholder texture packs are
  replaced with authored art, which is the right moment to space the organic
  materials across the palette on purpose.

- **Walk report (2026-07-20, journal/0027): the client can only ever open seed
  1337, so every seed-specific coordinate in the docs is unreachable in game.**
  `dc-client` takes no `--seed`; `app.rs` boots `Authority::new(BENCH_SEED, ..)`
  with `BENCH_SEED: i32 = 1337`, and the worldgen authority takes `seed as u64`
  — so `0x0D5E_ED57_2026` (the seed every spike and the whole S10/0026 site list
  is measured on) **does not fit in the client's `i32` seed** and world voxel
  (107338, 58787) is a headless-test address, not a place a player can stand.
  journal/0026's "stand at world voxel (107338, 58787) and dig down" is not
  actionable from the game. Cheap fix available (a `--seed` arg on the same
  path as `--pack`), not taken on a photo walk because it is client code and
  this walk changed none. Until then, walks must re-site their own subjects on
  seed 1337, which journal/0027 did.
  **✅ VERIFIED STILL TRUE 2026-07-29** (S6 finding F7): `crates/dc-client/src/bench.rs:17` is
  `pub const BENCH_SEED: i32 = 1337;`, consumed at `app.rs`, and **no `--seed` argument exists
  anywhere in `crates/dc-client/src/`**. This is checked deliberately rather than assumed, because
  it underpins the still-open coal-diggability entry above — that guard's world is unreachable from
  the game only for as long as this stays true.

- **The Voxy-vs-Distant-Horizons thread** *(submission question DECIDED
  2026-07-19 session 3 — docs/design/voxy-dh-recon-2026-07-19.md
  § Addendum: far field rides Bevy 0.19's engine GPU-driven path on the
  one-shared-material `Mesh` substrate; vertex pulling/bespoke cmdgen
  rejected; packed quad survives only as a candidate storage format.
  **Step-0 CONFIRMED** 2026-07-19, FF2a journal/0023: the multidraw path
  engages for our custom material — live `mode=Culling`, `Opaque3d
  batches=74 sets=1` on Vulkan; custom vertex attributes don't disqualify
  batching. Substrate decision is real on this hardware.)*
  Still open from the doc: persistent edit-updated LOD store (summaries
  beside S3 region files, dirty-rail subscription — still owed; FF2a kept
  the mesher pure + the `ColumnSpan` payload persistence-shaped so it's a
  drop-in); Aokana SVDAG ray-march as the FF2b volumetric candidate. Vista-as-augury
  constraint stands (rendering is never an observer — binds when
  live-sim state becomes far-visible).

- Walk 14 (journal/0019 § walk 14): *(fullbright lost the mixture speckle:
  **RESOLVED by PBR-1 render polish**, journal/0020 — a `FullbrightTerrainMaterial`
  (unlit custom `Material`) now renders uniform/block faces as one flat registry
  albedo and mixed faces as the world-anchored 4×4-cell constituent speckle,
  computed shader-side from the splat attributes + a world-anchored cell hash +
  the albedo palette uniform, NOT the LabPBR textures. Zero mosaic geometry.
  Live-verified: uniform spawn strata read flat under fullbright, both shaders
  compile/render error-free. **In-world speckle photograph still owed** — needs a
  real mixed face.)* **Member-contact + placer closeup shots owed** — not
  findable by eye under ground cover; wants the pregen-introspection dev MCP tools
  (walk-9 Observed item). The fullbright speckle photo rides with them (same
  scan-guided-site problem).

- **Placeholder texture tiling: cleanup pass DONE** (journal/0020, PBR-1 render
  polish). The walk-14 "houndstooth" was diagnosed as **legible per-voxel
  repetition of a low-frequency directional texture signature, NOT a seam
  failure** — the tiles wrap (the generator's `_seam_check` is still green;
  offline 4×4 tiling shows no boundary discontinuity), so the "seamless tiling
  verified" claim is **not** falsified (no corrections.md entry). Two mechanisms
  shipped: (1) **world-anchored UVs** in the mesher (tile origin = world voxel
  coord — continuous across same-material blocks and greedy quads, and the source
  of the fullbright speckle's world-stable cell grid; near visually-neutral in the
  lit path on its own for the current seamless tiles); (2) **de-directionalized
  placeholder textures** (`gen_placeholder_textures.py`: freq floor 2→4 + a
  three-octave fractal weighted to high frequency, albedo spread 0.17→0.11, relief
  2.6→1.5, softer AO ramp — regenerated all 26 packs, seam self-check still green)
  — this is the mechanism that actually removes the visible artifact. Live spawn
  (grass/dirt) reads as isotropic pixel grain with no legible period. **Geology
  outcrop before/after at the walk-14 framing still owed to the milestone walk.**

- PBR-1 loose ends (journal/0019):
  - **Terrain lighting is hand-rolled** (directional sun + hemispherical
    ambient from a plain uniform, not Bevy's clustered path — PBR-1 has no point
    lights). Sun/ambient calibration photographed in walk 14 (0019 assets:
    lit-vista, textured-outcrop — no top-face blowout at this calibration);
    **user read owed for the SPLAT_N=4 + calibration ratification**; the
    real curve arrives with PBR-2's HDR + tonemap.
  - **Far field + legacy S1 are now textured too** (one material renders the
    whole lit world). *(The phantom-far-world it painted is **gone** under the
    worldgen authority — journal/0022's summary-shaped far field; the same
    terrain material now lights the horizon that lights the ground.)*
  - **Normal-map tangent frame is a per-face axis-aligned approximation**; the
    normal X/Y orientation may be inconsistent across the six faces (cosmetic on
    the subtle placeholder relief). Revisit with authored textures + POM (PBR-2).
  - **The 16×16 atlas has no mipmaps** (nearest, no mip) — distance aliasing on
    the far rings; accepted for placeholder, revisit with real textures/POM.
  - **World-anchored UVs are f32 world voxel coordinates** (journal/0020) —
    integer at corners, so tiles resolve exactly to ~±16 M voxels and acceptably
    across the playable range; extreme deep-time coordinates would eventually lose
    texel precision. Accepted (the far field, the other extreme-coordinate
    consumer, is a known defect). One tile per *current-scale* voxel, so far-mesh
    coarse voxels still stretch a single tile — subsumed with the summary-shaped
    far field.
  - **Embedded WGSL shaders are compiled by naga at pipeline-build time** — no CPU
    gate (`fmt`/`clippy`/tests) sees them (journal/0020: a `active` reserved-word
    typo passed every gate and would have blanked fullbright; caught only by the
    live smoke run). The live launch + log read is the shader's only compiler;
    keep it in the walk protocol for any shader change.
  - Specular **porosity/emission channels are sampled but wetness is not wired**
    (no weather → no sim-driven porosity darkening yet; PBR-2 + materials sim).
  - Placeholder packs widened **21 → 26**; the historical placeholder-textures
    Shipped line (`ROADMAP-history.md` § Shipped, "2026-07-19 — Placeholder LabPBR
    texture packs") still reads "21 deterministic 16×16 three-texture sets" (not
    amended — dated record).
  - `MeshData` now carries UV + splat attributes on **every** chunk (far field
    and benches included), a small per-vertex memory bump over block-only meshes;
    accepted (the far field is getting subsumed anyway).

- *(**Far mesh generated from S1 `TerrainGen` under the worldgen authority:
  RESOLVED** — journal/0022, the far-field horizon. Key 2 now samples the
  authority's OWN coarse surface (`WorldGenerator::coarse_surface`, the collapse
  kernel at a coarse stride — the lattice already folds in the deep-time surface)
  into a heightfield far field; the phantom old world ~1 km down is gone and there
  is a horizon. Keys 3/4 keep the S1 far mesh (`FarFieldTerrain`), which was never
  wrong for the S1 world. **Still open**: sealing `TerrainGen` behind
  `pub(in crate::authority)` — keys 3/4 still name it, so the seal waits on
  retiring S1 entirely, not on the far field. **Deferred by design** (journal/0022,
  noted here): (a) *persisted* summaries — the far field derives in-memory on
  demand this milestone (~0.22 s/full field), region-file storage couples to S3
  grouping and is the follow-on; (b) the heightfield is a **top surface**, so
  looking up from deep in a chasm loses the far field — the volumetric shell did
  extend down a chasm; a volumetric/skirted summary is the follow-on.)*
  **✅ VERIFIED STILL OPEN 2026-07-29** (S6 finding F8): the seal has **not** happened —
  `crates/dc-client/src/worldgen.rs:27` is still `pub struct TerrainGen`, not
  `pub(in crate::authority)`. The entry is right, and right for the reason it gives: keys 3/4 still
  name it, so the seal waits on retiring S1 entirely. Kept OPEN, not archived, for exactly that.
- Walk 12 residue: *(far-mesh visual assessment: DONE, walk 13 —
  journal/0018 § the empty horizon: at worldgen altitude the horizon is
  sky; the S1 phantom shows only from steep angles through haze. The
  far-field milestone is "build the horizon", not "fix the phantom".)*
  Still open: test-suite time +~6 min (deep-time on every Medium/Large
  pregen — wants a cost-insensitive fast path); Large extent runs a
  coarsened (~1.8 km) deep cell under the width cap until 3e-2's C
  refinement restores landform detail on approach.

- Walk 11 + step-3 loose ends (journal/0014): *(crouch factor: DECIDED
  2026-07-19, bodies.md § Crouch semantics — 0.6× stands; crouch is
  posture + the future sneak edge-walk-block verb, never crawlspace
  access; sub-standing clearances belong to prone/crawl, sketched in
  ideas.md § posture ladder.)* Max-bend leg fold reads
  tangled — knee pole/fold distribution wants a photo-driven tuning
  pass. *(Posture not exposed in `character_pose`: **RESOLVED** — journal/0021,
  instrument batch; the pose reply now carries `posture` in `set_posture`'s own
  `standing`/`crouching` vocabulary, so a readback round-trips into a command.)*
  Foot-IK ground read can momentarily see the S1
  far-mesh phantom at the extreme load-radius edge.

- Geology v1 loose ends (journal/0007): sea-floor/wilds columns keep empty
  strata records (subaqueous sedimentation = the carbonate milestone); no
  `def_changed` events for class/member defines (hot-reload signal; the
  events schema still enumerates only the original three kinds); dev-token
  `dc:*` grant *(DECIDED 2026-07-19, API.md § Capabilities: dev builds
  yes, shipped builds never — build-config gated; implementation owed)*;
  worldgen `MixtureTable` is per-generator,
  unbounded, and its ids are generation-order-dependent — a save layer must
  persist the table, never re-derive it (lifecycle belongs to region-file
  grouping, S3 OQ 7).

- Walk 10 + 3d loose ends (journal/0011): *(member identity is
  render-invisible: **RESOLVED by PBR-1**, journal/0019 — uniform-contents
  voxels now sample their material atlas layer, so siltstone renders
  differently from mudstone; the 3d contact smoothing is now visible. Walk
  confirmation owed with the PBR-1 milestone walk.)* **Class-presence
  quantization still cuts on chunk lines** (flow_energy rounding
  per-chunk) — most likely what walk 8 actually saw; needs per-voxel-
  column context. Igneous tectonic-setting fitness axis deferred to 3e.
  Olivine albedo reads close to grass green from afar — registry-data
  polish with the real texture milestone. Accessory gate/pore-fraction
  values picked, not tuned.

- Walk 9 + 3c-2 loose ends (journal/0010): dither cell count (4×4) and
  the resolve-at-boundary sidecar transport accepted as-built (integrator
  review 2026-07-19, per the no-bandaid razor). Far field and the
  legacy S1 authority render no dither (near-field worldgen only).
  Partial-height rendering is dormant until loose deposition emits sub-8
  columns; edits don't update materials (safe via the block gate).
  Mixed-heavy meshing worst case 16× triangles (real placer bands are
  thin). The walk wants **pregen introspection as dev MCP tools**
  (hydrology/province queries) instead of a side-channel example binary.
  Coarse clastic is rare at world scale (5 river segments on the Medium
  seed) — honest sedimentology vs knob, revisit when rivers refine.

- Walk 8 + user live observations (journal/0009): ~~**nobody owns body
  orientation**~~ **→ RESOLVED 2026-08-02: the ENGINE owns it — look-follows-travel
  default in `step_character`, `set_look` holds until `clear_look` (bodies.md § who owns
  the look, journal/0142)** — `SetMoveIntent` never touches yaw, `SetLook` is the only
  writer, so un-looked bodies strafe/moonwalk; v0 fix is a design call
  (renderer-side trunk-toward-velocity, cosmetic, vs controller
  convention), real answer is staircase step 3's trunk/look split.
  **Faceless heads make orientation unphotographable** — a face cue
  (texture or v0 asymmetry) is prerequisite to walk-verifying facing.
  **Material families cut hard on chunk lines** — candidate mechanism: at
  N=2 the chunk (28.8 m) equals the S7 column-quantization cell, so
  cell-stepped climate context flips selection exactly on chunk borders;
  diagnosis + smoothing (context interpolation / boundary dither) filed.
  **Third-person view** needed eventually to verify the player's own body
  (dev affordance; not slated). `jump` remains a fallback pose, not an
  authored clip. Plans/clips are define-only (no get/list queries yet).


- Walk 6 loose ends (journal/0006): the surface scan window is
  S1-terrain-sized (8 m headroom / 220 m depth) — structures stacked >8 m
  above the analytic surface won't be snapped to; revisit when worldgen
  amplitude grows. `find_open_spawn` uses terrain-only solidity — correct
  at startup, wrong if ever reused post-edits. Dev `spawn_character` stays
  unguarded (dev keeps full reach) — *ratified 2026-07-19 along with
  `surface:true` attach semantics (API.md § Characters records it; this
  line was stale until the 2nd-session doc sweep).*

- Walks 3–4 (journal/0003, 0004 — corrected record in corrections.md #3,
  #5): near-field lighting blows out pale top faces (flat shading +
  near-vertical sun + no tonemap shoulder) — art calibration, belongs to
  the visuals pass. Chasm cliff "speckle" now photographed from verified
  clean air (0006 asset): reads as genuine single-voxel terracing on the
  near-vertical carve, not a mesh defect — diagnosis still owed.
  *(`surface_height_m` under-report: diagnosed and fixed, journal/0006 —
  the "missing octave" hypothesis falsified, corrections #5.)*

- Far field doesn't see edits — the worldgen far field is a coarse *summary*
  (not a cache), so a broken block un-breaks beyond the full-detail radius; a
  summary that tracks edits is a follow-on (journal/0002, 0022). (Same for the
  S1 far mesh on keys 3/4.)
- Edits don't survive a 2/3/4 scale switch (authority rebuilt); the command
  log is the eventual persistence answer (journal/0002).
- Far meshing is main-thread, budgeted (S1/S3, and the worldgen heightfield) —
  wants async tasks (journal/0022 keeps it budgeted/incremental).
- Rivers are straight cell-chords (S7); course refinement needs the 2-ring
  argument re-proved at finer levels.
- Terrain amplitude conservative — no voxel-scale cliffs (S7).
- Sparse sidecar encoding for thin debris drapes (S8) — index array dominates.
- Seed-stable worlds across releases: versioning policy undecided (S7).
- HDR/exposure: v0 post grades LDR; sky-as-pass needs hook format 1 (S4).
- **S2 checkpoint facts (deep-time re-derivation cost) — ~~design owed before ledgers densify~~
  🔔 TRIGGERED 2026-07-25.** *(Promoted from this one-line tail by the staleness sweep, row U-1:
  the condition it was waiting on has occurred, and a line that fires its own trigger silently is
  the exact failure the sweep exists to catch.)* **The ledgers densified — in one day.** The deep
  record now carries three sparse per-cell records where it carried one: `LedgerField` holds
  **1,033,189 facts across 72,006 slots** (journal/0102), `DeepField::flux` adds **2,590,372
  entries / 40.66 MiB** (journal/0096), `dc:field/head` adds **6.96 MiB** (journal/0098) — and a
  fourth is *projected at* **973 MiB** for a per-depth weathering ledger (the WEATHERING-IS-ONE-
  PROCESS arc's R3). So the **re-derive-vs-persist** question this line reserved is live, and it
  is the same question that now blocks that arc. **It does not start from nothing:** flow.md
  § 11.3 gives it a **standing constraint, ratified 2026-07-25** — *any mode that changes what an
  **absent** entry means must be carried in the record*, never held as external knowledge. A
  checkpoint scheme that silently changes the meaning of a missing fact is forbidden by that
  contract before it is designed.

- **Texture steps toward albedo at range** (user, 2026-07-22, on ratifying
  FF2b's look: "the moire is very strong and only nearfield really needs full
  texture… at range, step toward albedo. (5 km bluff may as well just be
  red)"). Priors already pointing here: `--edges` distance-fades to zero past
  1.4 km because per-voxel signal aliases into moiré (journal/0031); the
  houndstooth fix cured a range artifact by *narrowing* albedo spread
  (journal/0020); and every material already carries a flat registry albedo
  (journal/0010 — fullbright's register), so the far target color is
  existing data, and the lerp lives in the ONE shared material
  (pack-compatible, batch-safe). **User hypothesis, to be measured: this is
  also a render-perf WIN** (far pixels skip the splat/texture work
  entirely) — A-6-clean, since the measurement picks the shader path.
  Sequence with the profiling slice so the before/after is real numbers.
  **Rider (user, same conversation): rendering ranges are PLAYER-FACING
  config knobs.** "even at medium distance texture barely registers … may as
  well skip the stack. like anything, i would prefer to have rendering
  ranges as knobs. this is a game people will run on different hardware.
  (goes for the octree LOD bands too). this is a game, it comes with game
  config settings, one must imagine." — S-1's where-there's-a-range-there's-
  a-knob doctrine extended to the render tier AND promoted from dev CLI
  flags to game settings: texture/detail-stack fade range, LOD band radii,
  horizon. Filed as a constraint on the albedo slice and on all future
  far-field work: ranges arrive as config, not constants.


- **The cake observation: minority phases guillotine at cell perimeters
  under the coherent draw** (user, 2026-07-22, on ratifying B1's look —
  the swirl-cake-slice metaphor). **✅ ANSWERED 2026-07-29 — journal/0127 is
  titled "the cake observation, answered": U22 discharged, the 460 m cell
  lines confirmed gone BY EYE** *(staleness sweep F6 — the answer sat 1,000+
  lines away in the close block with no banner here)*. B1's dither made the
  SOURCE continuous
  but the SHARES it thresholds were still nearest-per-cell, so minority
  swirls died at the 460 m line while shared majorities flowed through —
  the S-4 sharpening catching the residual half of its own fix. The cure the
  entry named (seeded membership dither of the SOURCE CELL, bilinearly
  weighted) is exactly what shipped (`sample_dithered`, journal/0125).
  **STILL LIVE, do not inherit as done:** (1) the **member-dither sibling** —
  `geology.rs::dithered_member` under chunk-centre formation context, U22's
  named sibling, different site, **still unexamined** (member-#0 audit § 3
  confirms 2a does NOT fix it); (2) the **toward-50/50 bias** of
  interpolated-uniform noise — its named heir was the far-summarize register,
  which was **ruled out of the refinement tier** (→ octree contract), so the
  bias now rides the shipped far field with **no heir**.

- **`tectonics.rs` salt conversion owed** (draws.rs residue 3, 2026-07-29; promoted from the
  close block to a body entry by staleness sweep F11 — a close block is a handoff that gets
  archived, not a residence). Owed to the file owner when next touched.

- **`SALT_DT_PERTURB` is a FIFTH surviving hand-rolled salt, missed by the draws.rs residue
  enumeration** (FULL spine-audit 2026-07-29, proposed correction 1): the `f10dc03` fix
  named three surviving salts; the finding it corrected named four; **`refine.rs:29` (live
  call site `:180`) appears in neither** — the enumerate-what-you-touched mechanism the
  paragraph was written to retire. Preferred fix: **convert, don't document** — one call
  site, byte-identical~~, and `refine.rs` then holds zero hand-rolled salts~~. ~~**QUEUED behind
  the in-flight member-#0 build pair's merge** (its worktree owns the draws.rs
  neighbourhood; never dispatch into a file an unmerged branch touches).~~
  **✅ CONVERTED byte-identically (stamped 2026-08-03, staleness F6: `draws.rs:205`
  registered domain, proof of byte-identicality at `:691-707`) — BUT the struck success
  condition above was FALSE when written and stays unmet: `refine.rs:152` still reads
  `grid::SALT_DT_ROUGH`, a sixth hand-rolled salt the residue enumerations missed
  (the enumerate-what-you-touched shape, again). That read is the remaining owed
  conversion, to the file owner when next touched.**

- **The spines § 6 audit index has no completeness check — it fell five behind ONE DAY
  after the backfill that fixed eleven** (FULL spine-audit 2026-07-29, process finding).
  A backfill is not a watcher; the fix is an enumeration something CHECKS (the
  `every_deepfield_member_is_classified` shape, applied to a doc index), not a list a
  human appends to. Sibling of the "no enumeration in the docs is checked for
  completeness" entry above — this is its fifth hand-found instance.

- **The tour-map instrument needs a LAND FILTER** (found 2026-07-29 during the far-frontier
  tour-map: station 1 scored a below-sea-level basin; full finding in
  `docs/audits/2026-07-29-far-frontier-tourmap.md`. Promoted from the close block by
  staleness sweep F11). Walk stations 3/4 of that map remain unwalked; poses in the same
  audit.

- **Texel-edge dither bands on close-pressed walls, anisotropic** (user field
  report, walk 0071, 2026-07-22; asset
  `journal/assets/0071-artifact-texel-edge-jitter-wall.png`; observed at
  world x ≈ 99,341 m, y ≈ 198 m — a basalt hole wall). At extreme
  magnification, VERTICAL texel boundaries dissolve into noisy dither bands
  while HORIZONTAL boundaries stay razor-straight. Not new, per the user; a
  rendering fix/mitigation dogear, not a walk finding. Two candidate
  mechanisms, one discriminator:
  (a) **f32 precision exhaustion in world-position-derived UVs** — at
  |x| ≈ 99 km an f32 ULP is ~8–16 mm vs a 56 mm texel, so the U axis
  quantizes noisily while V (from y ≈ 198 m, µm-precise) stays clean —
  matches the anisotropy exactly; ARCHITECTURE's floating-origin rule
  protects geometry but not any shader path reconstructing absolute world
  position in f32;
  (b) **heightmap-based splat mixing** (user's hypothesis) — near-equal
  weights on 4 same-texture splats let the per-pixel winner flip; standalone
  it does not predict the anisotropy, but if the heightmap/splat selector is
  itself world-position-sampled, (b) inherits (a) and they are one bug.
  **Falsifier: press against a wall near world origin** — (a) predicts both
  edge directions straight there and band width growing with |x|; (b) alone
  predicts the artifact everywhere. Then read the WGSL UV/splat-height
  derivation. Candidate mitigations if (a): camera-relative or
  origin-rebased UVs, f64-split world coords on the CPU side, or
  per-region UV rebasing.

- **NO VOLCANISM — no constructive edifices anywhere in the world** (filed 2026-07-20 with the
  dismal-mountains thread). **⚠ THIS OBSERVATION HAD NO HEADER AND NO BULLET UNTIL 2026-07-29**
  (S6 finding F5, applied): its body sat *inside* the texel-edge-dither entry above, indented as
  a continuation paragraph, so it was invisible to anyone scanning entry headers and it was **not
  among the 129 entries** the baseline sweep counted. Nothing was wrong with the observation —
  only with its address. The only structural defect the sweep found in the section, and the reason
  an entry needs a header even when the text beside it is already correct.
  earth-processes § 2 (Igneous) is a sketch; nothing is built. Arc/rift
  provenance raises elevation but builds no edifices. What it would buy:
  the fastest legal short-gradation mountain on Earth (a stratovolcano is
  ~3 km of relief in a ~20 km footprint), calderas, lava caprock → mesas,
  ash beds as strata events, hotspot island chains. Constructive
  point-process that feeds the existing erosion sim naturally. Note:
  hotspot tracks REQUIRE plate motion — couples to the tectonic-history
  question (one-shot upheaval, below).
  - **Cross-ref:** this is a sibling of *"our dismal mountains"* cause 2 (no dip/fold) and of
    *"the 5–460 m band has no process"* — all three are **absences of a constructive or structural
    process**, not defects in one that exists.

- **The 5–460 m band has no process — only decayed noise** (agreed
  2026-07-20, session-4 gen review; the primary mechanism behind "locally
  everything looks flat", and it is world-wide, not a mountain thing).
  Below the deep sim's 460 m floor, elevation is midpoint jitter off a
  per-provenance roughness budget decaying ×0.55 per halving: total
  sub-460 m relief ≈ ±26 m in an orogeny, ≈ ±4 m at walking wavelengths,
  ≈ ±1 m on a craton. No gullies, ravines, outcrops, knickpoints, talus,
  or hillslope-scale stream incision — the band where travel gets
  interesting is empty. S9's C-refinement (bounded regional refinement,
  halo theorem, `deeptime/refine.rs`) was designed for exactly this and
  sits unbuilt; it is the ratified extension point, not a bandaid.

- **The next arc after planet gen: back to block primitives + the artful
  procedural dressing** (user, 2026-07-20, stated as anticipation — filed
  so the sequence is on the record). "Jittering structural partials,
  deposits of loose partial mixes from grinding and hydrology below
  surface, stepped partials for loose material landscapes (the topsoils,
  sand dunes, etc.)." This is the collapse-tier rendezvous of four filed
  threads: the dormant 3c-2 partial-height loose rendering (content/sim
  half missing), method rule 5 (unsimulated remainder → procedural
  tricks), the monolith problem (thick units need partings/defects), and
  the new eolian dune-FIELD regions (journal/0034) whose individual dunes
  are exactly the stepped-partial landscape the user names. Also where the
  sub-460 m process band's outputs get their look.

- **"Ours is not a deep world right now — you can keep digging into
  no-variety" (user field report, 2026-07-20).** Below the recorded
  deep-time strata the column is undifferentiated basement; depth is
  monotonous by construction. Connects to: recorded overburdens ≤ ~100 m
  (coal-rank note in dc-core geology.rs — the record itself is shallow),
  the § 5 burial/diagenesis/metamorphism engine (sketch), exhumation
  (ratified — will surface deep rock but also needs deep rock to BE
  something), and § 8 groundwater/karst (slated deep hydrology). The
  vertical dimension is a mostly-unbuilt frontier; no diagnosis filed
  beyond this inventory.

- **Gen never writes Quarter/Slab shapes or debris volumes (user,
  2026-07-20, "need to think on that" — deferred, filed).** The contents
  model supports partial structure shapes and loose debris eighths, and
  3c-2's partial-height loose rendering ships dormant — but worldgen emits
  only Full-shape voxels with no debris, so none of it is exercised in a
  generated world. This is the generation half of the filed
  block-primitives arc (stepped-partial loose landscapes: topsoils, dunes,
  talus; jittered structural partials at outcrops). Design owed: WHERE gen
  should emit partial shapes (weathered outcrop edges, scree, soil
  horizons) and what the deep-time/collapse seam is. Pairs with the
  sub-460 m process band and the pore-packability rule (materials.md).


- ~~**Charcoal's premise expired and the code still encodes the conclusion**~~ **✅ THE CODE NO
  LONGER ENCODES IT — verified at source 2026-07-29** (S6 finding F3; shipped by journal/0063).
  Both of this entry's stated blockers are false today:
  - *"no charcoal material is registered at all"* → `crates/dc-core/src/materials/mod.rs:133`
    defines `pub const CHARCOAL: MaterialId = MaterialId(MatRepr::M25)`, with the token
    `"dc:charcoal"` at `:213` and a registry entry at `:686`.
  - *"`deep_class` routes a charcoal-tagged unit to its mineral host"* →
    `crates/dc-worldgen/src/geology.rs:430` routes `Biofacies::Charcoal => CLASS_ORGANIC_CHARCOAL`
    (and `:422` for `Litho::OrganicCharcoal`), with the doc comment at `:364` and `:391` recording
    that the deliberate non-routing was retired. It is **loose-formed** too —
    `crates/dc-worldgen/src/fill.rs:419` puts `CLASS_ORGANIC_CHARCOAL` in `is_loose`, with
    `:408-409` naming journal/0063 as *"the whole substance of calling it an inclusion rather than
    a stratum."*
  The entry's wider point — *a conclusion justified by a constraint we later removed, recorded in
  prose, with nothing to fail when the constraint went away* — stands and is the reason it is kept
  rather than archived. *(Original diagnosis below, for the record.)* `geology.rs:312`
  deliberately does not route `Biofacies::Charcoal` to a class, reasoning:
  *"mean ~0.035 m over 158 310 beds, and none of them survives the 0.9 m voxel
  quantization… a charcoal member would be dead content. The honest
  representation is an inclusion (pore/debris partial) — filed, not built."*
  **The load-bearing clause is now false.** One eighth is 0.1125 m, so a 3.5 cm
  bed is 0.31 eighths, and since journal/0055 allocation is *unbiased addressed
  stochastic rounding* — so that share wins a real eighth ~31 % of the time.
  Charcoal is expressible today, as exactly the inclusion the comment
  describes. What blocks it is small and specific: **no charcoal material is
  registered at all**, and `deep_class` routes a charcoal-tagged unit to its
  mineral host — so the fire bed's metres already flow through the mixture
  path wearing mudstone's identity. **Fourth instance in one day** of the
  ARCHITECTURE.md § "A summary is not an authority" class: a conclusion
  justified by a constraint we later removed, recorded in prose, with nothing
  to fail when the constraint went away (cf. corrections #29).

---

## NEXT SESSION — written at the 2026-08-04 GEO-4 close (the geo thread's pickup; supersedes GEO-3 below)

**Read first:** sweeps hook → `docs/dependency-graph.md` (E4 / P11 / PARENT MATERIALS rows)
→ this block → **ROADMAP § Observed's two new top entries** (lateral quantization; E4-2's
verdict) → `docs/audits/2026-08-04-{s0-correlation-measurements,deposition-clock-design,
e4-2-convergence-study,voxel-explainability-audit}.md` → journals **0154, 0157**.

### The paragraph that matters
**The record learned what time it is, and the correlation rule got smaller because of it.**
The user asked one architecture question — *"they were laid down on the same clock. did we
throw the time away?"* — and it dismantled a design pick: the epoch had never been dropped
at packing, it had never been *handed to the recorder*, and chapter turned out to be a
lossy projection of the very clock we were reconstructing by inference. **The clock slice
shipped the same day at ZERO storage cost** (journal/0154; u24 thickness + u8 raw epoch in
the second word, which the merge key structurally never reads), and **S1's correlation
kernel then collapsed to a keyed join** (journal/0157): the partition is the sorted union
of the epochs the parents stamp, so pinch-out, P-4's onlap feather and seam-freeness all
fall out with **no branch in the file**. Five correlation picks ruled, all five E4 picks
ruled, and **the roster philosophy ratified** — materials are labels over regions of a
continuous term space, relations derived from declared axes, never hand-paired
(`materials.md` § DECIDED 2026-08-04 + the new `/roster` skill).

### Gate state — ⚠ PARTIAL, AND THE DEBT CARRIES
**`fmt --all --check` = 0 and `clippy --workspace --all-targets --release -D warnings` = 0
on merged main, verified.** The **test stage could not complete**: three attempts, three
`LNK1104` link failures on three *different* example binaries
(`denudation_probe`, `member_diversity_probe`, `pore_decorrelation_probe`) — each the one
the **parallel bodies session was running at that moment**. Probe examples carry
`test = true` (the probe doctrine), so `--lib --tests` cannot skip them either; a running
exe cannot be relinked. **This is machine contention in the shared `CARGO_TARGET_DIR`, not
a code red** — no test failed; nothing linked. **S1's batch debt is therefore NOT cleared
and carries to S2's arc-chunk gate** (its own boundary anyway). **A lib-only run DID complete: `--workspace --lib --release`
= 514 passed / 0 failed across 7 suites, exit 0** *(this line first read "296/0 across 6" —
a number quoted from a log that was still being written, corrected at close from the final
`LIB_EXIT=0` tail; the gate's own rule about reading the finished log, applied to its own
wrap)* (it cannot touch example targets), so the
library code — including S1's new `correlate` module — is verified; what is unverified is
the **integration + probe-gate layer** (S1's 10 correlation invariants live in
`tests/correlation.rs` and did not run here).
**First act next session: the full trio on a quiet machine, before anything else merges.**

### First things — ordered, and two are user calls
1. **The lateral-quantization ruling** (§ Observed, top). S2 must not wire the
   winner-take-all cut before it is settled. Everything else in S2 is ready.
2. **E4-2's fork** (§ Observed): re-price K2-alt vs K3 on the measured numbers, or park.
   U-1 pre-committed to neither, deliberately.
3. **S2 — the correlation wiring slice**, once (1) is ruled: `column()` integration, the
   ≤14-file `record_for` repayment, P-5's retirements (`cell_of`, the near
   `sample_source_cell` site, the `NearRecordMembership` domain), B-1 at the record extent,
   fold-per-parent, the veneer's placement (`Borehole::unclocked_m`), **stubs #54's
   deadline**, CONTENTS + SURFACE re-capture with **deep-time stillness asserted in the same
   merge**, full trio at the arc-chunk boundary. Then S3, the acceptance walk (Claude drives).
4. **P2's flip** is now blocked on the **M re-pick alone** (the pits bar died — corrections
   #98), sequenced with E4-3 so goldens move once — which E4-2's verdict now gates.
5. **PARENT MATERIALS** (§ Sequenced) gained a live blocker: P-1's continuum predicate
   cannot be built until materials declare **regions** rather than points.

### Rulings this session (user's words at the records)
Roster philosophy *"that's it. I'm onboard"* · P-1 M-C-derived · P-2 *"yes"* · P-3 dissolved
by the clock · P-4 *"an empty stack is a parent whose every chap thickness is 0"* · P-5
*"yes, ratified"* + the direction rider (*"re-implement the variety in our terrain as the
result of purposeful standardized operators… most refinement was old native code, not on the
plugin shape"*) · O-2b for the clock · U-1 K2-with-measurements (fallback **un-committed**) ·
U-2 *"always implicit, no dual-worlds from one seed"* · U-3 *"The engine owns primitives"*
(×3, emphatic) · U-4 reframed to **instrument-meaning continuity** · the pits premise
rejected (*"the dimple test does not make sense"*) · the five roster dispositions · model
tiering restored (opus for delegated work).

### Falsified — the assistant's own, first
**#98** the pits bar's unwritten premise (found by sweep, ruled by the user) · **#99** the
"measurable unconformity gap" — mine, published in journal/0154 and refuted by S0 the same
day (interior flags are structurally zero) · **#99b** offering a three-way fork whose
alternatives could not be chosen — *"i don't think this is a real fork"* · and, unfiled
because it was never recorded as a claim: my swap-mechanism hypothesis for E4-2's
monotonicity failure was **wrong** (the limiter chain is monotone; Picard is the culprit).

### Owed / unverified
- **E4-2's branch is unmerged** (`worktree-agent-a787b0d7fdeb61b53`, worktree kept): the
  kernel compiles, dc-core's 163 pass, **its two structural fixtures fail as designed-for**,
  fmt/clippy repaired mid-diagnosis. `IMPLICIT_ACCURACY_MAX_EDGE_COEFF` is still a flagged
  placeholder — **nothing was re-fitted**. § 5's world runs unrun.
- **stubs #52** (epoch immutable under overprint; alteration-time heir) · **#54** (mirrored
  bilinear weights — **deadline S2's merge**).
- The `/roster` skill's § 0 self-update clause is now load-bearing: it has been folded once
  (the bodies findings) and will need another sweep as the term-schema work moves.
- `collapse.rs` is 3,006 lines and `creep_operator_probe.rs` 1,069 — extraction proposals,
  not done mid-arc.

### Machine state at close
No agents running. **One worktree deliberately kept** (E4-2, above); all others merged,
removed, branches deleted. A parallel BODIES session shares the checkout — its
`journal/assets/0156-quant-*` and the `plugins/demo-builder/Cargo.lock` change are theirs,
untouched. **The geo journal was renumbered 0156 → 0157 at integration** to leave their walk
assets alone (sixth ordinal collision of the parallel pattern). Build slot free at close;
lock file is the hook's, left alone.

---

## ~~NEXT SESSION — written at the 2026-08-03 GEO-3 close~~ **SUPERSEDED by the 2026-08-04 GEO-4 block above** (the geo thread's pickup; the bodies block below remains THAT thread's)

**Read first:** sweeps hook (**staleness-sweep + doc-topology are OVERDUE FULL** — held
2026-08-03 morning by user call while bodies ran spine-audit; the delta since is enormous:
five merges, four design audits, two rulings-heavy days) → `docs/dependency-graph.md`
P10/P11/E4/P2 rows → this block → `docs/audits/2026-08-03-stratigraphic-correlation-design.md`
§ picks (P-1…P-5) + `2026-08-03-e4-implicit-kernel-design.md` § 8 (U-1…U-5) →
journal/0149 + 0150.

### The paragraph that matters
Five things shipped and one pivoted. **P11 slice 3** (journal/0145, `cd1058a`): per-column
record membership + packed `DepUnit` L-8, mover IN the key on M0's 1.0308×, accessor layer
throughout. **FS-A** (journal/0146, `4635056`): release spectra as pack-authored edge
products, 8 literature-cited tables — **the grain writer withheld on semantic grounds and
the user RATIFIED the withholding in conversation** ("the build agent made the right call —
propagated grain sounds like the actual source"); propagated grain is P10's live next
build. **P2 measurement runs** (`2026-08-02-p2-measurement-runs.md`): target doubly
confirmed (2.63 ↔ 2.653 from the world's own Airy), bracket low 1.6× (target M ≈ 375–380),
three flip blockers named. **E4 design + E4-1** (journal/0150, `62b0449`): the 38-min
gen-time is CFL tax; kernel extracted byte-identically to `dc-core::field` (**zero goldens
moved**, stubs #30 discharged). **And the pivot** (journal/0149): slice 3's membership
dither was walked, REJECTED live, and superseded-as-plan the same day by the user's
stratigraphic-correlation sketch + smoothness principle — *"there are no deep cells"*;
deep cells are boreholes; smooth interpolation is the unearned default and sharpness must
model an honest process. The correlation design pass is DONE (R-C shared-clock rule; mass
argument HOLDS; read-side, so deep goldens stay STILL through the build).

### Gate state
**Main is FULLY GREEN at `090f778`+: the wrap trio ran on merged main after `cargo clean
-p` of all six touched crates — fmt 0 · clippy 0 · `test --workspace --no-fail-fast`
= 984 passed / 0 failed / 94 suites, exit 0.** All recorded batch debt (FS-A's merge,
the recorder M-1 comment fix, E4-1's merge) cleared by this gate. No expected-red list —
a red anywhere is a real defect.

### First things
1. **Correlation picks, one at a time (P-1 first — it carries the two-user-rulings
   tension:** 2026-07-19 "dress every contact with noise/dither" vs 2026-08-03 "utterly
   smooth default"; mixtures vs octaves-cut vs the M-C hybrid). Then P-2 (inverted
   acceptance criterion) · P-3 (R-C vs R-C′) · P-4 (onlap feather at the record edge —
   the 0149 wall becomes a wedge) · P-5 (formal supersession ratification). Then the
   correlation build (read-side; **deep goldens must be bit-still** — the strongest
   tripwire; M0′ fill-cost measurement first).
2. **E4-2** (implicit scheme, non-default, convergence study; picks U-1…U-5 —
   **NEEDS RATIFICATION outstanding: the `dc-core::field` venue + its rayon rider**).
   E4-3 flips adoption WITH the P2 re-pick so goldens move once; the D3(M) ladder
   re-runs under the new integrator first.
3. **P10 propagated grain** — FS-A's ratified writer-owner; seam marked
   (`grain_write_seam` + `set_grain`); U2/U3 still open; split-factor gate binds here.
4. **P2 threads 2–3** (M re-pick · gen-time/register — mostly dissolved by E4) and the
   **pit safari** (walk-first ruled; cheap post-E4-3; census on the same world if wrong).
5. **The overdue sweeps** (staleness + doc-topology, FULL).

### Rulings this session (user's words at the records)
Record-terms ruling 3 ("Face", + the directional-provenance rationale) · U4 ("I agree:
5") · P-2 ("if it's cheap to replace 8, may as well start at 8") · P-3 + march order
("whatever's easiest… eyes on the prize") · U7 ("R2 it is — authored edges", + the
mods-generality rider) · pits approach ("go right to [the walk]… census on same world") →
deferred ("no time to block on 40min gen") · E4 ("e4 yes. queue right away") · dispatch
autonomy ("you don't have to wait on my word for a dispatch that has all of its calls
made" — memory pinned) · accessor calls endorsed repo-wide (memory pinned) · the
boreholes sketch + smoothness principle (memory pinned) · FS-A's withheld writer
ratified. Relayed from bodies: gravity is a world constant (their filing; geo checked:
zero worldgen passes consume g today).

### This session's numbered artifacts
Journals **0145, 0146, 0149, 0150** (0144→0145 and 0147/0148→0149/0150 renumbers — the
ordinal guard + wrap caught both collisions; bodies holds 0144/0147/0148) · audits
**2026-08-02-p11-slice3-design** (+ build banner), **2026-08-02-p2-measurement-runs**,
**2026-08-03-e4-implicit-kernel-design**, **2026-08-03-stratigraphic-correlation-design**
· assets 0149-interfinger-* (pose recorded) · memories: accessor-calls-preferred,
dispatch-without-asking-when-calls-made, smoothness-default-sharpness-earned · no new
corrections entries (all falsifications stamped at their sites: the 0141 § 11 pointer,
the collapse.rs:1459 citation, the underived 1 m hollow bound, the M-2 recorder comments,
the P2 bracket).

### Owed, small
File-size extraction proposals (hook-flagged on touch, none done mid-slice):
`material-behavior.md` cold half · `session-workflow/SKILL.md` liveness ·
`walk_tour_0115.rs` + `weather_inventory.rs` + `recorder.rs` (2.0×) + `erosion/mod.rs`
by-concern splits · ROADMAP itself (1.9× — archive resolved § Observed entries next
touch). E4-1b (`sat.rs` conversion) filed with a loud marker, no build order. The FS-A
walk waits for propagated grain (nothing visible until a writer exists).

### Machine state at close
No agents running; ALL geo worktrees merged, removed, branches deleted. Working tree
clean, **pushed** (`git status -sb` clean of ahead-markers at close). No cargo/rustc/
dc-client; port 7777 free; build lock hook-managed. A parallel USER session (bodies)
shares the checkout — explicit paths, never `add -A`; the ordinal pre-commit guard is
live and caught both of today's collisions. Gate logs: `%TEMP%\wrap-trio.log`
(UTF-16; the totals are quoted above and in the wrap commits).

---

## NEXT SESSION — written at the 2026-08-03 BODIES close (⚠ **THE PARALLEL PATTERN ENDS HERE — next session is SINGLE-THREAD** and holds BOTH this block and the GEO-3 block above)

**Read first:** sweeps hook → `docs/dependency-graph.md` § 2b (rows **B7**, **B8**, and **B6**'s
eight inbound edges) → this block → the GEO-3 block above → journals **0142, 0144, 0147, 0148**
→ corrections **#93, #94, #96, #97** (all four the assistant's own).

### The paragraph that matters
**Locomotion stopped being an authored clip.** A body's cadence, stride, duty, foot lift and
vertical motion are now computed from its own skeleton — shipped headless (0144), consumed by
the renderer (0147), and **walked with the user (0148)**. Verdicts: rest *"the bob is gone…
soles are planted"*; motion *"they genuinely look pretty good… the cadence difference is very
clear, i like it"*; the bob *"a tad exaggerated but — they're block people."* **We predicted
that bob verdict from the literature before taking the walk** — 6.6 cm compass vs 4.6 cm
measured (Saunders/Inman/Eberhart 1953) — which is the measure-against-the-literature rule
paying forward instead of backward. `Keyframe.root_bob_m` left the schema; corrections #80's
two-authority hover is now **unrepresentable rather than fixed**. Workspace **980 passed / 94
suites / 0 failed**.

### Ratified this session, in the user's terms
- **Look ownership** — move intent defaults to look-follows-travel; `set_look` HOLDS until the new
  `clear_look`. *"A definitely sounds like the right call"*, plus the multi-segment-neck
  compartmentalisation caveat, which the chain-solve design later honoured.
- **"It's not actually a knee until constraint is declared"** — the ruling that **dissolved** B7's
  J1 rather than answering it. Engine = declared limits plus whatever geometry supplies; magnitude
  in both directions is complete and honest. **Inheritance of fold sense is PACK behaviour**
  (copy-or-mutate the parent generation), correcting an assistant partition error.
- **Gravity is a WORLD constant defaulting to Earth** — *"nobody ever consciously chose 25 m/s²"* —
  widened same day to **the SDK surface, consumed by geo passes**. Three hardcoded `25.0`s
  collapsed to one authority.
- **Locomotion lean is NOT a posture** — a term in the posture solve; support is a MODE, timing is
  a GAIT, lowering is CROUCH, and what remains is one continuous lean.
- **Proportion variation: SIZE FIRST**, declared allometric axes later; range=engine,
  distribution=pack, **pack owns fairness**.
- **The sim owns the target; the client owns the approach** — *"good pattern to stick with in
  bodies"*, now its own `bodies.md` section with three instances.
- **Blast radius is not a reason for conservatism** at this stage — *"we can fix anything that
  breaks"* — explicitly **not** a retirement of the gates.
- **Three keyframes**, **`root_bob_m` out of the schema**, **`biped_walk` retired** (*"mostly
  because it's not worth doing anything with"* — parked, **not enshrined**), **root segment
  welded** (provisionally — see Owed).

### Falsified — the assistant's own, first
**#93** a fix-framing built on unratified content (the bob), which the board had carried and this
session propagated · **#94** *"look-at is outside the sim-visible set"* — false for the gaze,
which aims perception; one word, two mechanisms · **#96** *"you can't ship a broken main"* — a
real principle imported at the wrong scale, and the **second invented constraint presented as a
property of the problem** in one hour · **#97** *"a stand-in marked in code is filed"* — four B6
heirs shipped to **no locus**, caught by the user asking a question, **and the entry's own origin
date was wrong and had to be corrected within the hour** (they shipped in slice ONE and survived
the integrator's own gate, a merge, and a second slice).

### First things next session — single thread, and they are ordered
1. **B6 (per-segment materials/mass) is the strongest candidate on the board, and the argument is
   empirical:** at the 0148 walk **the user independently found THREE of its four stand-ins by
   eye** — the bob reading high, the missing push-off flexion, the run's absent vertical — without
   being pointed at any. Eight named inbound edges (graph B6). *This is a claim about priority,
   not a ruling — the bio/eco gate remains a user call.*
2. **The 12 fps call, now takeable.** Its stated precondition (*"a body whose feet actually reach
   the ground"*) was met at this walk, and the walk gave a **non-taste** reason: a fixed 12 fps
   **aliases** a derived cadence (stout 4.29 frames/cycle vs biped 6.97). **User leaning is
   FORFEIT and is recorded as a LEANING, not a ruling.** Keep the diagnosis either way: quantize
   per **CYCLE**, not per second.
3. **B7's build** — design done (`2026-08-02-joint-limits-b7-design.md`), all seven calls closed.
   Its **five stubs land WITH the build**, deliberately not before (they describe code that does
   not exist). Wants the pre-B3 window.
4. **The walk loop has no stop channel** — the user built a *wall*. Leash radius, bounded intent,
   or a freeze-all verb; none designed.

### Owed / unverified — read before trusting anything
- **The root weld is PROVISIONAL** against two live threads the user named: idle root-motion as a
  legitimate expressive technique, and — sharper — **the weld silently decides that death/knockdown
  must come from physics**, which was never discussed.
- **`swing_gain` (stubs #43) is a stand-in for a STOP TRANSITION nothing owns.** The design pass's
  `clearance` was speed-invariant; taken literally a body stopping mid-swing freezes with a foot in
  the air. Heir: the stop transition, which `posture-gait.md` § 4 explicitly does not design.
- **A control that diffs in-code stand-in markers against the three loci does not exist** (#97).
  Scope measured and left open: `heir` appears at 200+ in-code sites across ~40 files; grep cannot
  separate a real stand-in from prose. **Finding out is the task.**
- **`stubs.md` is briefed as off-limits to agents and the rule is too broad** — the hazard is the
  ORDINAL, not the content. Proposed narrowing: agents write the entry, the integrator numbers it.
- One **foreign uncommitted change**: `plugins/demo-builder/Cargo.lock` (+52 lines, a lockfile
  update). Not this session's and deliberately left — shared checkout.

### Machine state at close
**⚠ THE GAME IS DELIBERATELY LEFT RUNNING** (`dc-client` pid 50396; MCP dev 7777, character 7778)
**at the user's request, for the geo thread's walk.** The scratch world carries this session's
station: a cut stone bench with a ~63 m runway at **x −46…−24, z −72…−2, y ≈ 1015**, six parked
characters, and **a wall the user built** to stop them walking off a ledge. Scratch-pad doctrine
applies — it is not pristine terrain if geo wants an untouched station. All worktrees pruned; no
`.agent-build.lock`; working tree otherwise clean and pushed.

---

## ~~NEXT SESSION — written at the 2026-08-02 BODIES close~~ **SUPERSEDED by the 2026-08-03 BODIES block above** (the bodies thread's pickup; the geo thread's is the GEO-2 block above, its FINAL predecessor archived to history — both threads live, two parallel arcs)

**Read first:** sweeps hook → `docs/dependency-graph.md` § 2b → this block → journals
**0135, 0137, 0138** → corrections **#85–#87, #92** (all this session's own) →
`bodies.md` § Postures (the day's biggest ruling).

### The paragraph that matters
**The bodies arc went declaration → bake → renderer → walked, inside two days.** B0's
roles/modes/open-actions landed (journal/0135); the resting-posture bake shipped with its
design pass's predicted table confirmed to 1e-9 (journal/0137: biped 0.880 m, stout
0.440 m, longleg 1.020 m, knees 0°); the consumer slice put the derived hip on screen and
the user walked it (journal/0140; 0138→0140 renumber) — verdicts: rest *"soles look planted other than the
idle bob. reads right"*; motion *"reads fine with caveat"*. The caveat is the STRAFE
(Observed): the walk-8 fix inverts for a stale look — S6 row 95's "nobody owns body
orientation," OPEN since 07-19, remembered by the user before any sweep surfaced it.

### Ratified this session, in the user's terms
- **B0's shape** — with `collide` left OPEN for B4's machinery ("keep velocity up").
- **Postures are REGISTRY content by namespaced id** (sweep-checked first, per the user's
  procedural requirement): engine owns the contract, default pack ships the 2026-07-19
  ladder, controller surface = introspect + invoke-by-id — `bodies.md` § Postures.
- **The bake's venue** (corrections #86): *"the bake cannot run 'at pack build', or not
  only"* — every minting clock is a call site; evolution mints species in deeptime.
- **Gates batch per arc-chunk** (CLAUDE.md § Gates) — and its first bill arrived same day
  (the expected-red trio rode hours unseen; net judgment: still worth it, the debt-clearing
  run caught it).

### Falsified — all four the assistant's own, all caught by the user or by receipts
**#85** the liveness split axis emitted at source files (the hook's own comment refuted
it) · **#86** provenance: an agent-authored venue sentence attributed to the user ·
**#87** a dispatch REPORTED THAT NEVER HAPPENED — the rule: an agent is "running" only
beside its live receipt · **#92** a cross-arc finding filed without reading the other
arc's live close block (the tripwire trio was authorized expected-red all along).

### First things next session (bodies thread)
1. ~~**The look-ownership ruling** (user, small)~~ **✅ DECIDED + SHIPPED 2026-08-02
   ("A definitely sounds like the right call"): look-follows-travel default, explicit
   look held until `clear_look` releases — bodies.md § who owns the look (with the
   multi-segment-neck caveat), slice in journal/0142.** Original question: does move
   intent default to look-follows-travel unless a look is explicitly held, or do
   drivers own the look? Sibling of the postures ruling.
2. ~~**Corrections #80's temporal bob** — the last thing between these bodies and glued
   feet.~~ **⚠ FRAMING FALSIFIED 2026-08-02 (user, corrections #93): *"there has been no
   contemporary sketch or ratification related to the bob and i don't even know that it
   makes sense as something we want for all bodies, or uniformly for all bodies."* The
   authored clip bob is unratified bootstrap content (#81 already recorded it deleted —
   an output of the per-species bake); the bob's disposal is decided at the gait-bake
   design pass, not by a wiring fix. #80's space-layering FACT stands (solver hip vs
   render root, two expressions that can drift).**
3. ~~**The gait-bake design pass**~~ **✅ SHIPPED 2026-08-02 —
   `docs/audits/2026-08-02-gait-bake-member1-design.md`, FIVE user calls ruled in its header,
   BUILD NOT YET GREENLIT (the user is reading the pass first; member #0's precedent is an
   explicit greenlight before code).** It inherited the posture key, the landed rest pose, a
   motion verdict, and owned stubs #34. Outcomes: the **bob is not a term anywhere** — root
   height becomes one `root_offset(posture, mode, phase)` read by both the IK hip and the
   render root, so corrections #80's drift is *structurally impossible* and a body whose
   contacts never alternate bobs **exactly zero by construction** (the user's "not uniformly
   for all bodies", with no toggle); the shipped walk clip decomposed to **a coherent walk at
   1.84 m/s played at 4.5** (the skate), demanding **0.154 m** of bob against an authored
   0.040 (the hover from a third direction), and **internally inconsistent by 46 mm**; stubs
   #34 resolves as **one `BindTarget` across all four binders**. Seven bones-found-missing
   findings (G1–G7) — the "cautiously ratified" qualifier working. Zero CONTESTS.

### Owed / unverified
- **Workspace-green is blocked on P11 slice 2's single re-capture** (geo-owned; their
  verification was LIVE at this wrap — see their block below for the expected-red list).
- **Subagents die when they background their own gate** — two for two this session, both
  harvested by hand from worktrees (Observed; interim rule proposed: agents gate in
  FOREGROUND — fingerprint awaiting greenlight).
- `body.rs` grew again with the measurement series; the liveness/module-split family
  (session-workflow, ROADMAP registry at 1.7×) per the geo block's "Owed, small".
- Corpus sweeps were due-FULL at session start and deferred to the geo session — verify
  at next pickup which ran (their blocks reference doc-topology F1, so at least that).

### Machine state at close
Game closed clean (exit 0), port 7777 free. This session's worktrees merged and removed;
its three empty orphan dirs deleted. **The geo session is LIVE at this wrap** — its cargo
holds the build slot (slice-2 verification) and two of ITS agent worktrees exist
(`agent-a34dc…`, `agent-aa713…`) — untouched, theirs to harvest. The build-slot mutex is
a PreToolUse hook now (2f673ef): nobody touches `.agent-build.lock` by hand. Staging rule
adopted from the geo session's fingerprint: **explicit paths, never `add -A`** (this
session paid for it once: a stray client log).

---

- **PROBE EXAMPLES WITH `test = true` MAKE THE GATE UNRUNNABLE WHILE A SIBLING RUNS ONE**
  (geo wrap, 2026-08-04 — three attempts, three different exes). The probe doctrine puts
  `test = true` on measurement examples so the gate can see their assertions; the
  consequence, unnoticed until parallel sessions ran probes and gates simultaneously, is
  that **every probe binary is a gate build target**, and a *running* probe cannot be
  relinked (`LNK1104`). `--lib --tests` does not skip them (they ARE test targets). So a
  gate and a probe run cannot coexist in the shared `CARGO_TARGET_DIR`, and the failure
  wears a compiler error's clothes rather than announcing contention.
  **Interim rule: treat `LNK1104` on an example as machine contention, never a red — check
  `Get-Process` for the named probe before diagnosing anything.** Fix shapes, none owned:
  a per-session target dir for probe RUNS · the mutex hook covering probe execution as well
  as builds (it currently guards `cargo` invocations, and `cargo run --example` is one, so
  this may already be closable) · or a gate profile that excludes example targets when a
  sibling is live.
