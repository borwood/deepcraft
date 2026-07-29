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

- **The octree substrate — DESIGN PASS OPENED, D1–D3 DECIDED** (2026-07-22,
  live session; `docs/design/octree-substrate.md`). The water.md leaning is
  resolved: **one substrate = the existing S3 chunk pyramid**, named and given
  a payload contract (node = `(level, ChunkPos)`; first two consumers —
  renderer, water — define the payload; statistical/social recorded as
  intended extension). **First build slice ratified: FF2b-minimal** — the two
  existing reduction pyramids (block + `MixtureDownsampleRule`, spines § 3
  row 1) through FF2a's stepped mesher, coarse volumetric far chunks replacing
  the top-sheet-only far field; persistence + dirty-rail is the follow-on
  slice. **Stepped all the way** ratified for the far register (alternatives
  recorded in the doc § 5; vista-as-augury is complementary — it governs far
  *live* state, not terrain). This supersedes FF2b's earlier pairing with the
  caves/underground water thread for the *minimal* slice — the node contract's
  water stratum stays requirements-only and the hydrology pause holds. **Node
  contract v0.1 RATIFIED** (same session — the gap-hunt added two-sided
  derivation: reduce upward where children exist, synthesize top-down from
  the worldgen authority where they don't, statistical agreement where they
  meet; ungenerated ≠ empty; strata independent; seeded synthesis).
  Narrative: journal/0069. **FF2b-minimal LANDED** (2026-07-22, background
  agent, worktree branch — journal/0070): the two-sided derivation is real.
  `dc_core::farfield` carries the span-stack payload (`ColumnSpan` gains
  `bottom`; `quantize_top` moves to dc-core so synthesis and the streamer
  share ONE quantization); `dc-worldgen/src/far.rs` synthesizes nodes
  top-down from `coarse_surface` (seeded); the client's `FarPyramid` feeds
  every streamed chunk up the block + `MixtureDownsampleRule` pyramids
  (**spines § 3 row 1 consumed** — far spans render `classify` of reduced
  mixtures) and the FF2a mesher now speaks span stacks (bottom faces,
  interval walls) with an A-5 guard (`subtree_fully_inserted`) and a
  reduction standoff inside the near field's draw radius. Agreement measured:
  n=8192 columns, mean(reduced−synth)=+0.938 coarse voxels, 100 % within 1
  (the majority-vote-rounds-up vs floor mechanism, journal/0070). Tripwire at
  10 km (stretched L4): 1 648 tiles / ~168 MiB / ~4.9 ms per tile — mesh path
  nowhere near the Aokana wire; past ~10 km the answer stays "more rings"
  (journal/0042). Follow-ons unchanged: persistence + dirty-rail (far edits),
  synthesized sub-surface strata (the `surface_sample` summarization-half
  home), partial-coverage composition, deep-span greedy merge.
  **MERGED TO MAIN + LOOK RATIFIED AS-BUILT** (2026-07-22, integrator merge;
  user, from the 0070 screenshot set: "visually indistinguishable from the
  previous version, for me. which is good!" — the reduced-tile patches ride;
  the 10 km checkerboard is pre-existing, filed in Observed, and is B1's
  cure to claim; merged-main gate evidence in the merge-gate-0070 log).

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
  - **NEXT SLICE — the fractional top, and it is BLOCKED on a user-owned
    decision.** The eolian remainder is real ledger (2.09 m / 0.9 m → 2 voxels
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
- *(**Tectonics architecture RATIFIED 2026-07-20** — all of U1–U8, with
  U3 amended (ritual ceiling relaxed to "5 min if that's what it takes");
  see the tectonics.md banner. The SPIKE is next — sequenced below, behind
  the eolian agent's landing: both write `deeptime/erosion.rs`.)*
- **Tectonic design pass — DRAFT LANDED (historical entry)**
  (docs/design/tectonics.md, merged 2026-07-20; Fable design agent). The
  inversion: surface uplift stops being the input — plate kinematics
  (advected Voronoi seeds, K~8 chapters) drive analytic boundary forcing →
  crustal-column thickening → smoothed-load Airy isostasy derives
  elevation, buying rebound/exhumation/forelands from one mechanism.
  Deformation re-derived analytically at collapse resolution (no per-cell
  event storage); sparse event list for what kinematics can't re-derive
  (unblocks the § 2 igneous flag). Eight user decisions U1–U8 pending
  (plate-scale knob, chapter count, ritual-length fork, orogen widths,
  advection scale, punctuation budgets, amplitude re-sequencing, flip).
  § 14 corrections verified by integrator (corrections #20; Erosion::new
  uplift_sum cache; Large-extent province-density inversion). Next:
  user ratifies architecture → SPIKE per its § SPIKE.
- **Zonal circulation profile** — dispatched same moment (write-set
  disjoint from spike: climate.rs only). Smooth wind magnitude + subsidence
  aridity; kills the band-flip line and puts a desert belt at ~30° for the
  true reason. Changes every new world's climate — integrator presents the
  measured shift for the user's eye.

*(**Ore design pass: DRAFT LANDED** 2026-07-21, `docs/design/ores.md` —
engineering pass over the DECIDED 2026-07-20 roster; R1–R7 awaiting the user.
Two collisions reported for the record: the ratified lode-gold flagship vs
S12's metre-scale exhumation (probe-conditioned in R1), and BIF vs the
Phanerozoic register (reconciled in R3). Placer source-blindness filed as
stubs.md § 11.)*

**Slated by ratification 2026-07-20 (all six unknown-unknowns landed;
sequenced, not yet scheduled):** marker beds + punctuation event types ride
the volcanism design (earth-processes § 2 flag); ore-genesis roster is a
user-owned content design pass (earth-processes § "payoff layer") to be run
as main-session design conversation; impact structures ride punctuation
hooks.

**Session 3 shipped, all gates green on merged main:** FF2a voxel far field
(0023) · far-seam uniform-push fix (0024) · S10 biotic layer (0025) ·
organic materials + biotic flip (0026) · organics photo walk (0027) ·
S11 water locality + body graph (0028) · erodibility coupling (0029) ·
erodibility production flip + walk (0030). Eight journal entries; seven
corrections filed (#11–#19, two of them the assistant's own).

**Session 5 shipped (2026-07-21, journal/0039–0049, corrections #23–#25),
all gates green on merged main:** deep-config plumbing (0039) · walk 0040 +
the amplitude answer "neither" (#23) · S13 roughness measurement (0041,
#24/#25) · `--horizon` knob (0042) · climate registration fix (0043) · **U8
tectonic flip (0044)** · ore textures (0045) + substance redo (0048) · steep
walk 0046 · **full_agents flip + tour map (0047)** · **the first live
guided-tour ratification (0049)**. Doctrine: ledger-expression + enhancement
+ perf-first + genesis/stubs (stubs.md, 11 entries). Ore design pass drafted
(ores.md R1–R8 pending).

**(SUPERSEDED by the 2026-07-22 close at the end of this file.)** NEXT SESSION — rewritten at the 2026-07-21 session close, AFTER the live
walk (supersedes every earlier same-day block). Read first:**
**journal/0050–0058** — the leak, the eviction, the fill contract, carry-`H`,
DeviceLost, distribution-first, the holes, the eight-kilometre typo — and
**corrections #26–#29**, three of which are the integrator's own errors.
`docs/design/stubs.md` for the doctrine registry.
**Walk wiring:** game MCP is a checked-in `.mcp.json`; launch the game FIRST
(`cargo run --release -p dc-client -- --horizon 3`), then `/mcp` reconnect.

> **⚠ THE FIRST THING TO DO IS WALK.** Six merges today changed what the world
> is made of, how deep it digs, and what its surface looks like — and **the two
> most visible fixes (surface dither, the holes) landed after the user's
> session closed, so nobody has seen them.** Everything below is downstream of
> that walk. **Station coordinates are METRES** — pass them to `pose_set`
> directly; the integrator multiplied by 0.9 and spent an afternoon 8.6 km from
> every station (corrections #28).
>
> What to check, in order: (1) are the sky-holes gone (`--fullbright --edges`,
> re-walk `journal/assets/0056-holes-after-settle.png`'s view; if bands persist,
> distrust journal/0057 first); (2) are the chunk-shaped surface patches gone
> (compare against `0056-surface-quantized-per-chunk.png`); (3) does the
> **loess margin (82346, 24391) m** — the deepest section in the world, ~90
> sediment blocks, 76 mixed spans — read as *sediment* or as noise; (4) do
> contact bands change material on chunk lines (the boundary-dither loose end,
> still open for **mixed** voxels); (5) a long `--horizon 6` session — **DONE
> 2026-07-21, journal/0065: it survives.**

**Wide horizons: the blocker is GONE and now PROVEN at 6** (journal/0065,
2026-07-21). The DeviceLost crashes were host-RAM exhaustion from an unbounded
chunk store, now evicting (journal/0051, flat over 210 teleports at
`--horizon 3`). Re-measured at 6, in both regimes, alternated 6/3/6/3 against
machine drift: **teleport storm +0.542 / +0.529 MB/jump at horizon 6 versus
+0.531 / +0.542 at horizon 3** — the horizon signal is zero — and **idle at
horizon 6 drifts under 7 MB in seven minutes** with every probe count frozen,
which is the regime that used to die at ~4.5 min. Plus one unbroken
**700-jump / 23-minute** horizon-6 session: RSS sawtooths in a ~980–1 170 MB
band, long-run slope **+0.11 MB/jump**, 71 928 evictions, exit 0. All runs
exited **0**, no `DeviceLost`, no panic, no `ERROR`. Horizon 6's real cost is a **constant**
~155 MB / 704 resident far tiles (vs 288 at horizon 3), paid once at fill.
**Wide-horizon walks are unblocked**; 8–10 should hold, and the number to
watch there is `far_tiles` at fill, not slope.

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
1b. **WALK THE NEW WORLD — the biggest unratified appearance change this
   project has made.** Two merges changed every surface and every dig depth
   (journal/0053 carry-`H`, journal/0055 the record-skinned surface) and the
   user has seen neither. The tour map for 0053 is below; for 0055 the sites
   to add are the **bare granite shoulder** (101663, 5073 — basement at grade,
   with a horizon that agrees), the **dune field** (107183, 9672 — nine mixed
   spans where there were zero), and **any cut face** for the new contact
   voxels. LIT pass for shape and depth; `--fullbright` for the material
   question specifically. Both are revertible: one merge commit each.
2. **Ores R1–R8** (ores.md draft).
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

**⚠ UNRATIFIED APPEARANCE CHANGE AWAITING THE USER'S EYE (2026-07-21):**
carry-`H` (journal/0053) changed dig depth across the whole world and the
user has not seen it. Arid basins and dune fields went from 1–3 voxels of
dirt over stone to 8–12 voxels of loose fill; 0.2 % of land is now bare rock
at grade (e.g. 101663, 5073 — basalt, no soil). Soil depth now correlates
with erosion history instead of rainfall, which is the ratified *direction*,
but the magnitude and the fact that the world got **deeper on average rather
than barer** is the opposite of what the tour verdict anticipated. **Walk it
before ratifying** — it is one merge commit and trivially revertible.

*Tour map for that walk* (seed 1337 / Medium; regenerate with
`cargo run --release -p dc-worldgen --example soil_depth_probe`).

**⚠ UNITS — read before teleporting (corrections #28).** The station
coordinates below are **world METRES**, which is what `pose_set` takes, so pass
them **directly**. `soil_depth_probe::STATIONS` holds metres and *divides* by
0.9 to reach voxels; the integrator multiplied instead and walked every station
of the 2026-07-21 live tour **8.6 km off target**, then "verified" it by
confirming `pos_voxel` matched what he aimed at — a check that could not tell
the two hypotheses apart. The voxel address of a station is `metres / 0.9`
(loess margin = voxel 91 496, 27 101).

Use the LIT pass — this is a dig-depth/section question:
- **(101663, 5073) m — bare rock at grade**, `H` 0.17 m. The new extreme:
  basement at the surface, no soil at all. Did not exist before this slice.
  *Start here.*
- **(5993, 14732) m — tour station 1**, `H` 10.66 m. The station that motivated
  carry-`H` and moved the opposite way (corrections #26).
- **(107183, 9672) m — tour station 2 dune field**, `H` 7.99 m; the record
  holds 379 units that expressed as *zero* whole-voxel strata before
  journal/0055. Best place to see what the sieve was eating.
- **(82346, 24391) m — loess margin**, `H` 80.49 m → **188 voxel spans, 76 of
  them mixed, ~90 sediment blocks**. The deepest, richest section in the world
  and the best cut face available. *(Verified headlessly in journal/0058 — the
  "only 3 voxels here" alarm was the integrator standing 8.6 km away.)*

*(Superseded by a session-close block; kept for the record. **The pointer used to read
"above" and no longer resolves:** the block that superseded this was the 2026-07-21 one,
written when close blocks sat at the top of the board — it was consumed by its successors
and is not retained anywhere. The surviving blocks are § **NEXT SESSION** **below**
(2026-07-27, and the 2026-07-26 morning block under it) plus the six archived in
`ROADMAP-history.md`.)*
**RE-SEQUENCED 2026-07-21 by the journal/0040 walk.** The amplitude call is
**answered: neither 80 nor 160** (corrections #23) — `thickening_scale` acts
at ~25 km and above and buys *zero* sub-km relief, so it cannot fix dismal
mountains and no longer blocks anything. The order that replaces it:

1. *(**Roughness decay: MEASURED** 2026-07-21, S13/journal/0041. What remains
   is the **user's pick between three costed candidates** — see Sequenced.)*
0. *(**Steep-site re-walk: DONE** 2026-07-21, journal/0046 — the #25 debt
   paid, on a stock post-U8 production world. The scarp is legible (terraced
   stone basin at −77 m, dirt shoreline stripe, green rim; ~80 m/km of real
   deep-field relief); the crest is still the 0040 prairie from a 6 km
   horizon — both halves of #25 confirmed by eye. The distant lapse-rate
   banding (green→brown→stone) is visible at the horizon for the first time.
   The recalibration pick now has its context pictures: A/B would texture a
   plateau that has no shape to reveal; C is the only candidate that changes
   which landforms exist in the 460 m–7.4 km band.)*
2. *(**Far-field horizon knob: SHIPPED** 2026-07-21, journal/0042 — the
   landform-shape walk is now possible and is owed: re-walk the amplitude
   vantages at `--horizon 6` on the LIT pass.)*
3. **Erosion-supply calibration** (Sequenced) — S12's metre-scale exhumation
   finding, now co-equal with (1) as a relief-generating lever.
4. **The amplitude value itself** — deferrable. Rides as-built at 80 until
   (1) and (3) change what the knob is multiplying.
5. **Sim light SPIKE** — design pass is done (`docs/design/light.md`);
   § 10 of that doc states exactly what the spike must measure.

**Read first next session:** `docs/design/things-that-will-happen.md` (new
this session, and now item 2 in CLAUDE.md's read-first), then
corrections #18 and #19 — both are about choosing an instrument that can
see the question you are asking.

## Sequenced

- **✅ ALL FIVE USER DECISIONS RULED 2026-07-28 — journal/0120.** Surfaced, parked, and then
  taken in a dedicated unpacking session the same day. **Rulings are inline against each item
  below; the context is kept because the reasoning is what makes each ruling legible.**
  **What shipped from these rulings:** `worldgen.md`'s ON-HOLD banner + new § *Sequencing* ·
  `north-star.md` § *The core/plugin boundary* — **the refinement tier RESOLVED to (c)** ·
  CLAUDE.md's *recorded ambition vs unratified content* clause and the **immutable body,
  mutable header** spike policy (read-first item 5) · banners on `S10-results.md` and
  `S2-results.md` · `spines.md` § 3's **third exit** (held-as-candidate) · corrections #12's
  supersession note. **Nothing here is owed.** *Two follow-ups the rulings created are filed
  as their own entries below — the unswept-banner backlog and the completeness-check gap.*

  *Original framing, kept: five calls surfaced 2026-07-28, NONE of them the integrator's*
  (user-directed the same day: *"keep the user owed part durable with pointer to its necessary
  context… I won't muddy waters by addressing that in this conversation"*). **Deliberately parked
  by the user, not forgotten.** Consolidated into ONE live entry because the alternative — leaving
  them inside a `✅ DONE` block and a notebook — is the buried-in-a-closed-artifact failure this
  session spent the day measuring. **Context is inline, not merely pointed at**, so this entry is
  actionable cold.
  **Sources:** `journal/0121` (the removal's narrative) · `journal/corrections.md` **#66** ·
  [`docs/design/corpus-knowledge-notebook.md`](docs/design/corpus-knowledge-notebook.md) §§ 4–5 ·
  [`corpus-knowledge-evidence.md`](docs/design/corpus-knowledge-evidence.md) §§ 3.6g/3.6j/3.6k ·
  the `✅ DONE` removal entry below, which points here.

  1. **APPEARANCE — the shipped world lost 102 wood posts at four sites** (seed 1337,
     `Extent::Medium`). Verified gone: the 12 chunks that carried every post read **102
     `Block::Wood` → 0**, and a 200-chunk box around all four former clusters reads **0**.
     **No before/after screenshot pair is possible** — the "after" is ordinary ground — so this is
     a **notification for the record**, not a walk. Nothing else visible changed and **no golden
     moved** (corrections #66). *Why it is yours: an appearance change is ratified by the user's
     eye, and removal from what a player could walk into is an appearance change even when there
     is nothing to photograph.*
     - **✅ RULED — ACCEPTED (user, 2026-07-28).** *"That's fine and it's what I wanted. That
       system was not designed — it will have a designed successor at one point, but I had
       nothing to do with it. Came entirely from Claude bootstrapping the project, attempting
       to satisfy the list of things I mentioned that I would like to be in it **eventually**."*
     - **⚠ AND THE FACT THAT CLOSES IT, from the user:** *"I never once saw a wooden post in
       the world and no images were captured — it predates journals and our walk protocols."*
       **So the missing before/after pair was never a limitation of the removal.** The content
       was **never observed by the one eye that ratifies appearance**, across its entire life.
       *This is the sharpest available statement of "existence is not standing": a thing can
       render in the shipped world for a week, be defended by three independent instruments,
       and still have no witness. Appearance content that no one has ever seen has not
       accrued standing by surviving — it has only accrued inertia.*

  2. **DESIGN-DOC RULING — three live docs still describe civ/history as a real pipeline stage,
     contradicting the 2026-07-26 ruling.**
     - `docs/design/worldgen.md` § *"Below the region scale"* item 4: *"**History** — peoples,
       polities, trade, wars, migrations: dc-sim's coarse tier run over pre-player millennia."*
     - `docs/design/things-that-will-happen.md:126` — the sword looted from a ruin.
     - `docs/design/ideas.md:384`.
     Against the user's *"we do NOT have any form of evo/socia/civ modeling **even at the design
     stage**: they are NOTHING."* **The removal slice deliberately did not touch any of them** and
     was right not to: it could not distinguish a ratified user design from bootstrap text, and
     editing a design doc from inside an implementation slice is **corrections #65's exact failure
     mode**. *Why it is yours: which side of a user-vs-user contradiction wins is the one thing a
     sweeper is forbidden to decide (`doc-topology` § Rules).*
     - **✅ RULED — SPLIT THE THREE, and the split is a doctrine, not a tidy (user, 2026-07-28).**
       - **`worldgen.md` → ON HOLD, not struck.** *"History is coming, eventually, for the
         reasons worldgen states (not exhaustive)."* Shipped: a top banner, six marked sites,
         and a new § *Sequencing*. **The doc had understated the problem** — history is that
         document's *thesis*, not one bullet: its title, its core decision, its extent knob
         and its borders contrast all rest on it. **And one leg of the boundedness argument
         is the history requirement**, so § *The core decision* now records that **closure**
         is what carries boundedness today (the other leg is pure earth science and is live).
         Left unmarked, that is a ratified decision visibly resting on a suspended premise —
         A-2 waiting to be "discovered".
       - **`things-that-will-happen.md` → UNTOUCHED, and the doc's standing recorded.** *"Things
         that will happen is correctly 'what kind of engine this WILL BE and what kind of
         experience the default pack WILL BE'. The ambitions there are recorded with a high
         amount of user involvement and are **not claims about what we currently have built**
         as content or can support as an engine."* The looted-sword line **stays**.
       - **`ideas.md:384` → UNTOUCHED.** A mood line in a bullet about visual language.
       - **THE GENERAL RULE, now in CLAUDE.md:** the doctrine governs **unratified bootstrap
         CONTENT**, never **RECORDED AMBITION**. *The tell: does it RUN, or does it PROMISE?*
         Striking a future-tense user-authored line **retires a goal**, which no sweeper and no
         slice may do. **This is the second time in three days that this directive was about to
         over-reach by one word** — the first was `ecology.md`. Both were caught by the rule the
         directive itself sits next to.
       - **⚠ AND THE STATUS IS *ON HOLD*, NOT *NEVER*.** *"We do want these systems
         **eventually**: they are effectively on hold."* Read *"they are NOTHING"* as a claim
         about **what exists**, never about **what is wanted**.

  3. **SCOPE — dc-sim's entire S2 statistical tier now has ZERO production callers.**
     `pregen/history.rs` was its only one; removing it left `engine::{query, observe, force_fact}`,
     `Ledger` and `ToyWorld` reached by nothing but their own `s2_torture` / `s2_measurements`
     suites (new `spines.md` § 3 row). **The removal entry's scope line said *"dc-sim's
     region/agent-step draws"*, but those draws ARE `simulate_sample`** — so following the scope
     literally deletes `engine.rs` (528 lines), `world.rs`, both suites, and the artifacts
     `docs/spikes/S2-results.md` reports on. **The slice filed a DEVIATION PLEA instead**, which
     is the correct move. **The call:** keep the tier as a shape reference, or dispose of it as
     the same class of thing as the content it served? *(Note `S2-results.md` carries **no
     supersession marker at all** — the spike whose implementation just died has nothing on it.)*
     - **✅ RULED — KEEP BOTH THE PRIMITIVE AND THE TOY (user, 2026-07-28).** *"I didn't even
       know this system existed… The statistical system is genuinely intended, though I can't
       say whether as-is it will fit the desired shape when we actually do move on to
       implementing the civ/socia part of the default pack and the engine affordances."*
     - **The ruling's operative half is the NOTE, not the keep.** Whoever stumbles on this
       module — or is sent looking — must read: **the code that read it is gone · the primitive
       is the deliverable · it is a CANDIDATE to be re-checked against requirements that do not
       exist yet, never adopted on sight · and when that thread may open is a USER CALL.**
       Landed in all three places a reader actually arrives: the module doc
       (`dc-sim/src/statistical/mod.rs`), `S2-results.md`'s banner, and the `spines.md` § 3 row.
     - **`spines.md` § 3 gained a THIRD EXIT because of this.** The index knew *consumed* (the
       good exit) and, since journal/0121, *deleted*. This row is neither: **HELD AS A
       CANDIDATE** — ratified as wanted, with nothing yet to judge it against, so it is neither
       owed a consumer nor eligible for disposal. *Without the third state a reader assumes the
       first and goes hunting for a consumer nobody wants found.*
     - **`S2-results.md` now carries its banner** — which is also the first application of the
       decision-5 policy below.

  4. **SCHEMA — delete dc-sim's settlement/civ types, or keep them?**
     `Subject::{Site, Polity}` · `Aspect::{SiteExists, SitePolity, SiteEvent, PolityExtent}` ·
     `SiteEventKind` · `Value::{Exists, PolityRef, Event, Extent}`. **Producer-less since the
     removal.** Left in place and marked in-code as *not a schema to build on*, because deleting
     variants of a `Serialize` enum is wider than a content removal's scope. *Why it is yours:
     same doctrine as item 3 — unratified bootstrap schema has no standing, but the disposal is a
     scope fork.*
     - **✅ RULED — KEEP, with item 3 (user, 2026-07-28).** It rides the same ruling: the tier
       stays as a candidate, and its vocabulary stays with it. **But the in-code marking is what
       carries the standing** — it is **producer-less example vocabulary, NOT a schema to build
       on**, kept because deleting `Serialize` variants exceeded the removal's scope, *not*
       because anyone ratified it as a design. Re-stated in the module doc and the § 3 row so
       the distinction survives without this ROADMAP entry.

  5. **POLICY COLLISION — is a spike-results doc immutable testimony, or live authority?**
     `corrections #12` states the policy: *"`S10-results.md` is **left unamended** — a spike result
     is a dated record of what was measured; **this entry is the pointer**."* `CLAUDE.md`
     read-first item 5 states the opposite: *"Spike results live in `docs/spikes/S*-results.md` —
     **measured numbers, don't re-guess them**."* **Both are reasonable, they are incompatible,
     they live in different files, and nothing has ever reconciled them.** The concrete cost is on
     record: `S10`'s cost table is **~2× the real production cost** (25.19 s claimed vs 13.79 s
     measured — the spike drove the *scalar* path, production takes the *parallel* one), **a user
     ratified a ship decision on it**, and `S10` holds no reference to its own correction.
     Measured corpus-wide: **8 of 15** full-path correction→file edges are one-directional, and
     **14 of 30** audit/spike files carry no staleness marker of any kind. *Why it is yours: this
     is a choice between two ratified-feeling policies, and it changes what read-first means.*
     - **✅ RULED — IMMUTABLE BODY, MUTABLE HEADER (user, 2026-07-28).** A spike's measurements
       are **never rewritten** — testimony about a day is not edited — but a results doc **must
       carry a top-of-file banner pointing at whatever refuted, superseded or re-scoped it.**
       Recorded in **CLAUDE.md read-first item 5** (the site that asserted the losing half) and
       as a supersession note on **corrections #12** (the site that asserted the other half).
       Neither is rewritten; both now agree.
     - **The obligation lands on the WRITER OF THE CORRECTION, in the same commit.** *That is
       the property doing the work, and it is chosen from measurement rather than taste: a
       convention survives when it is inseparable from an act the author must perform anyway,
       and dies when it asks them to restate something in a second notation. Stamping the target
       happens while both files are already open. The counter-example is on the record —
       `JUSTIFIED-BY`, documented in two places with a promised sweep, got **3 uses, 0 in
       `crates/`**.*
     - **Applied where a refutation is already known:** `S10-results.md` (→ corrections #12, the
       ~2× cost table a **user ratified a ship decision on**) and `S2-results.md` (→ the item-3
       ruling). **Not a sweep** — see the backlog entry below.
     - **What it actually fixes is structural:** *a one-directional pointer is not a pointer.*
       The stale end is exactly where a cold session enters, and a chain of authority cannot be
       walked from an end that holds no link.

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
  UNAPPLIED** (2026-07-28, `docs/audits/baseline-2026-07-28/` — nine audits, ~3,900 lines,
  watermark `f652b60`). **The audits ARE the record; this entry is the index.** Applied at
  baseline: only the integrator's own same-day stalenesses + the `ARCHITECTURE.md` ON-HOLD
  banner. *Everything below is real, cited `file:line` on both sides, and NOT yet fixed.*
  - **✅ ALL FOUR USER CALLS RULED 2026-07-28** (unpacking session; applied to
    `ores.md`, `worldgen.md` § Sequencing, `ecology.md`, `stubs.md`):
    1. **BIOLOGY — the two decisions were never in conflict; the missing word was *building*.**
       The deep-time biotic layer stays **ON** and rides as-built — *"biology in this sense is
       seamed with heir"* — but *"**we aren't building bio-based rock formation any longer**
       until bio/eco stuff, which is waiting on the rest of the non-bio earth science stuff +
       engine capabilities."* **No new bio-driven rock-formation work opens before the gate.**
    2. **🔴 ORES — A LOAD-BEARING PREMISE IS REJECTED, and this is the biggest of the four.**
       *"We do not need to have ore 'exposed' — the default plugin pack will ship a voxel game
       **with digging**… absolutely no reason to treat it like everything needs to be
       discoverable on the surface. Weird and misconceived and likely very relatively old."*
       **Kills every "illegible until exhumation increases" caveat, the lode-gold A/B fork, and
       `probe 3` — the probe measured the wrong thing and is NOT owed.** *Exhumation stays real
       for **genesis honesty** (where an ore forms); what dies is exposure as a precondition for
       shipping one.* `ores.md` is **conceptually behind `materials.md` / `material-behavior.md`,
       which win on disagreement**; a revisit is owed and unscheduled. **⚠ Note how it survived:
       the assumption was never stated as a decision — it rode inside *measurement caveats*,
       which read as evidence rather than as premises, and held a user fork shut for a week.**
    3. **`material_transport` — RATIFIED as-is.** The user is already running a string of work
       on it and confirmed `COMPETENCE_SCALE`'s *"mud, sometimes"* is **not** to be treated as a
       knob to tune (it is downstream of the denudation rate; tuning it would be a number
       pretending to be a mechanism).
    4. **THERE IS NO "GENERAL REGISTRY" AND THERE NEVER WAS — the clause is DELETED, not
       reworded.** Surfaced by the user asking *"I honestly don't understand what the registry is
       supposed to be except for a list which we can extend."* **Correct — and nobody ever
       proposed one.** It was an inference that implied future work.
       - **A seam's success condition is that it DISAPPEARS.** The heir *replaces* the slot; it
         does not fill it forever. **Only completed case:** `burial_temp_c`'s heir turned out to
         be a **field**, so it retired as a **field pass** and left the file (journal/0093) —
         *"the answer was 'this is not a provider at all — it is a field.'"* `depth_to_water` is
         documented as heading the same way. **You do not design a third-party declaration for a
         pattern whose job is to vanish.**
       - **⚠ THE ROOT CAUSE — "slot" means two unrelated things**, sharing a code shape
         (`Option<fn>` + identity) and nothing else: **provider seams** (world-level, scaffolding,
         *temporary*) vs **material behavior slots** (`north-star.md` § Materials — what a
         content author writes, **the SDK surface, permanent**). north-star called the latter
         *"the `Providers` pattern **generalized** from world-level to material-level"* — true of
         the shape, **and read as the world-level SYSTEM being promoted into the SDK.** That
         misreading produced the phantom. **north-star now disambiguates it in place.**
       - **Where world-level seams land post-split is UNDISCUSSED and deliberately UNDECIDED**
         (user: *"I genuinely don't know… I don't think anyone has had a direct conversation about
         it… I don't want to burden us with more half-baked designs"*). **No decision is owed.**
       - *The assistant proposed a "policy injection" counter-argument and it is **dropped, not
         recorded** — neither party could name an instance, and writing down a hypothetical that
         shapes future thinking is the thing being avoided.*
       - **⚠ The integrator's OWN first two fixes of this were also wrong**, both from the same
         ambiguous north-star sentence (*"plugin-authorable engine sockets"*). Left visible at
         `doc-topology/SKILL.md` because being wrong twice from one sentence is the argument for
         disambiguating it. **Propagated to all six citation sites** — `stubs.md`,
         `providers/mod.rs`, `north-star.md`, `doc-topology/SKILL.md`,
         `corpus-knowledge-notebook.md`, `corpus-knowledge-evidence.md`. Journals **0060** and
         **0120** quote the old clause and are **left untouched: dated testimony, immutable body.**

  - *Original framing of the four calls, kept for the reasoning:*
    1. **`ecology.md:262-263` (DECIDED 07-20, user) vs `worldgen.md:189-191` (DECIDED 07-28,
       user)** — biology is *"a shipped part of world generation"* (confirmed live,
       `deeptime/field.rs:288`) vs *"engine + non-bio earth science → **then** ecology"*.
       **Both yours, eight days apart, reconciled nowhere.** Likely resolution: the S10
       deep-time biotic pass ≠ the ecology *design* pass — **but that sentence is written in
       neither doc, and it is not the assistant's to write** (corrections #65).
    2. **`ores.md`'s lode-gold NEEDS-RATIFICATION fork is held shut by an expired caveat** —
       *"until the erosion-supply calibration lands"*; **it landed 2026-07-26** (corrections
       #56, journal/0114). *A user decision has been available for two days and the doc says
       it is blocked.*
    3. **`material_transport: true` is the shipped default with no ratification record found**
       — not in the archive, the close blocks, or journals 0110–0112, while the board flags it
       `NEEDS RATIFICATION (user-owned)`.
    4. **`stubs.md:20-24` says "four conversions is not enough to design a registry from"; a
       fifth landed** (`:196-201`, journal/0078). That clause is quoted as binding doctrine in
       three places **including the argument for not designing the knowledge layer yet**, so
       whether five changes the judgement is a user call.
  - **🔴 HIGHEST BLAST RADIUS, integrator-applicable:**
    - **`spines.md` § S-6 still teaches order-derived-by-topo-sort as the exemplary compliant
      shape** across 148 lines, and does **not mention the 2026-07-26 authored-order decision
      anywhere** — no strike, no banner, no § 4 entry (verified by pathspec). **Read-first item
      0b; every brief that "names its shapes" has been naming a retired one.** *corrections
      #65's geometry, recurring inside the index built to prevent it.*
    - **The erosion axis is marked settled and is not.** `ROADMAP.md:2513-2533`, live
      `NEEDS RATIFICATION`, no banner: *"nothing further to ratify on the erosion axis."* Its
      null came from a probe **blind to `diffusion`** — 96 % of export (journal/0111:256-260,
      corrections #56) — so **the methodology is defective regardless of calibration**, and
      journal/0114 measures relief **+18 % at 100×**. ⚠ **But do NOT restate it as "the
      landscape is supply-limited today"**: that holds for the *calibrated* world, and
      `calibrated_rates` ships **OFF** (`walk_tour_0115.rs:150` asserts it). *Two agents each
      had half of this; the split matters.*
    - **`tectonics.md` (953 lines, largest design doc) carries NO staleness banner** and its
      § 7.3 still specifies the retired receiver tree as *"the carving source, one authority"*.
    - **`spines.md`'s "34 seams inventoried; 5 converted" is wrong — the inventory says 31**,
      and that figure is promoted as *the only obligation ledger with a denominator* in the
      docs-ops argument.
    - **ROADMAP's FIRST BULLET (`:33-38`) asserts the trusted/untrusted backend tiering** that
      north-star § Deviations 2 calls *"EXPLICITLY NOT THE MODEL"*.
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
  - **⚠ NOT COVERED BY THIS BASELINE: `spine-audit`'s question.** All nine slices were
    docs-vs-docs. **Nothing checked `spines.md` against the CODE**, which is why its watermark
    is deliberately `null`. *That is the next sweep, and it is the one with a live finding
    already waiting for it (S-6 above).*

- **✅ SWEEPS RUN FIRST THING, INCREMENTALLY, AND THE HARNESS SAYS WHICH ARE DUE**
  (**DECIDED 2026-07-28, user**: *"sweeps should probably run first thing… additional sweeps
  should be able to focus mainly on new stuff since last time, or full audit if the underlying
  source has moved (update to spines, etc)"*).
  - **THE DIAGNOSIS THAT PRODUCED IT.** This repo has four corpus controls and **three share one
    trigger: the main session remembering.** Measured over eleven active days — `spine-audit`
    left **zero** artifacts despite its own *"run a few times a day"*; `doc-topology` ran **once**,
    the day it was created; the staleness sweep ran **twice** and was never even a skill. **The
    filesize hook is the only control not gated on memory, and the only one that fires
    reliably.** *And the remedy on file — "make the staleness sweep recurring **like
    spine-audit**" — was wrong in an instructive way: it assumed skill-packaging produces
    recurrence, and `spine-audit` is the disproof. **A skill still waits to be invoked.***
  - **SHIPPED (a): `staleness-sweep` skill** — the procedure existed since 2026-07-24 with two
    worked audits; this is packaging, and packaging alone was explicitly **not** the fix.
  - **SHIPPED (b): `SessionStart` hook** (`scripts/sweep_due_hook.py`) — states which sweeps are
    **DUE**, in which **MODE**, and **WHY**, computed from `docs/audits/.sweep-watermarks.json`.
    **This is the half that satisfies CLAUDE.md § Gates' rule** — *do not answer "the gate cannot
    see X" with a rule asking people to remember X.* **It reports; it does not dispatch** — an
    agent cannot self-dispatch and the spend is the session's and the user's call.
  - **THE RULE THE USER'S TWO HALVES IMPLY, stated so it is checkable:** a sweep's **incremental**
    mode is valid only while its **REFERENCE side** is unchanged; when the reference moves, every
    prior verdict was made against a different rule → **FULL**. `spine-audit`'s reference is
    `spines.md` (mechanical: does the diff touch it). The staleness sweep is *inherently*
    incremental — but goes full when a **recalibration** lands, because one commit can turn a
    whole cohort of old observations into artifacts (journal/0111) and no per-entry reading finds
    that. **`doc-topology` has NO reference side** and its unit is a *pair*, so incremental there
    is **new × ALL**, not new × new — done as: read changed docs in full, then let **their nouns**
    drive the search across everything else. *That inverts grep's known weakness — the search
    terms come from the diff rather than from the reader's suspicion.*
  - **The watermark is written BY the sweep, in the same commit as its audit** — never a separate
    step. Chosen from the measured adoption law: a convention survives when it is inseparable
    from an act the author must perform anyway (`JUSTIFIED-BY`, which asked for a restatement,
    got **3 uses, 0 in `crates/`**).
  - **⚠ HONEST LIMIT:** staleness is **3–8 %** of recorded failures; ~91 % were wrong the day
    they were written. **A green sweep must never read as "the corpus is sound."**
  - **Deliberately NOT built:** no claims index / knowledge graph — the noun-driven incremental
    mode needs no durable artifact, and *"the corpus already authors the graph; nothing reads
    it"* plus "don't build the general mechanism first" both bind here.

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
  - **✅ DONE 2026-07-28 — the dangling cross-file pointers.** The audit estimated *"~25"*;
    the real count is **44 pointers at 43 sites**, all fixed. **The audit undercounted by 18.**
    - **23 in `ROADMAP.md` pointing at the archive** (audit said 16): 22 of the
      `"see § Shipped"` form, plus the last line of the 2026-07-26 morning close block —
      *"the 2026-07-25 block **below**"*, which was one of the six archived, so the live
      board's own close block pointed past its own end.
    - **19 in `ROADMAP-history.md` pointing back at the live board** (audit said 9).
    - **4 were already directionally wrong BEFORE the split** and are now doubly wrong: the
      roughness-decay entry's *"Sequenced below"* read from inside § Observed, and the three
      `NEEDS RATIFICATION (below, § Sequenced)` markers on the 2026-07-20 erodibility / water
      / biotic Shipped entries. *§ Sequenced has sat **above** § Shipped since at least
      `34d88f2`; these predate the archive, which merely made them unresolvable.*
    - **The `:515` orphan — *attributed*, not guessed.** The block it meant was the **2026-07-21
      close block** (verified at `34d88f2`, where it sat 19 lines above), and that block was
      consumed by its successors rather than archived — so it exists nowhere, and the line now
      says so instead of pointing at a block 3,800 lines the other way.
    - **The retargets are addresses only.** No claim was resolved and no contradiction
      adjudicated. Where a Shipped entry has a **journal number** it is kept as the stable
      pointer, per the archive's own design.
  - **✅ DONE 2026-07-28 — `flow.md:715`'s RATIFIED stamp.** The stamp is legitimate for
    § 11's WINDOW decision (DECIDED 2026-07-25, user); what wrongly inherited it was the
    *setup sentence* characterising the other two axes. The ORDER half is now **struck and
    marked SUPERSEDED 2026-07-26**, in the shape `material-behavior.md` § 5's ORDER bullet
    already uses, with a note on why § 11.1's argument survives the strike (it needs only
    that the scheduler had **no name for the window**, which holds either way).
    *Still open from the same finding (#15), deliberately not taken here:* `flow.md:459` and
    `material-behavior.md:367-369` carry the same `ORDER (topo-sort)` framing **without** a
    ratification stamp.
  - **✅ DONE — `spines.md` + `stubs.md` scheduling heirs for the removed bootstrap content.**
    Both resolved *into* the removal rather than surviving it, as this finding asked.
    `stubs.md` § 1 is now **RESOLVED BY DELETION** (journal/0121) and — the more useful half —
    was rewritten to carry the doctrine it cost: *a stub entry is not neutral about its
    subject's standing, it **asserts** it; ask the standing question before writing an heir.*
    `spines.md` § 3's row was replaced by the S2-tier row and, on 2026-07-28, given the user's
    **held-as-candidate** ruling plus the § 3 intro's **third exit**. *Note the shape: the
    finding asked for two deletions and what shipped was two doctrine changes — the heir lines
    were correct on their own terms, and the defect was the inventory's grammar, not the rows.*
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
  - **NEXT SWEEP'S SPINE: `ROADMAP.md` § Observed (~1,970 lines)** — the largest unswept
    surface in the corpus, and the section the archive structurally could not reduce.

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


- **✅ DONE 2026-07-26 — THE DOC-TOPOLOGY SWEEP + THE ROADMAP ARCHIVE** (user-directed at the
  close: *"our docs ops past critical mass, causing information loop to fail to close often"*
  — sequenced and executed the same session). *Entry kept for its reasoning, because the
  reasoning is what makes the sweep recurring rather than a one-off tidy.*
  - **THE DIAGNOSIS SEPARATES THREE FAILURES THAT LOOK LIKE ONE.** journal/0119's collapse
    was **not** a volume failure: the sketch (`ideas.md`), the reconciliation
    (`material-behavior.md` § 5), the refutation (`journal/0090`) and the unbuilt RATE axis
    were **all in the corpus, all findable by grep**. What failed is that **no reader ever
    had all four in view at once**, because the connections run *between* documents.
    - **VOLUME** — the board is grepped, not read, and grep returns only what you already
      suspected (`DeepField::chapters`, three spine audits). *Real, and third in value.*
    - **TOPOLOGY** — nothing compares docs to *each other*. `spine-audit` checks docs vs
      **code**; the staleness sweep checks entries vs **newer work**. **A claim and its own
      refutation can coexist forever.** *This is the one that cost an architecture.*
    - **AUTHORITY** — an assistant reconciliation could silently supersede a ratified user
      design. **Fixed the same day** by one CLAUDE.md rule; cheapest and highest-value of the
      three.
  - **SHIPPED (a): `.claude/skills/doc-topology/SKILL.md`** — five contradiction shapes in
    value order, prioritised by **blast radius not age**, requires `file:line` on **both**
    sides of a pair, and forbids the sweeper from resolving what it finds (which side wins is
    frequently a user call — that is *why* it survived). Provenance breaks ties:
    *user-originated constraints are data; assistant-originated ones are hypotheses that
    happened to survive.* **Run after any batch of merges that ships an arc**, and whenever a
    design thread reopens something old.
  - **SHIPPED (b): `ROADMAP-history.md`** — archived **by STATUS, not age**. Shipped (2,297
    lines) + six superseded close blocks moved out; the live board went **7,210 → 4,410**.
    Each Shipped entry keeps its **journal number** as the stable pointer, and the journal
    already holds the narrative. *Age is the wrong axis: a two-week-old `Observed` may be the
    liveliest thing on the board.*
  - **⚠ HONEST LIMIT — the archive buys less than it looks like it does.** 4,410 lines is
    still past reading whole, and **neither half of it would have prevented journal/0119**.
    The remaining volume lever is `Observed` (1,969 lines), which is *not* archivable by
    status — an Observed entry is live by definition. **Left open deliberately**: the
    thresholds and the split conventions are the user's to set, not the hook's provisional
    guesses.


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
    - **⚠ ONE FORK THE DOCS DO NOT SETTLE — NEEDS RATIFICATION.** *"A coarse-rate pass does not
      fire at epoch 0"* is a property of the **runner**, not of any declaration. It was written
      for the three passes that genuinely **are** seeded before the loop (`climate`, `geotherm`,
      `head`), where firing at epoch 0 would redo the seed. Now that a world can author a coarse
      period onto **any** pass, that rule silently also says *"and your re-rated erosion pass
      does not run in epoch 0"* — which nobody decided. The likely fix is a declared `seeded`
      flag beside the cadence. **Kept exactly as it was**, because changing it moves the shipped
      world; marked in `runner.rs::DeepPass::fires`.
    - **⚠ EXTRACTION CANDIDATE, NOT TAKEN (user call).** `runner.rs` is **1,694 lines** against
      the 700-line threshold (2.4×) and this slice added to it. The cold half is the
      journal/0090/0104/0107 declaration-history commentary; the live half is the roster + the
      loop. *Not split mid-slice, per the hook's own instruction to propose rather than do.*

    *Record of how the slice was framed before it shipped, kept because the sequencing argument
    is the reusable part:* ~~with the creep limiter as its acceptance test~~ — **the acceptance
    test was met on 2026-07-29 by a stand-in, and that changed the slice for the better.**
    - **What happened.** journal/0122 fixed the creep blocker by **sub-cycling inside the
      pass**: `n = ceil(max_cell eff_diff / CREEP_MAX_EDGE_COEFF)`, derived from the von
      Neumann bound `a = 1/8`. Grid-scale oscillation is gone (surface concavity ACF(1)
      **−0.867 → +0.185** calibrated; closed hollows past 10 m **818 → 12**; five safe
      multipliers where journal/0114 found none). **The acceptance criterion above is
      discharged — by a `dt` the pass computed for itself because the engine has none.**
    - **So the slice is now sharper, not gone: replace the hand-rolled sub-cycle with the
      engine axis, and hold the goldens.** This is strictly better than the original framing
      — the axis now lands against a **known-good fixed point** instead of against a defect,
      so "did RATE reproduce it" is a hash comparison rather than a judgement call.
      `stubs.md` § 30 carries the stand-in with RATE named as its heir.
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
      the analysis once, in one pass. **Every future field pass that diffuses anything has
      the same trap and no defence.** That is an engine-shaped hole, and `spines.md` § S-10
      names the shape it belongs to — with the sub-cycle explicitly excluded from the spine
      *because* it belongs here.
  - **CONTINUATION SLOT** (this is a slice OF *"passes are plugins and the engine is
    agnostic"*): after RATE — **(b)** authored order + the validator (and with it the
    retirement of the revision chain); **(c)** the open resource vocabulary, `DeepAxis`
    deleted; **(d)** epochs declared with pass members + chapter count **or a terminating
    condition** (the sketch's other half, never built); **(e)** the vocabulary split
    question — whether schedule coordinates and world resources need to be different
    *kinds*, or whether authored order dissolves the distinction entirely. **Do not close
    the arc when RATE lands.**
  - **⚠ THE PROCESS FAILURE IS PART OF THE RECORD.** The user's sketch was **reconciled, not
    contested** — its ORDER half died inside an implementation slice, in one clause of a doc
    the user does not read. New CLAUDE.md rule: *a user-originated design may not be
    superseded by an implementation slice.*

- **✅ DONE 2026-07-28 — THE BOOTSTRAP HISTORY CONTENT IS REMOVED** (journal/0121,
  **corrections #66**; merged and gate-verified on main: fmt 0, clippy 0, **85 binaries / 808
  passed / 0 failed / 3 ignored**, reconciling exactly against the 86/811 baseline — −1 binary
  (`s7_handoff.rs`) and −3 tests, **all three named and confirmed absent**). Net **−713** Rust
  lines. *Entry kept below for its reasoning; two findings that OUTRANK the removal are recorded
  here because neither was suspected:*
  - **⚠ NOT ONE GOLDEN MOVED — corrections #66.** This entry said *"The goldens will move and
    that is correct"*, and corrections #64 said the fingerprints are *"structurally downstream of
    the posts"*. **Both false, and falsified by measurement, not argument:** 0 wood voxels in
    `contents_contract`'s sample set pre-removal, and `geology`'s sampler covers `cz ∈ [−20, 24]`
    while the nearest post sits at `cz = −727`. All four byte-identity goldens ran and passed
    **by name** on merged main. **The transferable lesson: *a pre-authorised golden move is
    indistinguishable from an unexplained one, which is the opposite of caution.*** A new A-2
    sub-shape — not a justification outliving its premise, but a **permission** outliving its
    justification, and never true. *`spines.md` A-2 records it: "structurally downstream is a
    statement about the call graph; whether a fingerprint moves is a statement about which chunks
    the sampler visits" — the same reflex produced both halves, six lines apart, in the entry that
    named the reflex.*
  - **🔴 FOUR OF THIS SLICE'S FINDINGS ARE USER DECISIONS AND LIVE IN ONE PLACE:
    § Sequenced → ~~"USER DECISIONS OWED"~~ **"✅ ALL FIVE USER DECISIONS RULED 2026-07-28"**
    *(pointer repaired 2026-07-29, baseline sweep S5/F9: no heading named "USER DECISIONS
    OWED" has ever existed, so a reader greping the quoted string found only the two pointers
    and never the target — and the target is now RULED, not owed)*
    (items 1–4: the 102-post appearance notification · the
    three design docs that still describe civ/history as a live pipeline stage · dc-sim's S2 tier
    now having zero production callers, with the slice's deviation plea · the producer-less
    settlement/civ schema). **Consolidated there rather than duplicated here**, because a live
    decision buried inside a `✅ DONE` block reads as closed — and because two copies of a
    decision are two things to drift. *This pointer is deliberately reciprocal: that entry names
    this one. Written this way on purpose — the session that wrote it had just measured that **8 of
    15** correction→file edges in this corpus exist only at one end.*
  - **Rides as built:** `Block::Wood` stays with no worldgen emitter (removing the variant would
    renumber block ordinals and move every golden — destroying the attribution this slice was able
    to make); five retired draw salts leave a **deliberate hole** at `0x5700_0005`…`0x5700_0009`
    with an in-code rule *take the next unused value, never fill a hole*; `stubs.md` #1 resolved
    by deletion, #10's blast radius restated as zero; **draw-domain part (a) is discharged** by
    removal rather than conversion.
  - **Accept-by-outcome, not by gate:** the 12 chunks that carried every post go **102
    `Block::Wood` → 0**, and a 200-chunk box around all four former clusters reads **0**.
  - **Residency, absolute:** the whole recorded settlement history — 153 facts, 13 sites, 2
    polities, 90 collapses — cost **5,520 bytes of 377,364,589** (0.0015 %), and the only
    production code that ever read it was the function measuring its size.
  - **New Observed item, not caused by this slice:** `approx_resident_bytes` is **non-monotone in
    extent** (Medium **377 MB** > Large **213 MB**) because the deep grid is width-capped
    (`cell_m = max(extent_m/DEEP_MAX_WIDTH, DEEP_CELL_M)`) and record size is deposition-dependent.
    A reader of `s7_measurements`' table would find that puzzling and has no note to reach.
  - *Original entry, preserved:*

- **🔴 REMOVE THE BOOTSTRAP HISTORY CONTENT — polities, sites, ruins, the history pass**
  (**DECIDED 2026-07-26, user**). *"They are unratified zealous fabrications from the early
  bootstrapping of the project and I DO NOT care about them, they WILL be wholesale
  replaced, they should just be removed. We do NOT have any form of evo/socia/civ modeling
  even at the design stage: they are NOTHING."*
  - **WHY IT IS A REMOVAL AND NOT A MIGRATION.** There is no design, no model, and no
    plugin-pack intent behind any of it. It is not built on the SDK pass shape and could not
    be — **we have never designed a mechanism for declaring structures/blueprints and
    spawning them in the world at all**. Keeping it means keeping goldens that protect
    content nobody voted for.
  - **THE CONSUMER GRAPH IS ALREADY TRACED** (journal/0118's rider, corrections #64): the
    history pass is an unconditional `vanilla_passes()` member; ruins reach the screen via
    `collapse.rs::ruin_posts` → `Block::Wood` in `generate_chunk`; and
    `Pregen.{ledger, overlay, n_polities, observe_count}` have **exactly one non-test reader
    in the workspace** — `approx_resident_bytes`, which only measures their size.
  - **SCOPE:** `pregen/history.rs`, `Pregen.sites` and the four fields above,
    `collapse.rs::ruin_posts` + its `Block::Wood` emission, dc-sim's region/agent-step draws,
    and the pass's `vanilla_passes()` membership. **The goldens will move and that is
    correct** — the scratch-pad rule applies exactly (*byte-identity is a regression
    detector, not a specification*).
  - **IT ALSO DISCHARGES draw-domain part (a)**, which was sequenced as a user-owned
    appearance slice solely to protect this content. With the content gone the residual
    `engine.rs` draw has nothing to re-roll.
  - **⚠ EXISTENCE IS NOT STANDING** (CLAUDE.md § Conventions, added the same day): the
    integrator proposed *counting* the ruins before the user's direction landed, which
    already concedes that some number would matter. It would not.


- **✅ THE HILLSLOPE OPERATOR IS FIXED — SHIPPED 2026-07-29 (journal/0122).** The blocker
  below is discharged; it is kept live only because **one inference in it is falsified and one
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
  - **⚠ CORRECTIONS #72 — the (b) block below is WRONG and #63 (ii) with it.** It *was* a
    stability limit. journal/0116's 4× refinement took the calibrated arm from 100.8× past the
    bound to **25.2× past it**, so its null was a statement about the number 4. Every
    measurement in that entry stands; one inference does not. **Do not cite "not a time-step
    limit" from below without reading #72.**
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

- **~~🔴🔴🔴~~ ✅ FIXED — THE HILLSLOPE CONVEYOR CHECKERBOARDED THE REGOLITH ABOVE 1× — stubs #29,
  RE-SCOPED 2026-07-26 by the walk (journal/0115, corrections #61/#62) and then
  **DIAGNOSED 2026-07-26 by journal/0116 (corrections #63)**, and it BLOCKS the calibration
  below.** *This entry has been renamed twice. It read "THE INCISION CLAMP THAT WAS GREEN
  BECAUSE NOTHING ERODED" (sized at 148 pits — both wrong), then "THE EROSIONAL SOLVE GOES
  GRID-UNSTABLE ABOVE 1×" (right about the symptom, wrong about the mechanism: the
  discriminators say it is **not** a stability limit). The name now says where it is. The
  superseded framing is kept at the end because its mechanism is probably still real, just
  not dominant.*
  - **WHAT IS MEASURED (production-Medium, seed 1337, calibrated vs shipped as control).**
    Deep-cell concavity — `mean(8 neighbours) − self`:

    | | mean | p10 | p50 | p90 | p99 | >1 m | >20 m |
    |---|---|---|---|---|---|---|---|
    | shipped | −0.14 m | −0.3 | −0.1 | +0.1 | +0.3 | **0.0 %** | **0.0 %** |
    | calibrated | +0.73 m | **−50.8** | −0.1 | **+52.8** | **+106.3** | **35.8 %** | **23.2 %** |

    Closed hollows (`filled − routed`, the non-saturating census): **0 → 1,377 (3.1 % of
    land), deepest 112.8 m, 5.4 km³ of fill.** Regionally **7.8 % within 10 km** of the walk
    station — they cluster 2.5×.
  - **THE HEADLINE, and it is one sentence: relief grew 4.6 %, cell-to-cell roughness grew
    ~170×.** The shipped world's entire concavity distribution fits in ±0.3 m; the calibrated
    world's decile spread is ±50 m with the **median unchanged**. The landscape's *shape* is
    intact and the *grid* has become noise. Symmetric tails + untouched median = adjacent
    cells oscillating against each other.
  - **THE PITS ARE THE TAIL, NOT THE DEFECT.** A closed hollow is where the oscillation
    happened to bottom out with no outlet. Fixing the clamp would clamp the tail and leave
    23 % of cells 20 m off their neighbours — a slice that goes green and does not fix the
    world. **Do not brief the clamp fix as the blocker.**
  - **~~HYPOTHESIS~~ — THE DISCRIMINATORS RAN 2026-07-26 (journal/0116). THE STABILITY-LIMIT
    STORY IS FALSIFIED; THE DEFECT IS STRUCTURAL AND IT IS IN THE REGOLITH.** The standing
    hypothesis was *an explicit scheme past its stability limit*; it was flagged unmeasured,
    it was measured, and it is wrong.
    - **(a) It IS a checkerboard.** Concavity lag-1 autocorrelation **+0.377 / +0.267
      (shipped)** vs **−0.867 / −0.912 (calibrated)**, lag 2 back at +0.56 / +0.71,
      first-difference ACF −0.86. The three reference values are derivable in closed form —
      white noise **−1/6**, perfect checkerboard **−1** — so the discriminator is not "is it
      negative" but *how far past −1/6*. Both axes: a true 2-D Nyquist mode.
    - **(b) It is NOT a time-step limit.** Refined **4×** at fixed total simulated time
      (`k×` epochs, `1/k×` every per-epoch rate), concavity rms goes **40.46 → 45.29 →
      38.76** — 4 % under a 4× refinement, non-monotone — while the landscape holds (relief
      +3.9 %, mean surface −0.3 %) and the shipped control reproduces to three digits. The
      checkerboard gets **purer**: ACF(1) −0.867 → −0.909 → **−0.947**. Closed hollows *do*
      converge (1,377 → 955 → 280), so the **pits** are partly a step artefact and the
      **oscillation is not**. *`myr_per_epoch` does not exist as a knob; the register is
      `iterations` against per-epoch rates. The claimed shared register with stubs #27's
      heir (b) is **withdrawn** — they share the limiter, not the clock.*
    - **(c) ISOSTASY IS THE DAMPER, NOT THE DRIVER** (mechanism proposed mid-flight, killed).
      `iso_rate` 0.50 → 0.25 → 0.00 takes concavity rms **40.46 → 62.03 → 90.34** and hollows
      **1,377 → 2,150 → 13,012**. On the **shipped** world `iso_rate = 0` takes rms 0.22 →
      19.71 and hollows **0 → 6,215**. **Do not touch `iso_rate`** — it is the only
      grid-scale low-pass in the solve.
  - **WHERE IT LIVES — split `surf = r + h` and this is what the fix slice is briefed
    against.** Same Laplacian over each summand, plus the creep limiter's binding fraction:

    | | limiter bound | conc(**r**) rms · ACF(1) | conc(**h**) rms · ACF(1) | surf rms | mean h |
    |---|---|---|---|---|---|
    | shipped k=1 | 88.7 % | 3.42 m · −0.10 | 3.43 m · −0.10 | **0.22 m** | 4.58 m |
    | calibrated k=1 | **96.0 %** | 23.77 m · −0.55 | 61.95 m · −0.82 | 40.46 m | 41.41 m |
    | calibrated k=2 | **94.9 %** | 12.59 m · −0.34 | 55.21 m · −0.88 | 45.29 m | 36.59 m |
    | calibrated k=4 | **94.7 %** | **5.88 m · −0.11** | **42.43 m · −0.93** | 38.76 m | 24.45 m |

    - **The BEDROCK solve converges** (23.77 → 5.88 m, ~`1/k`, ACF back to −0.11): there is a
      real time-step artefact in this world, it is in `r`, and D2 converged it away.
    - **The REGOLITH does not.** `conc(h)` falls 32 % while its ACF sharpens to **−0.93**, and
      the refinement does not hold the cover fixed (mean `h` 41.4 → 24.5 m), so normalised by
      what the operator moves the roughness **grows**: `conc(h)/h̄` 1.50 → 1.51 → **1.74**.
      **The flat surface total was two defects cancelling.**
    - **The limiter is deaf to the step: 96.0 → 94.9 → 94.7 %.** Measured, not inferred. It
      caps export at *the cover the cell has*, so the transfer is a function of **inventory,
      not `rate × dt`** — which is why refining `dt` did nothing.
    - **Saturation alone is NOT sufficient — do not brief it as if it were.** The limiter
      binds on **88.7 %** of *shipped* cells and their `conc(h)` ACF is −0.10. What the
      calibration adds is **cover** (4.58 → 41.41 m mean regolith).
    - `corr(concavity, h − h̄) = −0.831` calibrated (+0.147 shipped); `rms(h − h̄)` 70.1 vs
      4.0 m. On the shipped world `r` and `h` roughness **anti-correlate almost exactly** —
      3.42 + 3.43 m of component concavity summing to 0.22 m. **That compensation is what
      broke.**
    - **Register: the flux limiter / donor-cell partition in `erosion.rs::diffuse`.**
      **Hypothesis for the slice to test first, explicitly not measured:** a donor-cell scheme
      that moves everything downslope has a period-2 mode by construction (A gives all its
      cover to B; B is now higher and gives it back), damped only by isostasy downstream.
  - **Until this lands the engine cannot run erosion at ANY realistic rate.** Unchanged, and
    now for a better-understood reason. It gates journal/0114's flag flip and every future
    calibration.
  - **⚠ THE ACCEPTANCE CRITERION THAT MISSED IT (corrections #61).** journal/0114's binding
    criterion was *"relief within 5 %"*. Relief is `max − min` — a **global extremal**
    statistic that is mathematically incapable of seeing spatial arrangement; you can shuffle
    every interior cell and leave it unchanged. **Any future erosional slice pairs its
    aggregate criterion with a neighbour-relative one** (Laplacian, gradient distribution,
    autocorrelation), or it is measuring the axis that did not break.
  - **⚠ AND THE GUARD CANNOT FAIL INFORMATIVELY (corrections #62).**
    `mfd_routing::no_interior_cell_is_cut_below_all_of_its_neighbours` counts cells below
    **all eight** neighbours — a winner-take-all predicate that **saturates**: as the defect
    generalises, neighbours sink too and stop qualifying each other, so the count falls back
    toward zero exactly when the damage becomes universal. Re-assert it on **fill depth and
    concavity**. *Found by the user flying the terrain, after two probes and a gated assertion
    all agreed with each other and were all wrong the same way.*
  - *(SUPERSEDED FRAMING, kept because it is probably a real contributing mechanism: the
    never-incise-below-the-lowest-receiver clamp is applied at incision, and weathering,
    creep, wave and eolian all run **after** it in the same epoch and can lower a cell past
    its floor. That predicts **isolated deep holes**, which is a subset of what the world
    shows. Its A-2 variant in spines — a test's unstated premise, "erosion is fast enough for
    this to mean anything" — stands on its own merits.)*

- **🔴 RE-PICK `EROSION_CALIBRATION` AGAINST THE FIXED OPERATOR — and flip the flag**
  (sequenced 2026-07-29, user: *"yes"*; journal/0122). **The constant was fitted to a broken
  solve and does not survive its repair.**
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

- **🔴 THE TRANSPORT OPERATOR HAS A CEILING — stubs #27** (journal/0114). Six full 200-epoch
  worlds measured (1×, 10×, 45×, 100×, 300×, 1000×): **export is proportional to mean regolith
  thickness**, and **creep's flux limiter already binds on ~89 % of cells that have regolith to
  move, at the SHIPPED rates.** The pass is a **one-cell-per-epoch conveyor**, so 100× on
  transport alone buys **1.6×**, and reaching the craton band costs order **100 m of cover**.
  - **This revises journal/0111's diagnosis.** *"A calibration, not an architecture"* was **half
    right**: the constants are wrong **and** the transport operator is capped. Named heirs:
    rivers that actually carry, or a non-capped creep operator.
  - The uniform-scaling hypothesis was **falsified by a diagnostic the agent added because the
    hypothesis needed a falsifier** — not by argument.

- **🔴🔴 CALIBRATE THE DEEP-TIME CLOCK — the largest measured defect on the board**
  **(⚠ PARTLY BUILT 2026-07-26, journal/0114 — see `ROADMAP-history.md` § Shipped. It is BUILT, MEASURED and
  DELIBERATELY OFF, blocked on stubs #29 above. The band is NOT reachable at any multiplier;
  read stubs #27 before assuming a bigger number fixes it.)**
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

- **🔴 COARSEFIELD ADOPTION — the ratified cure that lost its entry** (revived 2026-07-29 by
  user ruling: *"i think we revive it too"*). **This is the fix for U22 (the cake law) and U3
  (the per-chunk palette checkerboard), and it has had no owner since 2026-07-24.**
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
  - **BUILT, AND NOTHING CALLS IT — the whole API.** Verified workspace-wide at `ae4bb29`:
    `sample_dithered`, `sample`, `summarize`, `DitherSource` (trait, **zero production impls**)
    and `CoarseField<T>` itself all have **zero callers outside `dc-core`**. Only `ShareVec<N>`
    landed. `collapse.rs:949` still defers to *"the `CoarseField<T>` extraction (audit Part 2)"*
    as though it were pending — **the extraction is done; the adoption is not.**
  - **⚠ AND IT IS INVISIBLE TO ALL THREE LOOSE-END LOCI.** It is in **neither** `spines.md` § 3
    "Built, and nothing calls it" (12 rows, no CoarseField row) **nor** `stubs.md` **nor** — until
    now — ROADMAP. `spines.md` § S-4 calls it the *"Ratified end-state — **EXTRACTED**"*, which
    reads as **done**. Per read-first item 6 an unlisted loose end is *the defect, not a licence*:
    the largest built-and-unconsumed mechanism in this thread was hidden from the lookup that
    exists to find exactly this. **Add the § 3 row in the same commit as the first adoption slice.**
  - **THE TWO SITES ARE ONE ROOT, AND A FIX TO ONE DOES NOT FIX THE OTHER.** The root is
    `DeepField::record_at_voxel` reading **NEAREST** at the ~460 m deep cell — its own doc comment
    flags it (*"a variable-length unit sequence cannot be interpolated, so the facies story steps
    at the ~460 m deep-cell grid — FLAGGED sampling choice"*, `field.rs:758-772`). Two expressions:
    **U22 / far field** at `collapse.rs::surface_class` (the class draw's *shares* are still
    nearest-per-cell, so minority phases die at the 460 m line), and **U3 / near field** at
    `collapse.rs:1409` (one point-sample per chunk shared across all 1024 columns). The 2026-07-24
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
    are content**. *The mechanism is not designed; the direction is set.*
  - **FIRST SLICE — a DESIGN PASS, not code** (user: *"refinement primitives want a design
    pass and then a decomp and port of existing collapse"*). Field-notebook-first per the
    earth-processes method. It must answer: what is the primitive set · what the operator
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
  - **THE MECHANISM (build it): a hook on file write that reminds, with an instruction to
    restructure** — pull separate concerns into separate files. **Do not answer "the tool
    cannot see X" with a rule asking people to remember X** — that was tried for probes and
    **failed in one day** (CLAUDE.md § Gates). A reminder the harness issues is a mechanism; a
    line in a doc is not.
  - **ADOPTION (user's terms): immediate for NEW files; gradual refactor of old work WHEN IT IS
    TOUCHED.** No big-bang rewrite.
  - **✅ CONVENTIONS DECIDED 2026-07-28 (user)** — was *"to be set, not guessed."* Set from the
    corpus measurement rather than taste, and **shipped in `scripts/filesize_hook.py`**, whose
    module docstring is now the record (the hook is the only mechanically-enforced corpus
    control, so the convention lives where it is enforced).
    - **THE SPLIT AXIS IS LIVENESS, NEVER TOPIC.** Every split moves out the **cold half** —
      still true, still cited, no longer read to do today's work. **Both real conversions
      already did this and neither was by topic:** `ROADMAP → ROADMAP-history` split by
      **status**, `notebook → evidence` split by **read pattern**. The convention is the axis
      those two taught, not a new invention — which is what `stubs.md:22` requires.
    - **WHY TOPIC-SPLITTING IS DISALLOWED, and it is the non-obvious half.** Contradiction here
      is produced by **addition** (design docs delete 2–4 % of what they add), and every
      expensive failure was a claim sitting near its own refutation — two sentences apart
      (#65), forty lines (#58), two subsections (#53), 400 lines (journal/0119). **Topic-
      splitting a live doc converts an in-file contradiction into a cross-file one**, reachable
      only by the `doc-topology` sweep — five of whose top eight findings were unsuspected by
      construction. **That trades VOLUME (third in value) for TOPOLOGY (the one that cost an
      architecture).** *Stated at honest strength: co-location did not prevent those
      contradictions — access was never the problem. The claim is the weaker, sufficient one:
      topic-splitting costs the one condition under which a reader could notice and buys only
      line count. Liveness-splitting cannot do this, because the cold half has stopped
      accreting.*
    - **THE THRESHOLD APPLIES TO THE HOT FILE ONLY** — archives and evidence files are exempt
      by designation. *The old hook flagged `ROADMAP-history.md` for being exactly what it was
      built to be: crying wolf on a file doing its job, with no correct action available.*
    - **THREE CLASSES OF `.md`, by READ PATTERN** — **NARRATIVE** (journals, audits, spikes:
      written once, read whole, never revised) **exempt**, and splitting one is *harmful*
      (measured: 120 entries, median **177** lines, max 536) · **REGISTRY** (`ROADMAP`,
      `corrections`, `spines`, `stubs`: looked up by ordinal, not read) **2,500**, split =
      **archive resolved entries** · **ARGUMENT** (`docs/design/*`, skills: read in sections,
      actively revised) **1,000** — the only class where the threshold bites and the only class
      where topology failures happen.
    - **Verified on the real corpus:** flags `ROADMAP` (5,120) and `corrections` (2,681) as
      registries with a real action, and `material-behavior` (1,036) as an argument doc; silent
      on `ROADMAP-history`, the evidence file, every journal entry, every spike, and `spines`
      (1,283, under the registry bar). **Signal went from "everything large" to three files with
      a correct move each.**
    - **DELIBERATELY NOT BUILT:** no taxonomy registry, no frontmatter marking file class, no
      validator. **Two conversions is below this project's own bar** (`stubs.md:22`;
      session-workflow § Seam-first #6). Class is derived from path, which suffices until the
      next two or three splits teach more.
    - **⚠ HONEST LIMIT, recorded so this is not oversold:** volume is the **third** most
      valuable of the three docs-ops failures and the archive *"would not have prevented
      journal/0119."* **This buys agent context efficiency; it is not a correctness fix.** The
      open **corpus-addressability** thread may subsume part of it — *a file addressable by
      section may not need to be small* — so the ARGUMENT threshold is the negotiable number if
      that lands.

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
    (corrections #64, a read-only trace taken
    2026-07-26 during (b); **not re-scoped by that agent — the integrator's and the user's call**):
    - **The region-step draw (`engine.rs:331`) is the real one, and it IS visible.** Collapsed
      pressure → the sack roll (`pregen/history.rs:221`) → `abandoned` → `Pregen.sites` →
      `collapse.rs:1588`'s ruin posts → `Block::Wood` in `generate_chunk` (`collapse.rs:483-488`)
      → meshed (`dc-client/src/meshing.rs:214`). **No flag anywhere** — the history pass is an
      unconditional `vanilla_passes()` member (`pipeline.rs:359-366`).
    - **The agent-step draw (`engine.rs:355`) re-rolls NOTHING.** The pregen overlay is built with
      an empty agent roster (`history.rs:82`/`:84-88` pass `vec![]` as `with_graph`'s `agent_home`),
      so the loop never executes outside dc-sim's own tests. **That half is byte-identical
      housekeeping and could ride with anything.**
    - **"Polities" is not re-rolled.** The count is fixed at epoch 0 (`history.rs:134-149`);
      `PolityExtent` facts move but live only in `Pregen.ledger`, which **nothing in production
      reads** (`pregen/mod.rs:300-304`; sole non-test reader `approx_resident_bytes`,
      `mod.rs:367`) — a spines § 3 "built, and nothing calls it" cluster.
    - **Goldens that would move:** `contents_contract.rs:70-86`, `s7_walk.rs:32-40`,
      `geology.rs:32` (all hash `generate_chunk` blocks, structurally downstream of the posts),
      and `s7_handoff.rs:118`, which pins a **seed-specific sack** and is the likeliest break.
      `GOLDEN_SURFACE` / `GOLDEN_RECORD` are **not** downstream (deep-time field only).
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

*(The `production_* → golden_*` rename that stood here — opened 2026-07-25 by journal/0106,
given a real entry by the staleness sweep row D-2 — **shipped 2026-07-26**; see `ROADMAP-history.md` § Shipped.)*

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
  - ~~**🔴 OWED BY MFD, and it is the nearest-term thing on this arc (2026-07-26, journal/0109):**~~
    **✅ SHIPPED 2026-07-26 — struck 2026-07-29 (baseline sweep S5/F4). Both terms are stale.**
    **Hybrid `p` shipped** as FLOW (b′) (`journal/0113`, `ROADMAP-history.md`; peak catchment
    84 → 298) and is **the shipped default** — `crates/dc-worldgen/src/deeptime/grid.rs` sets
    `mfd_exponent: 1.0` / `mfd_exponent_channel: 16.0` with `mfd_chi_lo`/`mfd_chi_hi`,
    doc-commented *"the hybrid-`p` law (journal/0113)"*. **The `k_bedrock`/`k_transport`
    recalibration** is at least partly discharged by the joint calibration
    (`journal/0114`, `EROSION_CALIBRATION = 45` — which this same entry acknowledges elsewhere),
    though that ships **behind `calibrated_rates`, OFF**. *This document cites hybrid `p` as a
    **completed predecessor** two hundred lines above while flagging it 🔴 OWED here.*
    Original text, kept as the record of what was owed:
    ~~**hybrid `p`**, plus **recalibrating `k_bedrock`/`k_transport`**.~~ Uniform `p` **does not
    concentrate flow** — peak catchment fell **1,245 → 84 cells** — because MFD lowers both `Q`
    and `S` at every cell, making it **systematically less erosive than D8 at fixed coefficients**
    (total load −16 % while load-carrying faces ×2.76). The literature's answer is `p` as a
    function of area/slope, or single-receiver above a channel threshold. **This gates whether the
    world ever SHOWS what the record now holds** — today the record carries 7.5 M simultaneous
    divergences and the viewport carries 2.7 cm. *Uniform `p` was the simplest correct thing and
    is the wrong long-run shape; that is the shipping agent's own verdict, not a later critique.*
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

- **APPEARANCE WALKS OWED** (tracking, user: "we do that when able" — journal screenshots for
  appearance-changing work). **(1)** ✅ **DONE — walk-confirmed 2026-07-24** (user, aerial
  fullbright: *"I can now confirm the LOD is fixed!!"* — fine near field transitions cleanly into
  the warm coarse far LOD, no cold-dither ring; screenshots `journal/assets/0091-lod-walk-*`).
  The only residue is the poke-through geometry check on the lit pass — low priority. Still owed:
  **(2)** ✅ **DONE — walk-confirmed & ACCEPTED 2026-07-25** (user; journal/0097). Flag-ON vs the
  byte-identical flag-OFF control on the same column: **7 voxels ≈ 6.3 m** of loose product at the
  basement contact (predicted 6.09 m), **24.2 % of land banded**, veneer and basement unmoved.
  Instrument `--fullbright` (a material question). **Accepted with a follow-up, not a blocker** —
  the band has a **hard perimeter** (see Sequenced "the weathering front needs a PROFILE"), and the
  gradational-looking top contact is **boundary quantization, not weathering** (one voxel deep,
  `mixed_voxel_contents`); **(4)** ✅ **DONE — WALK-CONFIRMED & PASSED 2026-07-25** (user, at the
  station, `--weather-inventory --fullbright`, bench cut; assets
  `0099-weathering-front-profile-bench.png`, `-full-section.png`): *"Success on the gradation!
  Aesthetically, which is all I can judge here, this is a pass. **Our world just got far deeper and
  more interesting to look at, just with this. The spawn area isn't a shallow pile of rubble over a
  harsh boundary of uniform rock anymore.**"* Measured on the record **before** the screenshot so
  the picture could not flatter it: product **6→5→4→3→1** eighths downward, parent structure
  **4→5→7**, form flipping at 295/294 from debris to `structure`+`pore_fill`, deepest front voxel
  **7/8 parent + 1/8 product**. The journal/0097 hard perimeter is gone at **both** faces.
  *(Live bonus: `has_contents:false` on the basement below — journal/0101's `identify(pos)` fix
  working in the field, where the query used to claim `dc:air` over solid stone.)*
  ~~**(4)** the **weathering-front PROFILE** flag-ON walk~~ (journal/0099,
  shipped 2026-07-25) — the band is now a graded **19-voxel** front, ~**2.67×** deeper than the
  old slab, with **retained parent structure at the bottom contact** (7/8 parent + 1/8 product)
  instead of a hard perimeter; station world **(84185 m, 9212 m)**, voxels **y=299…281** — cut a
  **bench** (not a pit), `--fullbright`, read with `world_get_contents`; **(3)** ⚠️ **the
  geotherm's coal-distribution shift — THE WALK IS A NULL AND NEEDS NO GAME TIME** (tour-mapped
  2026-07-25, `examples/coal_walk_tour.rs`; corrections #51): **the shipped world has ZERO coal**,
  so there is nothing to look at. The appearance question the user was going to be asked ("is this
  seam thick enough?") is **replaced by a content question** — *"is a coal-free world acceptable
  for now?"* — answerable at the desk, not in-game. **Do not spend a walk on it.** See the
  🔴 Observed entry. *(A fallback peat station exists if the world is ever walked for organics
  anyway: world −8266, −45533, surface 263.7 m, 2.7 m of peat outcropping at the surface — no
  bench needed; nearest-to-Station-A alternative at 58867, −34963, 50.9 km away.)*
  **(5)** ~~the `pore_rider_share` **correlation** walk — a fullbright walk along a strong front
  looking for banding correlated with the parent's eighth~~ **NEVER OWED — ANSWERED AT THE DESK
  2026-07-25 (journal/0105).** Filed here 2026-07-25 by sweep row D-5, which correctly caught that
  a proposed walk was living in an Observed entry and not in this tracker — but the walk had
  already been retired by the hash-domain slice that merged the same day. It measured the question
  spatially instead: on one 32×32 contact plane sharing one record and one fill plan, **every
  autocorrelation at lags 1–4 in both axes is inside ±0.07 of zero, before *and* after** the fix.
  Structurally absent, not merely subtle — both offsets are functions of a position hash, so a
  dependency between two decisions **at one voxel** cannot make structure **between** voxels.
  *Kept struck rather than deleted: the lesson is the tracker's, not the walk's — a walk proposed
  in an Observed entry and not listed here is a loose end by stubs.md doctrine, whichever way it
  later resolves.*
  Screenshots to `journal/assets/` named for their entry.

- ✅ **DONE 2026-07-25 — shipped, see `ROADMAP-history.md` § Shipped (journal/0100).** Measured result: flag-ON
  `DeepField` **311.02 → 179.12 MiB**, the flag's own cost **+161.81 → +29.91 MiB (5.41×)**,
  ledger heap **155.01 → 16.31 MiB (9.5×)**, index 3.4 % of the ledger, world byte-identical
  (same 1,033,189 facts in the same 72,006 slots). *Entry kept below as shaped, for the record.*
  **`FactLedger` IS 89 % EMPTY HEADERS — give it the CSR layout `flux.rs` already proves**
  (shaped 2026-07-25 at the user's direction; measured in `docs/spikes/S19-flow-record-cost-results.md`).
  - **WHAT.** `FactLedger` is `Vec<Vec<Fact>>` keyed per (cell, slot). Measured on a production
    world: **5,832,862 inner `Vec`s of which 5,760,856 (98.8 %) are EMPTY**; **89 % of its
    ~150 MiB heap is empty `Vec` headers**, against a real payload of **16.6 MiB over 1.03 M
    facts**. Turning `weather_inventory` ON therefore costs **+156.91 MiB** — and after the
    `shrink_to_fit` win that is **~1.4× the entire rest of the `DeepField`** (108.55 MiB bare).
  - **WHY NOW.** The walk **blessed the band** (journal/0097), so the flag is on its way to
    becoming a default rather than a dev toggle — and the moment it is, this is the single
    largest residency item in the world. Runtime residency is first-class (CLAUDE.md); gen time
    is free, so the conversion cost is free.
  - **THE FIX IS ALREADY PROVEN IN-TREE — do not design a new one (A-4).** `deeptime/flux.rs`
    (journal/0096) stores a far larger sparse per-(cell,chapter,face) record as **flat
    exact-sized arrays + a CSR index**, and measured the index floor at **0.056× of total** —
    i.e. *the index is free and the payload is the whole constraint*. Port that layout. Facts are
    also **causally triangular** (a slot deposited in chapter `c` cannot carry a fact from before
    `c` — 57.7 % of the naive rectangle, a free 1.73×), so never allocate the rectangle.
    **⚠ CORRECTED BY THE BUILD (journal/0100):** the triangular exploit is **SUBSUMED, not
    applied** — exact-sizing stores the **1,033,189 facts that actually exist**, which is **4 % of
    even the causal ceiling**. Triangularity is a **budgeting tool for projecting an UNBUILT
    record, never a sizing rule for a BUILT one**: once you can count the real entries, any
    formula over the possible ones is a ceiling you have already beaten. Keep that distinction
    when using S19's projections for the flow record's later slices.
  - **SCOPE.** `deeptime/inventory.rs` + its readers. **Pure layout change: byte-identical
    world, identical facts, identical `weathering_product_m`** — the goldens and every
    fact-count test must pass **unmoved and by name**. Acceptance = the measured before/after
    residency with `weather_inventory` ON, plus byte-identity proven by test name.
  - **BLOCKED-ON:** `inventory.rs` sits inside `deeptime/`, which the in-flight **head field**
    slice owns. Launch when that lands, or carve `inventory.rs` out of its write-set explicitly.
  - **NOTE THE SHAPE, not just the number:** this is the same defect the flow record was warned
    off in-flight and avoided. Fixing it here closes the loop — the measurement that protected
    the new record should also repair the old one.

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

- **✅ SHIPPED 2026-07-25 (journal/0099) — see `ROADMAP-history.md` § Shipped, journal/0099, for the result.** *(Entry kept for its
  reasoning; the flag-ON walk it earned is now item (4) of APPEARANCE WALKS OWED.)*
  ~~THE WEATHERING FRONT NEEDS A PROFILE, NOT A SLAB~~ (walk finding, user, 2026-07-25;
  journal/0097). **WHAT.** Movement 3's band is correctly *magnituded* and wrongly *shaped*: the
  collapse folds the scalar `FactLedger::weathering_product_m` into **one stratum of one class**
  (`CLASS_CLASTIC_FINE`, `geology.rs::emplace_weathering_front`), so the record→voxel path expresses
  a span that wholly contains a voxel as `Single` — **8/8 of one member**. Result: pure product
  above, **pristine contents-free basement below, a hard perimeter on both faces.** User: *"the
  layer of degraded bedrock has a hard perimeter and then pure bedrock, which does not make sense
  for the natural process it claims to model… nature does not in-place degrade a bulk unit of rock
  to another via weathering."*
  - **WHY IT HAPPENS.** The inventory edge is honest at its own tier — `(GRANITE, Structure) →
    (GRANITE, Loose)`, a **form** change on one material, mass-conserving, one fact per agent. The
    loss is at the **fold**: `weathering_product_m` is a **scalar, and a scalar cannot carry a
    profile.** What the model computed is a rate integrated over depth and time; what got emplaced
    is a slab. The downward gradient — intact rock → corestones → grus → clay — *is* what makes
    saprolite legible as saprolite, and it is exactly what the fold discards.
  - **HEIR SHAPE (user-proposed, expressible in TODAY's vocabulary — which is what makes this a
    follow-up and not a research project).** `structure → pore_fill` rather than
    `structure → structure`: **retained parent structure with weathering product in its pores**, the
    structure share falling with height through the front. `VoxelContents` already carries
    `structure[]` / `pore_fill[]` / `open_pores` / `debris[]` in eighths — the walk read them
    straight off `world_get_contents`. Today's band says `structure: []`, `debris: [mudstone 8/8]`:
    **the parent rock is simply gone.**
  - **COUPLES TO** stub #16 (a rind's *material identity* — the inventory says granite-loose while
    the collapse expresses mudstone; the two disagree today and #16 owns that half) and to the
    deep-cell inventory's **form vocabulary** (§2 forms / §3 transition graph — this is a
    form-transition question, so it belongs to the same machine).
  - **NOTE THE FLATTERING ARTIFACT (journal/0097, worth not re-deriving):** the band's *top* contact
    already mixes and reads convincingly — but that is **boundary quantization**
    (`mixed_voxel_contents` / `allocate_partial`, journal/0055), **exactly one voxel deep, wherever
    any two units meet**, and would look identical at the contact of two units that never
    interacted. It is not a weathering gradient and must not be mistaken for progress on this item.

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
  - **~~TIER BOUNDARIES DERIVE FROM THE LOD LADDER~~ — SUPERSEDED SAME DAY by the UNTIERED
    decision above. Preserved because the reasoning is still load-bearing for the
    *render-side* query, if one is ever wanted.** ~~DECIDED 2026-07-25 (user):~~ *"the
    boundaries should fall out of LOD bands, which already reduce material contents
    depth / honesty."* The tiers are **not a second, independently-tuned threshold set**
    — that would be A-4 (a mechanism beside the one we have) and would drift out of sync
    with what is actually on screen. `LodLadder` (`dc-client/src/farmesh.rs:126`,
    journal/0091 — one ladder, all named knobs, every ring edge already **derived** from
    it) is the authority; `identify`'s Near/Mid/Far read *it*. Payoff: the ladder is
    already structured to become **in-game per-player perf settings**, so honesty
    automatically tracks the player's own quality setting — turn the view distance down
    and the answers get *honestly* coarser, with no second knob to forget.
  - **~~⚠ OPEN, and it must be settled before dispatch — WHOSE ladder?~~ DISSOLVED
    2026-07-25 with the tiers themselves** (there was no good answer because the
    question was malformed — see the UNTIERED block above). Preserved only for the
    *render-side* query, if one is ever wanted.** `LodLadder` is
    **dc-client** state and is **per-viewer**, but `identify(pos)` is a **world** question
    reachable headlessly (dc-api agents, mods, tests) where there is no camera and no
    ladder. So the signature cannot simply read ambient client state. Candidate
    resolutions (not chosen): **(i)** `identify` takes an explicit *tier/observer* argument
    and the client passes the one its ladder implies — keeps the world query pure and makes
    the client the only place that knows about cameras; **(ii)** it defaults to
    **finest-resident** and the client narrows; **(iii)** the ladder (or a headless-safe
    projection of it) moves somewhere both crates can see. **(i) is the integrator's lean**
    — it preserves "what is at X is a world question, not a camera question", which this
    very entry already asserts. User call at dispatch time.
  - **~~THE FAR TIER'S PAYLOAD IS A MIXTURE, NOT A WINNER~~ — PRESERVED FOR THE RENDER-SIDE QUERY,
    like its two siblings above** (folded 2026-07-25, sweep row D-4). *This bullet is written in the
    tier language the **UNTIERED** decision retired the same day, and its surviving content is
    already stated up in that decision (**"payload is uniformly a mixture… there is no far tier left
    to special-case"**). What is **unique** to it and still live: the **render-side speckle
    direction**, and the cross-reference to the journal/0091 **LOD fix (b)** cold/warm material
    agreement, both in its last two sentences.* **DECIDED 2026-07-25 (user):** *"LOD may
    be textured by a **speckled mix** in the future, not just single material as it is now. So
    leave the seam for speckle — or better yet have it fall out by construction."* **It falls out
    by construction, and that is the design:** make the payload **uniformly a mixture at every
    tier**, so the tiers differ in **RESOLUTION** (how many components survive, at what precision),
    **never in KIND**. A single dominant material is then just *a one-component mixture at 8/8* —
    today's `classify` answer expressed in the general shape, with **no special case to migrate**
    the day the far field goes speckled. Writing `Far = one MaterialId` would bake exactly the
    "one arbitrary component" assumption this whole arc exists to retire (and would need an A-2
    correction the moment speckle lands). **Corollary:** the same shape carries the **UNRECORDED**
    answer below — an empty mixture is not the same value as a one-component `Air` mixture.
    **Ties into** the far-field speckle direction the genesis-passes / octave arc is heading for,
    and the LOD fix (b) cold/warm **material** agreement (S-9) still owed from journal/0091.
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
      `record_at_voxel(cx*32+16, cz*32+16)` (`collapse.rs:1409`). Octaves applied to the member
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

- **Collapse-cache `evict()` is unreachable from far-field-only sampling**
  (diagnosed 2026-07-21, journal/0050; `evict()` fires only from
  `generate_chunk`, so a `coarse_surface`/`column_record` sweep can grow
  `lattice_memo`/`locale_cache`/`region_cache` between chunk generations).
  Bounded in normal play (a storm generates chunks constantly) — not the RAM
  march, which was `HostWorld.chunks` and is now fixed. **Reassigned
  2026-07-21** to the forms/partials `collapse.rs` rewrite, which owns that
  file. *(**Separate hardening item — DeviceLost degrades loudly: SHIPPED**
  2026-07-21, journal/0054.)*

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
- *(**Far-field horizon knob: SHIPPED** 2026-07-21, journal/0042 — see `ROADMAP-history.md` § Shipped.
  `--horizon <km>`, default provably unchanged, measured to 10 km.)*
- **A 5th/6th far LOD level — for horizons past ~10 km** (found by the
  journal/0042 measurements, 2026-07-21). The 4-level ring scheme's honest
  ceiling is ~10 km / 172 MiB / 13.7 s to fill, because cost is quadratic
  inside the stretched L4. Journal/0023 already named the right answer — *add
  rings*, don't lengthen the last one. Only worth doing if the design target
  wants vistas past 10 km; note the haze limit below may bind first.
- **Wave-magnitude retune — STRUCK 2026-07-21 (user): no retune.** "That
  whole mechanism changes after water machinery. that would be a bandaid,
  against our standing rule against bandaids. can revisit later." The
  confirmed null (0.68 m, journal/0049 station 5) rides as-built until the
  fetch model (wave energy from S11 body size/shape/depth × the 0037 wind
  field) replaces the constant outright — wave expression is a consumer of
  the water design pass now, not a tuning slice.
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
- *(**Zonal circulation profile: SHIPPED** 2026-07-20, journal/0037 — see
  `ROADMAP-history.md` § Shipped.)*
- *(**Deep-config flag plumbing: SHIPPED** 2026-07-20, journal/0039 — see
  `ROADMAP-history.md` § Shipped. The four launch flags (`--tectonics`, `--full-agents`,
  `--amplitude`, `--extent`) boot a flagged world; the combined amplitude /
  tectonic / full_agents walk is unblocked. The override channel is
  `DeepOverrides` on top of `production_config`, NOT a `WorldParams` field —
  the ~30 `{ seed, extent }` call sites were left untouched.)*
- **Erosion-supply calibration** (from the S12 spike's new finding,
  2026-07-20): exhumation comes out metre-scale at shipped erosion rates,
  gating exhumed-core/foreland legibility independent of amplitude — the
  S9 calibration loose end, now load-bearing. Needs-measurement class:
  compare model denudation against real orogen rates, propose 2–3
  calibrations, render each — the user then chooses between pictures, not
  rate constants. Dispatch after the combined walk settles amplitude.
- **Tectonic uplift-plane redesign — DESIGN PASS (superseded — done)** (direction ratified
  2026-07-20, earth-processes.md § 1 DECIDED entry): tectonic history
  (uplift(t), plate advection, chaptered boundary re-classification) +
  analytic boundary forcing (uplift from exact bisector distance at deep-grid
  resolution). Spike-class; upstream of dip/fold, volcanism, and the
  amplitude call. Includes the plate-count/scale-compression knob decision
  (user-owned).

- **Retire `client_player_pose_set` into `dc:character/pose`** (API.md
  Decisions #5, ratified 2026-07-20): make the player a dc-api character
  and route the player controller through controller-verb commands — the
  work that makes player input replayable (decision 2). Explicitly
  sequenced AFTER the session-4 console and edges agents merge: it rewrites
  `mcp.rs` and `player.rs`, both in those agents' write-sets.

*(**FF2a — voxel-language far field: SHIPPED** 2026-07-19, journal/0023 — see
`ROADMAP-history.md` § Shipped. Stepped columns retired the smooth TIN; step 0 empirically confirmed
Bevy 0.19's GPU-driven multidraw engages for our custom material
(`mode=Culling`, 74 draws → 1 multidraw set); the buried-sheet and tile-crack
field reports are resolved, below.)* **FF2b — coarse volumetric
summaries** paired with the caves/underground thread of the water design
pass (when overhangs exist, the summary goes 3D; couples to S3 region
storage). FF2a left the extension point ready: the per-column payload is a
`ColumnSpan` the mesher already treats as one of a potential stack, and the
tile mesher is a pure function of plain span data (async-meshing / persistent
edit-tracked LOD store stay drop-in).


*(**Erodibility coupling — lithology-aware erosion: SHIPPED** 2026-07-20,
journal/0029 — see `ROADMAP-history.md` § Shipped. Cause 1 of the dismal mountains is closed:
erosion is lithology-aware, off by default, byte-identical when off,
agent-specific resistance so karst/glacial/littoral stay implementable.
What remains is **user decisions**.)*

**NEEDS RATIFICATION (user-owned — this CHANGES TERRAIN SHAPE):**
1. ~~**Flipping `production_config`'s `erodibility` to true**~~ **RATIFIED AND
   DONE 2026-07-20** (user: "flip it, i want to see"; journal/0030). It is on;
   every new world has a different shape. The appearance answer came back
   **negative** — the ground-level surface is measurably re-terraced, but the
   predicted ledges/benches at hard beds and the resistant basement core are real
   in the data and **absent on screen** at the shipped amplitude (summit
   silhouette identical; 5 km lattice ±2 m; the ground dropped one voxel). Which
   makes item 2 below the live question, not a footnote.
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

Original charter (for the record):
The gap was `erosion.rs` incising bedrock with a single global `k_bedrock` —
granite and mudstone eroded identically, so the world had no differential
erosion anywhere. The data already existed (the recorder knew the exposed unit
per cell per epoch); erosion never asked. Design notes honoured: agent-specific
resistance (NOT a scalar — the limestone cliffs-vs-caves trap); composition order
with `resist`/`wmult` stated; basement/shield prediction checked and holds;
feedback stability clamped; off-by-default and byte-identical.

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
1. ~~**The world-creation ritual grows 15 s → 25 s (+66 %)**~~ — **RATIFIED
   2026-07-20, and the number was pessimistic**: measured on the shipped path
   the ritual is **13.79 s** (biology's marginal cost +2.6 s, not +10 s). The
   25 s figure came from the spike harness's *scalar* driver; production takes
   S9b's byte-identical parallel path — **corrections #12**, journal/0026.
2. ~~**Flipping `production_config`'s `biotic` to true**~~ — **DONE**
   2026-07-20 (journal/0026). Biology runs in every new world; the `DeepField`
   changed for every world created from here.
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

*(**Collapse-tier organic materials + the production flip: SHIPPED**
2026-07-20, journal/0026 — see `ROADMAP-history.md` § Shipped. The `Biofacies` → class routing is in,
coal/peat/carbonaceous-mudstone exist, and the 24 m seam at world voxel
(107338, 58787) is diggable. Two follow-ons fell out of it, both
measurement-backed:)*

**Charcoal as an inclusion, not a band** (journal/0026, measured): the fire
record is the third most numerous facies (158 310 beds) and **none of it
survives the collapse tier** — mean bed ~3.5 cm against a 0.9 m voxel, 0 of
158 310 kept. A charcoal *band* is therefore impossible at this voxel scale, so
no charcoal material was shipped. The honest representation is the one geology.md
§ inclusions already ratified: a few dark eighths riding inside the host stratum
above the burn, exactly as the placer puts gold in gravel and 3d puts olivine in
basalt. Blocked on a mechanism, not a decision — `deposit_deep_history` currently
*drops* sub-voxel units, so this needs a redistribute-into-host rule that touches
every dropped unit (mineral ones included) plus an inclusion channel on
`StrataEvent` distinct from the placer's `ore`. Its own slice, with its own
invariant work.

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

**Water-model design pass** (ratified 2026-07-19, user; field-notebook
first per the earth-processes method): groundwater as "another dimension
for the flow to go" — water table / aquifers (S8 per-voxel porosity is
the waiting substrate), ponds and sub-resolution water (procedural-tricks
tail), visible/flowing water (couples to PBR-2 water), lakes/inland seas
already implicit as deep-tier flooded basins (spill levels known).
**Groundwater ↔ CAVES coupling flagged by the user** — speleogenesis as
the eventual cave story (today's caves are S1 noise carving). Design doc
before any code.

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
    at `collapse.rs:1409` — different site, different tier, same root.
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
- **DIAGNOSED 2026-07-25 — `world_get_contents` reports `dc:air` and `has_contents: true` over
  solid, correctly-unrecorded rock** (walk observation journal/0097; diagnosis
  `docs/audits/2026-07-25-contents-empty-over-solid-diagnosis.md`, probe
  `dc-worldgen/examples/contents_air_over_solid_probe.rs`). **The original premise is falsified**
  (corrections #49): the world was never empty there, and **`eye_in_solid` was the honest
  instrument** — it and the query's own `block` field read the same `block_at` and both said
  `dc:stone`. Unrecorded basement is `Block::Stone` by construction (`collapse.rs:457-459`) and no
  voxel below a column's height can be `Block::Air` at all (`collapse.rs:434-436`).
  **Mechanism — a dc-api query-surface defect:** `contents_at` answers a **per-voxel** question
  with a **per-chunk** presence test (`host.rs:320-326`), so an *unrecorded* basement voxel sharing
  a 32³ chunk with any recorded voxel returns `Some(VoxelContents::EMPTY)` — reported as
  `has_contents: true` (`host.rs:83`, contradicting `payload.rs:426-429` / `schema.rs:754-756`)
  with `classified: dc:air` (`host.rs:76-79`, the operation `classify.rs:31-45` explicitly
  forbids). Reproduced **to the voxel** at journal/0097's own station: phantom band 288–299, honest
  from 287 down — the transition is the **chunk floor `9×32`**, not anything in the world.
  **Global:** 702/10,985 solid voxels (**6.4 %**) across 169 columns; 39/169 columns affected;
  worldgen authority only. **Blast radius:** the F3 HUD (`inspector.rs:125-133`) and
  `character_sense_raycast` (`host.rs:1424-1426`) carry it identically — their correct "no contents
  record here" branch is **unreachable** in this case; the mesher (`meshing.rs:295-306`) and far
  field (`farfield.rs:134-152`) are **immune**, which is why only the *diagnostic* surfaces ever
  showed it. **NOT the bare-cell fallback** — that is a real thin-record *generation* artifact;
  this is a *reporting* artifact over a healthy record. Both entries stay.
  **OWED — and it is the `identify(pos)` arc's first concrete requirement:** a tier flag that can
  say **"unrecorded"** as a first-class answer, distinct from both "air" and "recorded". Not fixed
  here (diagnosis-only agent).

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

- ~~**Two declaration defects on the new `dc:deep/weather_inventory` pass**~~ **✅ BOTH FIXED —
  verified at source 2026-07-29** (S6 finding F2). This entry read as owed work and was not.
  - **(a) shipped as the honest fix, exactly as diagnosed.** `crates/dc-worldgen/src/deeptime/runner.rs:542-544`
    now carries `BioMod` in the within-epoch **`reads`** rosters (`WINV_READS_AGENTS` /
    `_TEC` / `_LEG`), the pass declares `reads_prev: &[]` (`:889`), and the module comment at
    `:523-530` states the reasoning verbatim — *"a `reads_prev` declaration was a fiction …
    Declaring it as a real `reads` makes the graph state what actually happens and PINS the order
    instead of inheriting it from a tie-break."* Shipped by the spine-audit follow-through,
    2026-07-25.
  - **(b) fixed by DELETION, which is the honest disposal.** `Exposed` is no longer declared at
    all; `runner.rs:532-535` says so out loud — *"**`Exposed` is deliberately NOT declared**:
    susceptibility is a constant off `BEDROCK_SEAM_MATERIAL` … declare what you read, not what you
    intend to read"* — and `:1454-1456` **asserts** it (`assert!(!w.reads.contains(&DeepAxis::Exposed))`).
    The false declaration was removed rather than made true, and there is now a test that fails if
    anyone re-adds it without the genesis heir. *(Original diagnosis below, kept for the record.)*
  - **(a) `reads_prev: &[BioMod]` is not what happens.** The pass reads `grid.bio_weather`, which
    `dc:deep/biotic` overwrites **in place** each epoch; no edge is declared against `biotic`, so
    which epoch's plane it sees is decided by `passgraph`'s id-lexicographic tie-break
    (`dc:deep/biotic` < `dc:deep/weather_inventory` ⇒ it reads **this** epoch's, not last's).
    *Rename the pass and the physics changes* — the declaration is a fiction the graph does not
    enforce. Honest fix is free: move `BioMod` into within-epoch `reads` (adds only
    `biotic → weather_inventory`; nothing reads `Saprolite`, so no cycle). Note
    `weather_inventory_is_absent_off_and_a_declared_cellular_pass_on` asserts the **declaration**,
    not the behaviour — green about the wrong thing (**A-3** shape). **(b) `Exposed` is declared
    but never read** (susceptibility is a constant off `BEDROCK_SEAM_MATERIAL` until stub #16's
    genesis heir lands); the comment claiming it reads "the exposed lithology" is untrue today.

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
  It is the **S-4 rule unmet**. **One live-probe question left:** are the visible squares
  the 460 m deep cells or the 28.8 m chunk staircase — settle by reading dominant surface
  material per chunk across an east-coast patch (`world_get_block` / the inbound
  inspector: change every 28.8 m ⇒ member stepping; only at ~460 m ⇒ deep-cell tiles).
  **Planning held.** **DIAGNOSTIC STATION — return here to
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
- *(**HOLES IN THE GROUND: FIXED** 2026-07-21, journal/0057 + corrections #29.
  Culling is now **occupancy-aware**: the neighbour predicate returns `f32`
  coverage and faces resolve **by span** — emit the band `[cover, frac]`, cull
  only when `cover >= frac`, so exactly one side owns each band and nothing is
  coplanar-doubled. Bottom faces get the mirrored fix. `height_frac` had been
  re-deriving loose-only privately, so the mesher was holding **the very second
  opinion the fill contract exists to prevent**; one `cover_frac` rule now
  serves both the interior path and the cross-chunk closure. Cost **+30 %
  triangles on a synthetic worst case** (randomised loose depth per column) —
  an upper bound, since real depths are spatially correlated and equal-height
  pairs still cull. Seven tests by name including both border cases.
  ~~**UNWALKED** — nobody has seen the holes gone; re-walk
  `0056-holes-after-settle.png`'s coordinates, and if bands persist, distrust
  journal/0057 first.~~
  **✅ WALKED AND CONFIRMED — the tag was discharged on 2026-07-22 and nobody
  updated it** (S6 finding F1, applied 2026-07-29). The walk-0059 session at
  `--horizon 3 --fullbright --edges` answered it YES by eye: **no sky-holes** at
  two partial-rich stations, journal/0057 confirmed. The confirming entry sat
  ~1,160 lines below this tag in the same section for seven days and was never
  read back against it — the closure-in-the-wrong-place shape, inside the one
  file every session opens. **It is now in `ROADMAP-history.md` § Observed —
  archived** (struck by the user 2026-07-29 for its *third*, still-open answer;
  the two YES answers are preserved there verbatim). Assets `0059-*`.)*
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
- **HOLES IN THE GROUND — partial voxels are missing side faces** (found in
  the live walk 2026-07-21, **diagnosed by the user**; FIXED — see above).
  Symptom: sky-blue bands straight through the terrain in a rectilinear
  pattern, persistent (identical screenshots 20 s apart — not a streaming
  transient). Assets `0056-nearfar-check-after-3km.png`,
  `0056-holes-after-settle.png`. The user's read, confirmed against the code:
  *"the bands you see are missing side faces. these partials mostly have no
  side faces - some of them do, following no apparent pattern."*
  **Mechanism:** `meshing.rs` culls side faces on a **block-tier boolean**
  (`neighbor_solid: &dyn Fn(..) -> bool`), so a 5/8 partial beside a 3/8
  partial has its whole face culled and the exposed 2/8 band is drawn by
  nobody. Its own header states the assumption that made this safe —
  *"Worldgen does not yet emit sub-8 loose voxels"* — which journal/0055
  falsified world-wide this morning. The dc-core occupancy primitives added by
  journal/0052 exist precisely for this. **The wider lesson:** journal/0010
  shipped partial-height rendering dormant and predicted it would "light up
  for free the day deposition produces its first sub-full column". It lit up
  and did not work, because the assumption it rested on lived in a doc comment
  nobody re-read when the world changed underneath it.
- *(**"Surface material is quantized per chunk": FIXED** 2026-07-21,
  journal/0058 — the member is now drawn per voxel column inside the shared
  `surface_sample` kernel. Chunk footprints expressing more than one surface
  member went **0/169 → 147/169**. Block fingerprints unchanged in both
  recorded worlds, which is within-class invariance confirmed by an 80-chunk
  fingerprint that knows nothing about the argument. ~~**Unwalked** — the fix
  landed after the user's session closed, so nobody has seen the patches
  gone.~~
  **✅ WALKED AND CONFIRMED — the tag was discharged on 2026-07-22 and nobody
  updated it** (S6 finding F1, applied 2026-07-29). Walk 0059: **no 28.8 m chunk
  patches** — a 120 m top-down frame shows organic blobs with wandering
  contacts, journal/0058 confirmed. Same seven-day gap as the sky-holes tag
  above, discharged by the same entry, now in `ROADMAP-history.md` § Observed —
  archived.)*
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
- **GPU DeviceLost crash under a teleport storm at `--horizon 6`**
  (2026-07-21, live session, user present). ~65 s after a 10-jump ~28 km
  teleport sequence: `DeviceLost ("driver implementation is at fault")` →
  swap-chain loss → wgpu buffer-map panic → bevy_pbr cluster PoisonError
  cascade. Suspicion (UNDIAGNOSED — needs reproduction, not a bandaid):
  far-field rebuild churn — each long jump rebuilds toward a ~900-tile field
  plus near chunks/colliders — hitting either a Windows TDR (one >2 s GPU
  frame) or VRAM/allocator exhaustion. This is the failure class the
  perf-first doctrine (ARCHITECTURE.md § Modularity and performance,
  DECIDED 2026-07-21) exists for: pooling/recycling of far tiles and chunk
  meshes is the designed answer; voxy-dh-recon's pooled-vertex-buffer row is
  the prior art. Note the process exit code was 0 — the crash is invisible
  to exit-code monitoring; the panic cascade also poisons instead of
  degrading loudly. Secondary defect either way: a DeviceLost should not
  cascade into unwrap panics. Repro suggestion: scripted teleport storm via
  MCP at `--horizon 6+`, watched with GPU memory instrumentation.

- **Stub inventory filed** (`docs/design/stubs.md`, 2026-07-21, read-only audit
  agent + integrator). Ten active stubs, each with its heir. One genuine
  discovery: **ruin-posts was UNDOCUMENTED** — the only world-visible
  substitution with no placeholder marker anywhere; ~~a loud code comment is
  owed at `collapse.rs::ruin_posts`~~ **— ✅ VOID 2026-07-29: `ruin_posts` is DELETED**
  (S6 audit § 2 #3). `grep -rn "ruin_posts" --include=*.rs crates/` returns **zero hits**;
  `docs/design/stubs.md` #1 reads *"ruin-posts — RESOLVED BY DELETION 2026-07-28 (journal/0121); no
  heir was ever built and none is owed"*. The entry's one genuine discovery was an owed comment on
  a function that no longer exists. *(Note: `stubs.md:64-65` records explicitly that this deletion
  is **not** evidence for the ecology clause in CLAUDE.md — do not cite it that way.)* Two audit additions to the decision's holdout list:
  igneous emplacement-depth constants and paleo-temp-is-present-day. Also
  flagged: the `field.rs` doc-comment claims the collapse tier "reads"
  exhum/t_crust when nothing does — do not trust it.

- **The haze curve, not geometry, limits the usable vista** (journal/0042
  measurement, 2026-07-21). With `--horizon 8` the outer third of the field
  washes toward white and silhouette reading works only to ~5–6 km, even though
  the geometry is there and paid for. The fog *range* now scales with the
  horizon; the fog **curve** (its falloff shape) does not, and whether it should
  is a **user-owned visual call** the agent deliberately did not make. Cheap to
  change, needs the user's eye on a before/after.
- **Material placement rules are climate mocks, and below ~460 m there is no
  history to read** (user design observation + integrator analysis,
  2026-07-21 — NOT yet a design pass, nothing ratified). The surface veneer
  rule (`collapse.rs::surface_sample`) picks Grass/Dirt/Stone from year-zero
  climate + a 6.5 °C/km lapse against a −4 °C threshold — no slope term, no
  consultation of the record. geology.md § formation context already ratified
  that year-zero climate is legitimate **only** for the surficial veneer, so
  this is the documented last holdout of a dead shim. Two further gaps found in
  the same sweep: `exhum`/`t_crust` ship in `DeepField` explicitly as "the
  metamorphic-grade axes the collapse tier reads" and **nothing consumes them**;
  and there is **no rule deriving material *form*** (loose / pore-partial /
  whole block) from provenance, though the representation exists (S8 mixtures,
  pore partials). **Integrator's framing, unratified:** the collapse layer
  *samples and dresses* rather than re-simulating — shape below the 460 m deep
  cell is lattice jitter and material below it is member dither, so the
  sub-km-relief finding (Observed above) and the material-mock question are the
  same defect. Wants a priors-first design notebook before any work.
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


- **`--fullbright` is blind to geometry, and that cost a walk its conclusion**
  (2026-07-20, journal/0030 + corrections #18). DIAGNOSED, not yet fixed. In
  fullbright every face of a block is one flat vertex colour, so on terrain made
  of a single material there is no cue distinguishing a top face from a side
  face: `0030-flank-before-fullbright.png` renders an entire terraced hillside as
  a **featureless grey field** while the lit frame of the same geometry shows
  every step. The pass that correctly proved a *material* claim in 0027 silently
  answered "no change" to a *geometry* question in 0030.
  **User proposal, 2026-07-20: give block faces dark borders in fullbright** —
  "would give you more sense of dimension and help distinguish block positions."
  Agreed, and it is the direct fix for the failure above. Design notes from the
  agent that hit it:
  1. Prefer **crease/silhouette edges** (outline depth- and normal-discontinuities)
     over per-cube wireframe. What makes a bench legible is the *step*, not the
     grid, and per-voxel outlines at 3.5 km would alias into moiré where a voxel
     is sub-pixel. Fade the edge term out with distance.
  2. Ship it as a **separate flag** (e.g. `--fullbright --edges`) so the pure
     "colour in = colour out" control that 0027 depends on still exists unmodified;
     borders are a renderer-added signal and the data pass should stay data.
  Two adjacent asks from the same walk, both cheap and both currently blocking
  landform photography: **(a) `--fullbright` should also disable distance fog** —
  the 3.5 km summit vista washed to near-white in *both* passes, so silhouette
  work at landform scale is presently impossible; **(b)** nothing is needed for
  sun determinism — confirmed with the user that the sun is static, which is what
  makes the lit pass trustworthy for before/after diffing after all.

- **INSTRUMENT: `--fullbright` is BLIND TO SHAPE** (journal/0030,
  corrections #18). It renders unlit pure vertex colour, so every face of a
  block is the same colour — on single-material terrain a fully terraced
  hillside renders as a **featureless grey field**
  (`0030-flank-before-fullbright.png`, whose every step is plainly visible
  in the lit frame of identical geometry). A near-zero fullbright pixel-diff
  therefore does **not** mean "the shape did not change"; it means this
  control cannot see shape. **Choose the control that can see the question**:
  lit for shape/relief, fullbright for material/data. Proposed fix, filed
  not built: **crease/silhouette edge outlining under a separate flag**
  (`--fullbright --edges`) — outline depth and normal discontinuities only,
  distance-faded, never per-cube (per-voxel outlines alias into moiré where
  a voxel is sub-pixel at km range); separate flag so the pure
  colour-in-colour-out control that the 0027 coal diagnosis depended on
  survives unmodified.

- **INSTRUMENT: `--fullbright` does not disable distance fog** (journal/0030).
  The 3.5 km massif vista washed to near-white in *both* passes, so
  landform-scale silhouette assessment is currently impossible — fog, not
  lighting, destroyed the frame (`0030-massif-*-fullbright.png`). Since
  fullbright exists to be a pure-data diagnostic register, atmospheric
  haze does not belong in it. Cheap fix; blocks silhouette work, which is
  exactly what the dismal-mountains thread needs.
  *(Related walk suggestion, not filed as a defect: crease-aware dark face
  borders under a separate flag — outlining silhouette/depth-discontinuity
  edges only, distance-faded, never per-cube, which would alias at range.
  Kept separate from `--fullbright` so the pure-data control survives.)*


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


- **No pooling/reuse of chunk or far-tile GPU resources** (user question,
  2026-07-20; read from source, not measured). Every chunk load
  `commands.spawn`s a fresh entity with `meshes.add(to_bevy_mesh(..))` — a
  newly allocated `Mesh` asset — and every unload `despawn()`s it, freeing
  the asset. Far tiles (`LoadedFarTile`) follow the same churn. Partly
  mitigated for free: Bevy's `MeshAllocator` slab-allocates vertex buffers
  (FF2a step 0 found our custom attributes only *select* a slab), and the
  ECS recycles entity ids — so the unmitigated cost is CPU-side `Vec` +
  `Mesh` asset churn on every chunk-boundary crossing. **Filed prior**:
  voxy-dh-recon transfer map already lists "persistently-mapped pooled
  vertex buffers (AZDO) → far-tile buffer management when tiles churn."
  **Unmeasured.** Per placeholder-state-is-not-intent, measure at the
  design-target scale (10 km field, hundreds of tiles churning while
  walking), not at today's 1.2 km — the far-field range knobs milestone is
  the natural vehicle for that measurement.

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

- Walk 8 + user live observations (journal/0009): **nobody owns body
  orientation** — `SetMoveIntent` never touches yaw, `SetLook` is the only
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

- Walk 7 loose ends (journal/0008): **unloaded-neighbour and far-mesh
  fallbacks still sample S1 `TerrainGen`** — near-field loaded chunks are
  worldgen, but the far LOD rings and load-radius border faces show the old
  hill-field; user-sighted in walk 8 as a *phantom old world ~500 m below*
  the real terrain that dissolves on approach. A visible artifact until the
  far field becomes worldgen/summary-shaped (pairs with the existing
  far-mesh Observed items). Single-material
  faces under fullbright are featureless color fields — information arrives
  with the 3c-2 dither and later the splat pipeline. The `Terrain` resource
  is retained solely as the 3/4-key legacy fallback. *(Scale-3 boot default:
  fixed in 3c-1; freeze wire-drop path: proven live in walk 7.)*

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

- ~~The embedded HostWorld never evicts chunks (~64 KiB per chunk ever
  streamed/edited); never-edited chunks are pure generator output and could
  be dropped freely (journal/0002).~~
  **✅ IT EVICTS — verified at source 2026-07-29** (S6 audit § 2 #1).
  `crates/dc-api/src/host.rs:524` is `fn enforce_chunk_budget(&mut self, protect: Option<ChunkPos>)`,
  a bounded LRU with hysteresis, called from `set_chunk_budget` and from the materialize path;
  `:246-250` documents it as a bounded LRU in which **edited entries are protected** — exactly the
  generated-untouched-vs-edited split this entry asked for. Shipped 2026-07-21 by journal/0051.
  **The refutation was 700+ lines above this line, in this same section, for eight days** (*"FIXED
  2026-07-21 (journal/0051): eviction landed, the march is flat (+27.4 → 0.00 MB/jump)"* — now in
  `ROADMAP-history.md` § Observed — archived). A claim and its own refutation coexisting in the one
  artifact every session opens: the corrections #65 shape. Discharged here so archiving the
  refutation does not leave the claim standing alone.
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
- ~~Site cap 240 (u8 RegionId) — concrete instance of S2's ledger-scale
  question (S7).~~ **✅ THE SUBJECT IS GONE — verified 2026-07-29** (S6 audit § 2 #4).
  `grep -rn "RegionId" --include=*.rs crates/dc-worldgen/` returns **zero hits** and `sites` is
  absent from `crates/dc-worldgen/src/pregen/mod.rs`; the bootstrap settlement-history content was
  removed 2026-07-28 (journal/0121). This was a scale constraint on content that no longer exists.
  *(The surviving `RegionId` hits in the tree are `crates/dc-sim/src/statistical/engine.rs` — an
  unrelated toy-world type.)* **S2's ledger-scale question itself is live and is not this entry** —
  it is the 🔔 TRIGGERED S2-checkpoint-facts entry below.
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

- ~~**We cannot see where runtime goes — the perf observability gap**~~ **✅ ANSWERED AS POSED —
  verified at source 2026-07-29** (S6 audit § 2 #5). The instrument exists:
  `crates/dc-client/src/perf.rs` — `:26` documents that *"under `--features perf` it expands to a
  real `bevy::log::info_span!(..)"*, `:40` is the expansion, `:68-69` pulls in
  `bevy::log::tracing::{Subscriber, span::Id}` for a custom collector. So the entry's flat claim of
  *"**NO** runtime frame/tick observability"* is false today.
  **The live residual is already its own entry, above:** *"the perf instrument can't show the
  frame-thread envelope"* — `PerfAggregate` sums self-time across the frame thread AND the
  task-pool threads, so per-thread-role attribution is what is actually missing. **A complaint
  about an instrument is not evidence the instrument is absent**, and the two entries sat ~1,000
  lines apart never referencing each other. *(Original entry below, for the record.)* (user,
  2026-07-22: "it's not easy for us to target where the perf killers are";
  same conversation as the CLAUDE.md runtime-is-sacred convention). The
  project measures gen-time rigorously (spike results, ritual A/Bs, budget
  counters) but has NO runtime frame/tick observability: no span-level
  profiling, no tick-time breakdown, no way to attribute the chunk-drop
  chop (Observed above) to gen vs meshing vs tick contention. **Enabling
  slice filed: wire `tracing` spans through the hot paths (chunk gen,
  meshing, collider tiles, far-field derive, sim tick) with Bevy's Tracy
  integration (`trace_tracy`), then capture a BASELINE profile of the
  known-bad scenario (vertical drop) into `docs/audits/` as the first
  ranked perf-killer list.** Instrument-must-see-the-question applied to
  time. Sequenced after FF2b-minimal merges (dc-client write-set overlap);
  pairs naturally with the erosion-budget flag slice already queued there.

- **Chunk gen time is now noticeable in vertical streaming** (user field
  report, walk 0071, 2026-07-22): dropping from height — so the adaptive
  load volume streams chunks *below* — gets so choppy that "time appears to
  slow to a crawl, sometimes." Observation only, no diagnosis: the symptom
  (sim time dilating, not just frame hitching) suggests generation work is
  contending with the tick rather than merely the renderer, but that is a
  hypothesis to test, not a finding. *(Sharpened same day with a prime
  suspect: `dc-client/src/streaming.rs:42` — `LOAD_BUDGET_PER_FRAME = 8`,
  generated SYNCHRONOUSLY on the main schedule; no AsyncComputeTaskPool
  anywhere in streaming. Far-mesh is the same pattern (2 tiles/frame,
  main-thread; journal/0023 filed "async is a drop-in", unclaimed). Chunk
  gen is a pure seeded function, so task-pool offload does not threaten
  determinism. Still profile before building — but the profiling slice and
  the async-offload slice are now an obvious pair, and the client runtime
  is otherwise nearly single-threaded against a deep sim that already
  proved byte-identical rayon parallelism.)* Distinct from the pregen-time
  non-constraint (that covenant covers world *creation*; this is runtime
  streaming). Couples forward to the octree substrate (coarse-below is
  exactly what FF2b-class nodes eventually provide while true chunks
  generate) — but likely wants profiling before any architecture is blamed.

- **The cake observation: minority phases guillotine at cell perimeters
  under the coherent draw** (user, 2026-07-22, on ratifying B1's look —
  the swirl-cake-slice metaphor). B1's dither made the SOURCE continuous
  but the SHARES it thresholds are still nearest-per-cell, so minority
  swirls die at the 460 m line while shared majorities flow through —
  the S-4 sharpening catching the residual half of its own fix (the cause
  is still cell-quantized; only the expression got smooth). Suspected
  harder-to-see sibling: the journal/0058 MEMBER dither should guillotine
  minority members identically at perimeters (unexamined — check when the
  cure lands). Cure named, assigned to the CoarseField extraction: near
  boundaries, seeded membership dither of the SOURCE CELL (bilinearly
  weighted), then draw within that cell's shares — the residue-(b) paired
  pattern at the surface. Also owed there: the toward-50/50 bias of
  interpolated-uniform noise (dice-sum CDF distortion) — either a
  CDF-corrected coherent source or the far-summarize register retires it.

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

- ~~**The `history.rs` reject-don't-crash skip is SILENT**~~ **✅ THE FILE IS DELETED — verified
  2026-07-29** (S6 audit § 2 #2). `crates/dc-worldgen/src/pregen/history.rs` does not exist and
  `RegionId` has zero hits in `dc-worldgen`; the bootstrap history content went 2026-07-28
  (journal/0121, −713 Rust lines) under *existence is not standing*. This entry filed an owed
  warning on a code path that no longer exists. **The pack-degradation doctrine it invoked (API.md:
  degradation must be LOUD) is untouched and still binds** — only this instance of it is void.
  *(Original entry below, for the record.)* (2026-07-20,
  from the circulation merge). When a relocated-settlement graph
  over-constrains the S2 pressure collapse, `history.rs` now skips the
  observation instead of panicking (integrator-approved) — but the
  pack-degradation doctrine (API.md) says degradation must be LOUD. A
  skipped world-history collapse currently emits nothing; it owes a named
  warning. Small.

- **Circulation is fidelity-correct but surface-invisible** (journal/0038
  record-walk, corrections #22). The ~30° Hadley desert belt is arid in the
  data (precip ~0.2–0.3, past the 0.32 biome threshold) but `collapse.rs`
  only bares the surface below precip 0.10, so it renders as grass — the
  eye reads elevation/temperature, and 30° is the greenest band. Small
  reconciliation slice, USER-OWNED appearance: decide how bare a subtropical
  desert should read (lower the bare threshold in the subsidence band, or
  raise subsidence magnitude, or add a distinct arid surface material short
  of full bare Dirt). Pairs with the amplitude walk once flag-plumbing
  lands — until then no flagged gen feature is walkable anyway.
  > blogworthy: "a climate the map can't see" — the gap between a
  simulation being correct and being legible.

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

## NEXT SESSION — written at the 2026-07-29 close (supersedes every earlier block)

**Sweeps first** (the `SessionStart` hook says which are due), then **[`docs/dependency-graph.md`](docs/dependency-graph.md)**
— new today, read-first item 1c — then this block. Then `corrections.md` **#71–#72**.

### The one sentence that matters
**The top blocker is gone** — journal/0122 fixed the hillslope operator, discharging *"until this
lands the engine cannot run erosion at ANY realistic rate"* — **and the two things it taught are
worth more than the fix: the shipped world had been 2.1× past its own numerical stability bound
for weeks with every golden green, and the pass hand-rolled a `dt` because the engine has none.**

### Ratified (user's terms)
- **Refinement primitives, DEFINED** — *"any primitive responsible for the way things are actually
  drawn in the runtime game where we interpolate/upscale fine chunks from coarse cells… **totally
  necessary for a plugin-agnostic engine**."* Zero members exist; the tier's first member is unscoped.
- **S-10 ratified** — the frozen-snapshot doubly-limited gather (2 instances: `sat.rs`, creep).
- **Stability belongs to the KERNEL, not RATE** — ruled, then **re-ruled the same day**: *"I really
  hate to make RATE more complex now. Couldn't substepping be solved within the field instead, where
  it takes `dt` from outside and calcs its own internal multiplier?"* **RATE is NOT expanded.**
- **RATE sequenced ahead of the refinement design pass.**
- **The pre-flip appearance hold is RELEASED, by the user withdrawing their own ratification** —
  *"I do not care if frost/wind magnitudes increase 45×, do not try for byte identicality on
  `calibrated_rates`, I do not care about my previous ratification on looks there."*
- **8 user field reports struck** as stale-by-moved-implementation; **U16 struck** on the grounds that
  grass/dirt are not generated in the current pack shape.
- **The S2 statistical tier is HELD** — probable future primitive; **do not find it a consumer**.
- **Graph-building folded into the session process** — start / throughout / `wrap` § 9.

### Falsified — the assistant's own first
- **#71 — "the erosion axis is engine work."** I read the newest artifact in the corpus (last
  night's close block) and repeated its side **while holding `CLAUDE.md` in context**, which says
  *every pass is content, including tectonics and erosion* **by name**. Second instance of #68's
  mechanism, source-vs-source. **A close block is a HANDOFF, not an authority.**
- **#72 — "the stability-limit story is falsified"** (#63 ii) is itself falsified. journal/0116's 4×
  refinement moved 100.8× past the bound to **25.2× past it** — its null was a statement about the
  number 4. Coefficient and limiter were never rival diagnoses: **the limiter MASKS the instability**,
  capping a flipped mode into a finite limit cycle that reads as stable and is deaf to `dt`.
- **I claimed `spines.md` § 3 lacked a third exit and wrote it into `ARCHITECTURE.md`.** It had all
  three, a day earlier. Corrected in place.
- **I imported the wrong clock** — argued substep cost under *"runtime is sacred"*, which governs the
  **gameplay** clock; gen time is free by doctrine. User-caught before it reached a doc.
- **I miscounted the open field reports** (15, actually 16) inside the brief that dispatched the work.

### First things next session
1. **`spine-audit` — still never run**, and now more owed: today added **S-10** and a **CoarseField
   § 3 row**. Its waiting finding is unchanged: **`spines.md` § S-6 still teaches order-derived-by-
   topo-sort as the exemplary compliant shape**, in the doc every brief justifies itself against.
2. **RATE** (`docs/dependency-graph.md` E3) — smallest it has ever been, because the stand-in already
   did the hard part and left a **known-good fixed point** to hash against.
3. **The refinement design pass** (E5) — its inputs (flux record, head field) are **built and idle**.

### ⚠ Owed / unverified — deliberately not done
- **17 escalations** from the corpus agent, filed and unactioned — `north-star.md` Deviation 2's
  enumeration (⚠ contests a user block), `ARCHITECTURE.md`'s social/history pipeline needing
  **ON HOLD not strike**, `ores.md` R8 naming (⚠ contests `geology.md`), **`journal/0059` the lost
  entry**. Each carries its own disposition line in `docs/audits/baseline-2026-07-28/`.
- **16 user field reports open**, each with an evidence line. **Two are ripe for your word: U2**
  (journal/0119 shows weathering is mass-neutral **to the bit** — the answer is *"it doesn't"*) and
  **U26** (premise falsified — `Quarter`/`Slab` *are* emitted — but the design ask is untouched).
- **`EROSION_CALIBRATION` re-pick is sequenced, not done.** 45 was fitted to the broken solve and
  **inverts** under the fixed one (cover now *thins* with the multiplier). `calibrated_rates` ships
  **false**. Pick against the **published band**, never a look.
- **`flow_cost_probe` still not `test = true`** — the probe the whole rule was earned on. Mechanical.
- **`journal/0117` does not exist** — skipped number or lost entry; nobody has checked.
- **`ROADMAP.md` is 5,176 lines** against 2,500 even after archiving 45 today.
  `scripts/roadmap_archive.py` does the mechanical half (**hardcoded path — edit before use**).
- **The two `sweep_due_hook.py` thresholds are still unverified guesses.**

### Running
**Nothing.** All three agents in, worktrees removed, branches deleted, tree clean, pushed, no lock,
port 7777 free. Gate verified on main at `8987944` — **822 tests, 0 failed**, `Compiling dc-worldgen`
from main's checkout after `cargo clean -p`, five new gate tests confirmed **by name**. No Rust has
changed since.

---

## NEXT SESSION — written at the 2026-07-28 EVENING close (SUPERSEDED by the block above)

**A `SessionStart` hook will already have told you which sweeps are DUE. Run them first** — that
is now the standing rule (user). Then read `corrections.md` **#68–#70**, then this block.

### The one sentence that matters
**The corpus got its first 100 % sweep and its first unprompted trigger — but the board's
*pickup* problem is untouched, and today's own work proves it: ~9 hours of shipped work had no
path into a cold session until this block was written.**

### Ratified (user's terms)
- **File-size conventions — SPLIT BY LIVENESS, NEVER BY TOPIC.** Move out the *cold* half; the
  threshold applies to the **hot** file. Three `.md` classes: **NARRATIVE** exempt (splitting a
  journal entry is harmful), **REGISTRY** 2,500 (split = archive resolved entries), **ARGUMENT**
  1,000. *Topic-splitting is disallowed because it trades the cheapest docs-ops failure (volume)
  for the most expensive (topology).*
- **Sweeps run FIRST THING**, incrementally from a watermark, FULL when the reference side moved.
- **Spike/audit docs: IMMUTABLE BODY, MUTABLE HEADER** — never rewrite a measurement; the
  **correction's author** stamps a banner on its target **in the same commit**.
- **Bio-based rock formation: no NEW work until the bio/eco gate.** The shipped biotic layer is a
  **seam with an heir** and rides as-built. Gate order: engine + all non-bio earth science in the
  ratified SDK-plugin shape → ecology → social. ***"Sufficiently complete" is a USER call*** — no
  checkable test, and **progress on earth science does not entitle anyone to open it.**
- **ORE DOES NOT NEED TO BE EXPOSED** — *"a voxel game **with digging**… no reason to treat it
  like everything needs to be discoverable on the surface."* Kills the lode-gold fork's blocker
  and `probe 3`. `ores.md` is **conceptually behind `materials.md`/`material-behavior.md`**.
- **`material_transport` ratified as-is**; `COMPETENCE_SCALE`'s *"mud, sometimes"* is **not** a
  knob to tune.
- **There is NO "general registry"** — deleted, not reworded. **A seam's success condition is that
  it DISAPPEARS.**
- **Refinement tier → candidate (c)**: engine owns primitives (field **and refinement** kernels)
  + runner; plugins declare all content. **The user flagged the list as non-exhaustive.**

### Falsified — the assistant's own first (#68–#70)
- **#68 — the recalibration rule I shipped and briefed was wrong within hours.** `calibrated_rates`
  is **OFF in production**; a recalibration nobody enabled voids nothing. **The hypothesis was in
  the brief going out, not in the report coming back.**
- **#69 — "the general registry" never existed**, and I then got the withdrawal wrong **twice**
  from the same ambiguous north-star sentence.
- **#70 — ore-exposure was an unstated premise riding inside measurement caveats.** ***A caveat is
  where an unexamined premise hides**, and none of the three sweeps look inside one.*
- **Four stalenesses I created during the session itself**, all caught by readers running against a
  frozen worktree. **The corpus goes stale from the inside, during the work.**

### First things next session
1. **`spine-audit` — the one sweep the baseline did NOT satisfy** (its watermark is `null` on
   purpose; all nine slices were docs-vs-docs). It has a finding already waiting: **`spines.md`
   § S-6 still teaches order-derived-by-topo-sort as the exemplary compliant shape**, with no
   strike anywhere — in read-first item 0b, against which every brief justifies itself.
2. **The erosion axis is marked settled and is not** — `ROADMAP:2513-2533`. Its null came from a
   probe **blind to `diffusion`** (96 % of export). ⚠ **Do NOT restate it as "supply-limited
   today"** — that holds only of the *calibrated* world, which does not ship. ~~**This is engine
   work and it is the highest-value thing on the board.**~~ **CORRECTED 2026-07-29 (user ruling;
   `journal/corrections.md` #71): erosion is DEFAULT-PLUGIN-PACK work, not engine work.**
   Read-first item 0 says it by name — *"every pass is content, including tectonics and
   erosion"* — and this close block contradicted it. It remains **the highest-value thing on
   the board**; only its side of the engine/content cut was wrong.
   - **✅ ITS BLOCKER IS GONE 2026-07-29 (journal/0122).** The hillslope operator that
     checkerboarded the regolith (stubs #29, the 🔴🔴🔴 item) is fixed: `diffuse` sub-cycles to
     the 1/8 monotonicity bound, the calibrated world's surface concavity rms goes **40.42 →
     0.30 m** and its ACF(1) **−0.867 → +0.185**, and closed hollows past 10 m go **818 → 12**.
     **What is now in front of the axis is a NUMBER, not a defect:** `EROSION_CALIBRATION = 45`
     was fitted against the capped operator and does not survive fixing it (45× now strips the
     world to 1.40 m of mean regolith). The ladder in journal/0122 is the input; **the
     multiplier is user-owned and appearance-class** and `calibrated_rates` still ships false.
     Also falsified in passing: `corrections.md` **#72** — journal/0116's "not a stability
     limit" was a 4× refinement against a register 100.8× away.
3. **The bulk-mechanical backlog**, if you want cheap wins: **37 Observed entries (29 %) already
   say ✅ DONE in their own bodies** — a pure archive job with no judgement calls; **8 spike/audit
   files need supersession banners** (drafted).

### 👁 OLDEST UNTOUCHED USER FIELD REPORTS — 8 days, and nothing schedules them
*New close-block line, 2026-07-28. **The sweeps keep the board accurate; nothing converts an
accurate open item into work.** Only the close block does — so it now carries the age of the
oldest thing the user personally saw and reported. **23 of 33 user field reports are open**;
these four are the oldest, all from **2026-07-20**:*
- *"Our dismal mountains"* (`ROADMAP:~4100`) — DIAGNOSED, four causes, unfixed.
- *"Thick units render as flawless monoliths"*
- *"Loose materials do not exist in the world yet"* — ⚠ **premise half-falsified**: journal/0055
  made sub-8 loose voxels world-wide, so the *renderer* half is live and **the user's actual ask
  is untouched.**
- *"The sim must know about light"*

**None may be closed on reasoning alone** — only on evidence the world changed. *A wrong
"resolved" on a user field report is the worst outcome a sweep can produce, which is why the
baseline reader refused to close the razor-straight grass/dirt frontier and asked for a re-shoot
instead.*

### ⚠ Owed / unverified — deliberately not done
- **~75 baseline findings are FILED, NOT APPLIED** (`docs/audits/baseline-2026-07-28/`, 9 audits,
  ~3,900 lines, ranked in § Sequenced). Only the integrator's own same-day stalenesses and the
  `ARCHITECTURE.md` banner were applied.
- **The two `sweep_due_hook.py` thresholds are UNVERIFIED GUESSES** made this afternoon:
  `STALE_AFTER_COMMITS = 25`, and a **keyword match on commit bodies** for "recalibrat". **The
  keyword rule already false-positived** — this session's commits *discuss* a recalibration and
  read identically to one that *performs* one. Tune from experience, not taste.
- **`ores.md` needs a conceptual revisit** against `materials.md` / `material-behavior.md` —
  **owed, unscheduled**, and larger than the one premise struck today.
- **`journal/0059` is a LOST entry** (orphaned assets, a substantive walk surviving only as
  `ROADMAP:4811-4830`); **journals 0110/0111/0112 have no archive entry at all**, including
  **0111, the scale recalibration.**
- **`flow_cost_probe` was never converted to `test = true`** — the probe the entire rule was
  earned on. A live defect, candidate corrections entry.
- **18 broken rustdoc links across two crates**; the gate is structurally blind to them. **Booked
  for a design conversation** — *"add `cargo doc` to the gate" is NOT the decided answer.*
- **The full test suite was NOT run today** and is **not claimed green**. The only Rust changes
  were doc comments; `fmt` + `clippy --all-targets --release -D warnings` covered them.

### Running
**Nothing.** All nine sweep agents in, worktrees removed, branches deleted, tree clean, pushed,
no lock, port 7777 free.

---

## NEXT SESSION — written at the 2026-07-28 MORNING close (SUPERSEDED by the block above)

**Read first: `.claude/skills/doc-topology/SKILL.md`'s opening stop-block** (new, and it is the
door to everything below), then `docs/design/corpus-knowledge-notebook.md` §§ 4–5, then
`corrections.md` **#66 and #67**.

### The one sentence that matters
**We spent the session measuring our own docs problem and it is not the problem we thought.**
**Staleness is 3–8 % of our recorded failures; ~85–91 % of them were wrong the day they were
written.** So a stale-ref detector has a single-digit ceiling *by construction*, and the target is
**assertion-time, not decay**. Two independent codings agree on every direction and differ on
magnitudes by ~2×, with **43 % of entries ambiguous** — so every number is a band.

### Shipped
- **The bootstrap history content is REMOVED** (journal/0121, −713 Rust lines). Gate green on
  merged main, **85 binaries / 808 passed / 0 failed**, reconciling exactly against 86/811 with all
  three dropped tests named and confirmed absent.
- **44 dangling cross-file pointers fixed** (the audit had estimated ~25 — it undercounted by 18,
  and 4 were already wrong *before* the archive split).
- **`corrections #67` — the first entry filed against another entry.** #40 was itself a
  misdiagnosis.
- **Three process fixes + the discoverability fix** (below), and the **push** standing instruction.

### Ratified (user's terms)
- **Keep `origin/main` up to date from here out** — *"push it and we'll continue to keep the remote
  up to date from here out."* Standing authorisation; folded into `session-workflow` § Integration
  and `wrap` § 10. *`origin` had sat 45 commits behind, 19 of them from before this session.*
- **The docs diagnosis must be discoverable from a SKILL, not from CLAUDE.md** — *"if i tell an
  agent the docs are a mess… this ought to be discoverable near immediately, in our process, likely
  via an appropriate sounding skill that may already exist."* Done: `doc-topology`'s description now
  names the trigger phrasings and its body opens with the four results. **CLAUDE.md deliberately
  unchanged.**
- **The five user-owed decisions are PARKED, not forgotten** — *"i won't muddy waters by addressing
  that in this conversation."* § Sequenced → ~~**USER DECISIONS OWED**~~ **"✅ ALL FIVE USER
  DECISIONS RULED 2026-07-28"** (pointer repaired 2026-07-29 — no such heading existed; and
  they are ruled, not parked), context inline.

### Falsified — the assistant's own first
- **My adoption law, falsified by me mid-session.** *"A convention is adopted iff a machine consumes
  it"* — **false**: five documented conventions with **no** consumer sit at 97–100 %. What survives:
  *a convention is adopted only if it is inseparable from something the author must do anyway, or is
  the natural way to say the thing.* `JUSTIFIED-BY` (3 uses, 0 in `crates/`) vs `heir` (651).
- **My staleness count was an overcount** — I said ~8 %, the independent re-coder found **3.0 %**,
  in the direction that flatters tooling.
- **My first reciprocity instrument reported `0/67`, which is impossible** (`\b` in a POSIX-ERE
  grep). A 0 % that flattered the thesis was one publication away.
- **I quoted my own single-coded figures as settled in this board for several hours after the
  re-coding revised them** — the *summary-that-outran-its-source* shape, committed by the author of
  the finding. Fixed at the wrap.
- **`corrections #40`** (assistant, 2026-07-23): the seam audit did not paraphrase; it quoted the
  comment *as it stood the day before*. **A stale READ, not a stale claim.** → #67.
- **`corrections #66`** (the removal): *"the goldens will move and that is correct"* — **not one
  moved.** *A pre-authorised golden move is indistinguishable from an unexplained one.*
- **`#56` asserted, unstruck, exactly what `#60` withdraws**, 220 lines away, with **no `#60` token
  anywhere in the file.** In the artifact whose whole job is recording falsified claims.

### First things next session
1. **RATE, with the creep limiter as its acceptance test** — unchanged and still the 🔴🔴🔴 top
   blocker. Acceptance: concavity ACF(1) back toward **+0.38** with closed hollows at **zero**,
   paired with a neighbour-relative measure (corrections #61). Brief must **name what the criterion
   is NOT**. *It is also a live specimen of the "ratified-but-unbuilt" obligation class — authored
   in prose, consequence named, untracked for four days while its absence produced the blocker.*
2. **A decision on the knowledge layer, not more analysis.** Notebook § 5 proposes a **reciprocity
   check** (assistant-originated, unratified): *does every artifact that supersedes another by name
   carry a back-pointer?* Zero new authoring, 8 known failures. Or park it. ~~**`stubs.md:22`
   forbids designing the general mechanism first, and that binds this thread.**~~ **WITHDRAWN
   2026-07-28 (user)** — there was never a registry to defer, and that clause never governed this
   thread. **`session-workflow` § Seam-first #6 still binds** and says the same thing about how to
   build anything.
3. **A `doc-topology` sweep is due** — an arc shipped today and the skill's shape-6 check changed.

### Gate
**Green on merged main at `a378d47`** — fmt 0, clippy 0, **85 binaries (78 + 7 doc-tests) / 808
passed / 0 failed / 3 ignored**, 0 errors/panics/FAILED, all four byte-identity goldens present by
name and `ok`, `Checking`/`Compiling dc-worldgen` from **main's** path. Commits after it are
docs-only.

### ⚠ Owed / unverified
- **I dispatched two agents WITHOUT `isolation: "worktree"`** — both ended up in the main checkout
  and one nearly swept an untracked notebook into its commit. My error, twice. Pass `isolation`
  explicitly.
- **One re-coding claim I did NOT verify myself:** that #60's falsifier sits two sentences above the
  claim in `journal/0111:119-129`. Four of its five I verified directly; this one rides on its
  citation.
- **The removal agent's gate/probe logs died with its worktree.** The numbers are in journal/0121
  and the merge commit; the logs are not recoverable.
- **Still not read, declared:** `north-star.md` bodies beyond §§ boundary→Deviations,
  `material-behavior.md` § 5 at source, the domain-doc bodies, spikes, `ROADMAP-history.md`, and
  individual journal entries other than 0119.
- **Owed on the analysis:** a second reader on the § 3.5c coding rule *(one done — a third would
  settle the 43 % ambiguity)*, and the 65 re-coded at § granularity.
- **`JUSTIFIED-BY`'s fate is a main-session call** — `spines.md` § 5 left it there 2026-07-24 and
  `spine-audit` check #4 still tells auditors to grep it. Today's number (3 uses, 0 in `crates/`)
  argues for rewriting § 5 around the prose form, but § 4's rule binds me as much as an auditor.
- **`erosion.rs` ~4,000 lines**, split sequenced not done. **File-size thresholds still the hook's
  guesses.** **CI remains deleted.**

### Running
**Nothing.** All agents in, all worktrees removed, all branches deleted, working tree clean,
**pushed — `main` == `origin/main`**, no held lock, port 7777 free.

---

## NEXT SESSION — written at the 2026-07-27 close (SUPERSEDED by the block above)

**Read first: `journal/0116` and `journal/0119`.** The erosion blocker is localised, and the
pass architecture changed underneath it. Then `ARCHITECTURE.md` § *The engine is
plugin-agnostic, and pass ORDER is authored*, and `corrections.md` **#61–#65**.

### The one sentence that matters
**The blocker is `erosion.rs::diffuse_scale_cell`** — it caps a cell's hillslope export at
its **entire regolith inventory**, with **no `dt` in the expression**. A donor-cell scheme
that moves everything downslope has a **period-2 mode by construction** (A gives all its
cover to B; B is now higher and gives it back), independent of step size. Measured concavity
ACF(1) **−0.87 / −0.91** against the shipped world's **+0.38**. **It is not the incision
clamp, not the timestep, and not isostasy** — all three were tested and killed (journal/0116).

### Ratified (user's terms)
- **THE ENGINE MUST BE MOD/PLUGIN AGNOSTIC, FULLSTOP, EMPHATICALLY.** Passes are **plugins**,
  viewed through the lens of third-party mods; *we are our own first modders*. **ORDER is
  authored per world**; `{reads, writes}` become the **validator**, not the generator.
  `DeepAxis` is the named violation.
- **The product:** a voxel crafting-game **generator**, each world its own sim composed from
  declared plugins, whose **first pack** is the earth-like generator.
- **Remove the bootstrap history content** (polities/sites/ruins) — *"unratified zealous
  fabrications… they are NOTHING."* **Existence is not standing.**
- **Trust is DEFERRED** — no difference in permission between native and WASM; do not
  justify a core/content placement by trust.
- **`ecology.md` stays a ratified user design**, but is dormant and open to reconsideration.
- **Burial-dominant coal rank is correct physics**; the half-thickness term is a **defect**.

### Falsified — the assistant's own first (#61–#65)
**#61** a *global aggregate* ("relief within 5 %") cannot license a claim about *local
structure* — relief +4.6 % while cell-to-cell roughness went **×170**. **#62** the pit census
**saturates**: "below all eight neighbours" is a ranking test wearing a magnitude test's
clothes, and it undercounted 2.6× — **caught by the user flying the terrain**, after three
instruments agreed because they were the same instrument. **#63** the stability hypothesis,
specified against `myr_per_epoch`, *a knob that does not exist*. **#63b** isostasy is the
only grid-scale **damper**, not the driver. **#64** part (a)'s blast radius was assembled by
symmetry. **#65** the phase order never "fell out of the declarations" — it was **fed in**.

### First things next session
1. **RATE, with the creep limiter as its acceptance test.** *Not* a patch beside the
   architecture — journal/0116's prescription is *"a transfer that stays a function of
   `rate × dt`"*, which **is** the RATE axis (ratified 2026-07-24, never built, `dt` pinned
   to 1.0). Acceptance pairs an aggregate with a **neighbour-relative** measure: concavity
   ACF(1) back toward **+0.38** with hollows at **zero**.
2. **The corpus-addressability design pass** — frontmatter, tags, query scripts, and
   **versioned standing models**, booked by the user. Third leg of docs ops beside the
   archive and the sweep.
3. **The bootstrap-content removal** (also discharges draw-domain part (a)).
4. **Doc-topology residuals** — ~25 dangling cross-refs from the split, and the coal
   evidence base mislabelled "the production world" in four docs (needs a **measurement**).

### Gate
**Green on merged main at `8bcca7b`** — fmt 0, clippy 0, **86 binaries / 811 passed / 0
failed**, verified by name, reconciling exactly across all four merges (+1 binary and +4
tests are the discriminators'). *The first attempt reported "exit 0" while having run **10
binaries of 86** — the harness exit was the task's not cargo's, `$LASTEXITCODE` was empty
because a cmdlet ended the pipeline, and the lock was gone because `;` is unconditional.
**Only the impossible count caught it.***

### ⚠ Owed / unverified
- **The live-magnitudes tour** is more owed than before (stubs #28: wave/wind/frost are 45×
  weaker relative to the landscape than the day their numbers were chosen).
- **`erosion.rs` ~4,000 lines**, three separable concerns; split sequenced, not done.
- **`ROADMAP` § Observed (~1,970 lines)** is the largest unswept surface — next sweep's spine.
- **File-size thresholds are still the hook's provisional guesses**, not the user's numbers.
- **CI remains deleted**; if wanted it needs designing, not resurrecting.
- **journal/0117 is a deliberate gap. ~~and 0120~~ — 0120 WAS WRITTEN 2026-07-28** (the five
  user decisions), which is exactly what the sweep reserved it for. **And the real gap is
  `journal/0059`** — found by the baseline sweep: three orphaned assets
  (`journal/assets/0059-dune-field-*`, `0059-loess-margin-*`), a substantive walk with a
  user-diagnosed defect surviving only as `ROADMAP.md:4811-4830`, and **nobody ever decided to
  skip it.** *A reserved number and a lost number look identical from the outside; that is why
  gaps get accounted for in the close block — and 0059 slipped through the accounting that was
  built to catch it.* — the rename slice and the sweep each judged
  a narrative entry unwarranted and said so. Not lost entries.

### Running
**Nothing.** All agents in, all worktrees removed, all branches deleted, working tree clean,
no held lock, port 7777 free.

---

## NEXT SESSION — written at the 2026-07-26 MORNING close (SUPERSEDED by the block above)

**Read first: `journal/0111` and `journal/0114`** — the world is ~10³× too slow, and the
reason is now known to be **both** the constants *and* a capped transport operator. Then
`docs/design/material-genesis-notebook.md` (opened today) and `north-star.md` §
*"The refinement tier is in neither list"*.

### Shipped (journals 0108–0114; corrections #53–#60; stubs #21–#29)
- **The clock nobody checked (0111).** The world denudes at **0.0110 m/Myr** — **9× slower
  than the slowest landscape ever measured on Earth**, stripping 5.48 m where a craton strips
  5–10 km. *The single most active cell of 44,264 is still below the global floor.*
- **Material-aware creep (0112)** — provenance in the archive **0.000006 % → 65.206 %**.
  §13.2's gravity member is the fluvial one **with the competence curve removed**; that
  absence *is* why colluvium is unsorted. Hillslope columns +64 % distinct species vs +11 % in
  valleys — **the contrast landed six times harder where colluvium belongs**.
- **Hybrid `p` (0113)** — peak catchment **84 → 298**, p99 **beating D8**, 95.8 % of
  simultaneous divergence retained. *Under uniform `p` the world had **no trunk network at
  all**: zero land cells draining >100.*
- **The joint calibration (0114) — BUILT, MEASURED, AND OFF.** See the blockers below.
- **S20 2c (0108)** — `Fact` 16 → 8 B, every axis kept; per-depth projection 973 → 504 MiB.
- **MFD (0109)** — simultaneous divergence **0 → 7.5 M**; *the tree never licensed the
  traversal, the potential did.*
- **Movement 2b (0110)** — identity travels + sorted deposition; its null diagnosed the above.

### Ratified (user's terms)
- **The genesis discriminator is RESOLUTION, not phase** — *"can the prior be named as a
  material we track?"* The material ontology is a **sieve**: some origins are transformations
  of **molecular parts**, and we have no molecular parts. **That is the whole case for genesis
  passes.** Four-way test, with **"transform whose driver isn't built yet"** as its own
  category, and filing one of those as a genesis named as the **irreversible** error.
- **Clastic facies settled:** genesis makes the *parent* honest · weathering the *loosening* ·
  transport the *destination*.
- **S20 option 3 + 2c** (paged facts + compact encoding). **2b now, on today's faces.**
- **File size is a correctness problem** — *"Claude must grep, missing context or reading too
  much irrelevant context"*. **Hook live and proven to fire.** Immediate for new files,
  gradual for old when touched.
- **Visible river channels are the first thing built on refinement primitives.**

### Falsified — the assistant's own first (#53–#60)
**Three of today's corrections are the integrator's own bookkeeping**, and two were
propagated onto this board: **#58** — *"p → ∞ **is** D8 exactly"* (the limit takes steepest
**slope**, `route_cell` takes steepest **drop**; it was the argument that `mfd:false` lives
*inside* the model's family, and would have licensed **deleting a pinned path**). **#60** —
*"two independent instruments agree to 2.4 %"* was a **low-flux coincidence, not a
cross-check**; D1 double-counts cover that re-crosses a shoreline cycling ±35 m four times.
Also: **#53** eight of S20's nine predicted tolerance breaks **did not happen**; **#55** the
2b null was honest about mechanism and **wrong about cause**; **#57** peat/coal/charcoal
cannot be *deposited* — §12's four-way test caught in the wild **hours after being written**;
**#59** `energy_band` was an absolute threshold **secretly keyed to `k_transport`**.

### ⛔ THE TWO BLOCKERS — read before planning anything erosional
1. **stubs #29 — ⚠ RE-SCOPED BY THE WALK (journal/0115) AND DIAGNOSED BY THE
   DISCRIMINATORS (journal/0116, corrections #63). It is NOT "the incision clamp", it was NOT
   148 pits, and it is NOT a time-step stability limit.** Above 1× the surface carries a
   **checkerboard** — concavity lag-1 autocorrelation **+0.38 (shipped) → −0.87
   (calibrated)**, against derivable references of −1/6 for white noise and −1 for a pure
   oscillation. **Relief grew 4.6 %, cell-to-cell roughness ~170×**; closed hollows
   **0 → 1,377 (3.1 %), 5.4 km³**. Split `surf = r + h` and the two summands separate: the
   **bedrock converges under 4× time-step refinement** (23.8 → 5.9 m) while the **regolith
   sharpens** (ACF −0.82 → −0.93), because the creep flux limiter is **deaf to the step**
   (96.0 → 94.7 % binding) — it caps export at *inventory*, not at `rate × dt`. **The register
   is the flux limiter / donor-cell partition in `erosion.rs::diffuse`.** Do **not** brief a
   clamp fix, a time-step fix, or anything touching `iso_rate` (isostasy is the only
   grid-scale damper in the solve; turning it off puts **6,215 hollows in the SHIPPED
   world**). **Until this lands the engine cannot run erosion at any realistic rate.**
2. **stubs #27 — the transport operator has a ceiling.** Export ∝ mean regolith thickness, and
   creep's limiter **already binds on ~89 % of cells at shipped rates**. 100× on transport
   buys **1.6×**. *So journal/0111's "a calibration, not an architecture" was **half right**.*

### Running
**Nothing. All agents in, all worktrees removed, all branches deleted, working tree clean.**

### Gate
**Green on merged main at `a05008f` — fmt 0, clippy 0, 807 passed / 0 failed / 85 binaries**,
verified by name, reconciling exactly across all five slices. Production is **byte-identical**
(`the_production_world_still_hashes_to_the_pre_slice_goldens`).

### ⚠ Owed / unverified
- **An appearance walk is owed** — creep's interbedded colluvium is the first thing in three
  slices worth standing in front of. Tour-map first; stations: hillslope road cut, scarp-foot
  apron beside a channel deposit, the re-baselined coal site. **Look at the pits first.**
- **`erosion.rs` is ~4,000 lines** and hosts three separable concerns (drainage/MFD · fluvial
  transport · diffusion+creep). Split **sequenced, not done** — deliberately, siblings were live.
- **ROADMAP is ~6,850 lines** and the hook flags it on every edit. The archive-by-status
  scheme is still undesigned.
- **CI was deleted** (day-one scaffolding, 3-OS debug matrix, failing on Bevy's Linux deps and
  burning 2.5 h Windows runs). If wanted, it needs designing, not resurrecting.
- **Commit trailer mismatch:** CLAUDE.md § Conventions says `Claude Fable 5`; this session's
  commits say `Claude Opus 5` (the model that did the work). **User's call which is canonical.**

### First things next session
1. **stubs #29 — the REGOLITH-CHECKERBOARD blocker** (re-scoped by the walk 2026-07-26; then
   **diagnosed the same day, journal/0116, corrections #63 — the discriminators are RUN and
   the stability-limit hypothesis is dead**). It gates every erosional number. Brief the fix
   against the **flux limiter / donor-cell partition in `erosion.rs::diffuse`** — *not* the
   clamp, *not* the time step, and **not `iso_rate`**.
2. **stubs #27's heirs** — rivers that carry, or a non-capped creep operator. **These two
   share the LIMITER, not the clock** — #29's "same `cell_m / myr_per_epoch` register" claim
   is withdrawn (corrections #63; the knob does not exist and the epoch length is measured not
   to be the register). A non-capped creep operator is now plausibly **one slice for both**.
3. ~~The appearance walk (creep)~~ — **DONE 2026-07-26, journal/0115.** Flip
   `calibrated_rates` once #29 is fixed.
4. **Refinement primitives design pass** — unblocked by hybrid `p`; visible channels.

*(The 2026-07-25 block is consumed; preserved as history — **archived 2026-07-26 to
`ROADMAP-history.md`**, where it is the first of the six superseded close blocks. It used to
sit directly below this line.)*


