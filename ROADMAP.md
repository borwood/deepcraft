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
  parent-shadowing hierarchy + slug→assets — the `Providers` pattern dropped to
  material level), their behavior, and the passes over them are **authored in a
  uniform, self-declaring, compiler-validated shape and tuned by data**. Behavior
  is code, tuning is data. **Plugin-first, closed-source-OK, untrusted-third-party
  safe** via a tiered backend behind ONE authoring shape: native `abi_stable` for
  trusted/first-party (incl. runtime), WASM sandbox for untrusted (gen-tier — the
  two-clock reconciliation; the Minecraft-space one-better). The crossing
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

- **🟠 DOC-TOPOLOGY RESIDUALS — the 19 findings not actioned 2026-07-26** (full audit:
  `docs/audits/2026-07-26-doc-topology-sweep.md`). Six were actioned the same day (the
  ecology carve-out, the field-solver framing, the two ORDER strikes, the S-/A- undercount,
  the CLAUDE.md trust-tier line). **The rest are owed, and are recorded here rather than
  left in an audit nobody re-opens:**
  - **Mechanical, no user call:** ~25 dangling cross-file pointers created by the
    ROADMAP/history split (16 `"see § Shipped"` in `ROADMAP.md`, 9 the other way, plus one
    orphan pointing "above" at a block 3,800 lines below it). *Caused by this session; not
    cleaned by it.*
  - **`flow.md:715` puts a RATIFIED stamp on a scheduler model that was retired** 2026-07-26.
  - **`spines.md:1048` + `stubs.md:30-40` still schedule heirs for the bootstrap content the
    user decided to REMOVE.** These should resolve into the removal slice, not survive it.
  - **The ABI spike is described as "locking the SDK shape"** — contested by north-star
    § Deviations 1, which defers the whole trust/backend question.
  - **⚠ THE COAL EVIDENCE BASE IS LABELLED "THE PRODUCTION WORLD" IN FOUR LIVE DOCS**
    (`stubs.md:348-351`/`:360-362` — the sentence justifying `COAL_BURIAL_M = 8.0` —
    plus `corrections.md:1164`, `ROADMAP-history.md:1228/1234`, `journal/0066:43`;
    `geology.md:163` carries the number with **no world label at all**). It is the **warm
    reference** fixture `0x0D5E_ED57_2026`, and the shipped world has **zero coal**
    (corrections #51). **The label is wrong with certainty; the correct VALUES are unknown**
    — they predate the geotherm recalibration — so this needs a **measurement**, not an edit.
    *A user ratification sits directly on top of it (`stubs.md:378-379`).*
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
    that scales transformations (`dt` is pinned to `1.0` today and nothing scales by it).
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
  - **🔑 FIRST SLICE — RATE, WITH THE CREEP LIMITER AS ITS ACCEPTANCE TEST.** Do **not**
    build this as a patch beside the architecture. journal/0116's prescription for the
    blocker is *"a hillslope operator whose transfer stays a function of `rate × dt`"* —
    **that is the RATE axis**, so the blocker's fix is the axis's first real consumer and
    *the world becoming grid-stable is the acceptance test*. Acceptance pairs an aggregate
    with a **neighbour-relative** measure (corrections #61): concavity ACF(1) returning
    toward the shipped world's **+0.38** with closed hollows at **zero**, not relief holding.
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


- **🔴🔴🔴 THE HILLSLOPE CONVEYOR CHECKERBOARDS THE REGOLITH ABOVE 1× — stubs #29,
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

- **🔴 REFINEMENT PRIMITIVES — the tier nobody assigned an owner** (opened 2026-07-26 by the
  user's question: *"who owns the **clever** refinement operators for presenting the
  interpolated/upscaled runtime world? how would a mod hand-roll emergent rivers on their own
  via the SDK, **that are visible**?"*). **Read `north-star.md` § "The refinement tier is in
  neither list" first — the hole is recorded there in full.**
  - **THE HOLE.** `north-star.md`'s core/plugin boundary enumerates both columns and **the
    refinement tier is in neither.** It was never decided; it was arrived at *by default*.
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
    constraint, two payoffs) · **invocation granularity** (per-cell or per-chunk, **never**
    per-voxel — *runtime is sacred*, and seam-first already forbids a provider in a hot loop
    answering a question that does not change inside it) · and how a **mod** authors a visible
    channel end-to-end.
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
  - **OPEN — the conventions themselves.** Thresholds (source vs. doc vs. journal — a journal
    entry is linear narrative and may legitimately be long; `ROADMAP.md` at ~6,200 lines and
    `collapse.rs` at 2,415 are the two worst offenders), what "separate concerns" means per
    file type, and the split conventions. **To be set, not guessed.**

- **Finish the draw-domain conversion: the residual hand-rolled sites** (opened 2026-07-25
  by journal/0105, which converted 26 of them and named these). Small, and each is named in code
  so it cannot be lost. **(b) is DONE 2026-07-26** and its rider **re-measured (a)'s blast radius,
  which was overstated — see below**.
  - **(a) `dc-sim/engine.rs`'s region-step and agent-step draws** address `[seed, k, SALT, …]` —
    the sample index sits before the domain, so putting them on `Draws::of` (which fixes the
    domain at slot 2) changes the key. **Still a user-owned appearance slice, but a smaller and
    differently-shaped one than this entry claimed** (corrections #64, a read-only trace taken
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
  - **🔴 OWED BY MFD, and it is the nearest-term thing on this arc (2026-07-26, journal/0109):**
    **hybrid `p`**, plus **recalibrating `k_bedrock`/`k_transport`**. Uniform `p` **does not
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
    the knob deciding *which grain sizes this world can move at all*, whose measured answer today
    is **"mud, sometimes."**
  - **Appearance: announced, and a NULL IN THE VIEWPORT for the second slice running.** Surface
    elevation identical to two decimals. **No walk is owed** — a tour-map would find nothing to
    stand in front of.
  - **🔀 THE CONTINUATION ORDER IS REROUTED BY MEASUREMENT, and this is the user's call.** The
    corpus sequences fluvial refinements next; the numbers say otherwise:
    **(b) the GRAVITY/MASS-WASTING member of § 13.2 — newly promoted to FIRST.** Creep routes
    this world's sediment and **carries no identity**; making it material-aware is where a
    visible facies signal actually lives.
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
    passes" are **one arc**, and it fixes the palette-quant + LOD root cause (coarse facies
    *point-sampled*) by driving distribution from a smooth physical field + octaves instead
    of a chunk-grid dither. Rides the material-behavior model (cellular passes, edges,
    agents, hierarchy).
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
   corrections #41. **Nothing further to ratify on the erosion axis.**
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

- ~~**OWED (small) — `dc:deep/climate`'s lagged terrain read is undeclared**~~ **✅ DONE
  2026-07-25 (journal/0107), in the head-declaration slice as predicted — *"worth doing the
  next time that file is open"*.** Declared **`reads_prev: [Forced]`**, and the three
  cfg-selected slices this entry expected turned out to be **the wrong shape**: lagging against
  the *last* terrain revision (`Settled`/`Compensated`/`Diffused`) pins strictly **less** than
  lagging against the **first**, because it would leave `climate` free to slide past `forcing`
  and `transport`. One token, one slice, no cfg selection, and the whole chain covered
  transitively. Schedule-neutral, proven by
  `the_terrain_revision_declarations_are_schedule_neutral`. *(Original entry below, for the
  record.)*
- **OWED (small, as diagnosed) — `dc:deep/climate`'s lagged terrain read is undeclared** (found by the
  journal/0104 `reads_prev` audit, 2026-07-25; deliberately not changed by that slice, whose
  mandate was the mechanism, not the declarations). `climate` declares `reads_prev: &[]` while
  its own comment (`runner.rs`) says it reads the **start-of-epoch topography**. That is a real
  lagged read of the terrain, undeclared. **It is not dangerous**, and that is the whole
  distinction the anti-dependency edge exists to draw: the ordering it needs is already pinned by
  a **true forward edge** — `dc:deep/forcing` reads `Climate`, and every terrain writer in the
  roster is downstream of `forcing` — so climate is provably ahead of all of them by the graph,
  not by the tie-break. Declaring it is **free and provably cannot move the schedule** (the edge
  is already implied by the existing transitive closure), but it needs three cfg-selected slices
  for the three terrain-revision rosters (`Settled` / `Compensated` / `Diffused`). Worth doing
  the next time that file is open; not worth a slice of its own.

- **🔴 THE SHIPPED WORLD HAS ZERO COAL — and the guard that should have caught it runs on a
  different world** (measured 2026-07-25, `examples/coal_walk_tour.rs`; **corrections #51**).
  **USER CALL REQUIRED — this is a world-content question, not a bug to quietly fix.**
  - **The fact.** On the world `dc-client` boots (`BENCH_SEED = 1337`, `Extent::Medium`):
    **0 coal units across all 297,025 deep cells**, against **27,134 peat units in 14,596 cells**.
    The hottest coalification candidate is **15.4 °C** against `COAL_ONSET_C = 22 °C` — **6.6 °C
    short**. Trial-onset curve on this world is a cliff: `4 °C → 87 %`, `8 °C → 31 %`,
    `12 °C → 9 %`, **`16 °C+ → 0 %`**.
  - **The instrument is proven, so the zero is real.** The same census over `0x0D5EED572026 /
    Medium` — the world journal/0093's numbers came from — finds **1182 coal cells / 1834 runs**.
    It sees coal when coal exists.
  - ~~**Why the guard missed it (the root defect).** `tests/geotherm.rs::production_field()` builds
    **`seed 0x0B0A_57EE_0059, Extent::Small`** — *neither the production seed nor the production
    extent*.~~ **✅ FIXED 2026-07-25 (journal/0106)** — see the closing bullet.
  - **The geotherm's physics is NOT at fault.** On a world with coal it followed the warm crust
    exactly as claimed (coal cells mean gradient 41.9 °C/km vs peat-only 31.3; rift/arc ≥40 →
    13.4 % coal, craton <20 → 0 %). **Seed 1337 has no warm crust with peat on it.**
  - **OPTIONS (user's):** **(a)** accept a coal-free world for now — coal is a blessed placeholder
    and biology will recalibrate it anyway; **(b)** lower `COAL_ONSET_C` toward the cliff's live
    range (`≈8–12 °C` gives 31 %/9 % on this world) as an interim seat; **(c)** treat it as
    evidence that a single global onset temperature is the wrong shape and let the
    genesis-passes/property-driven arc subsume it; **(d)** change the shipped seed — **rejected by
    the integrator as backwards**, tuning the world to fit a constant.
  - **✅ DONE 2026-07-25 (journal/0106) — the half that was not a content question: `production_field()`
    is now the shipped world.** `tests/geotherm.rs::production_field()` builds **seed 1337 at
    `Extent::Medium`** (memoized per test binary), and the coal guard is **split in two**:
    - `the_geotherm_rule_governs_coalification_on_the_production_world` — on 1337/Medium. Asserts the
      `temperature` field is populated, that candidates exist (26 845 of them), and that **every
      candidate's coal state agrees unit-for-unit with `T ≥ COAL_ONSET_C`** — "coalification responds
      to the gradient field" in falsifiable form. It **requires no coal**, deliberately: the zero is
      an open content question (a)–(d) below, and a guard must not be a hostage to it. It reprints the
      onset sensitivity curve (`4 °C → 87 %` … `16 °C → 0 %`) every run, which is corrections #51
      lesson 3 made permanent.
    - `coal_follows_the_warm_crust_on_the_warm_reference_world` — on `warm_reference_field()`
      (`0x0D5EED572026`, Medium), **named for what it is**. Asserts coal exists, is not degenerate, and
      that coal units sit on **hotter crust** than the peat that stayed peat (measured 42.8 vs
      31.7 °C/km). A non-production fixture is fine; a non-production fixture called production is not.
    - `COAL_ONSET_C` untouched. **All three tests pass** (63 s; the Medium runs cost ~55 s more than the
      old Small ones — the price of the guard being about the shipped world).
    - Audit of siblings: `providers_common`/`rh_unification`'s `production_*` helpers name the same
      non-shipped world, but their claims (golden byte-identity, derived-vs-scalar agreement) are
      genuinely seed-independent, so they are **annotated, not re-seeded**; the `golden_*` rename ripples
      into `providers_golden.rs` + comments in `flux_record.rs`/`head_field.rs` and is left sequenced —
      **and as of 2026-07-25 it really was** (sweep row D-2 caught that the word "sequenced" was doing
      the work of an entry that did not exist — the same doctrine gap that hid row S-7 below).
      **✅ The rename SHIPPED 2026-07-26 — see `ROADMAP-history.md` § Shipped**; the helpers are `golden_field` /
      `golden_pregen` and the golden test is `the_golden_world_still_hashes_to_the_pre_slice_goldens`.
      `s18_first_behavior_weathering::production_scale_saprolite_band_reaches_at_least_one_voxel` is
      **honest** (1337/Medium) and is the shape to copy. `deeptime::production_config` /
      `water::coarse::production()` name a *config*, not a world — legitimate.

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

- **⚠ A TIE-BREAK IS DECIDING PHYSICS AGAIN — `reads_prev` is documentation, not a
  mechanism** (spine-audit 2026-07-25; **the SECOND instance in two sweeps**, and the
  auditor's own words: *"this is the only fix that stops a third"*). **USER RATIFICATION
  REQUIRED — not an agent's call.**
  - ~~**The hole.**~~ **✅ DONE 2026-07-25 — option (a) shipped, journal/0104.** `passgraph`
    now takes a second edge kind: a `reads_prev` declaration is a reader-before-writer
    **anti-dependency**, so a lagged read is *ordered*, not annotated. **Production order
    byte-unmoved** (all six new edges pointed from a pass already ahead of its target — the
    physics was right and merely unenforced), goldens green by name, and the guarantee is
    `a_lagged_reader_stays_ahead_of_its_writer_under_a_hostile_rename`. Full detail in the
    Shipped entry. *(The paragraph below is the original diagnosis, kept for the record.)*
  - **The hole (as diagnosed).** `DeepPass::reads_prev` appears in `runner.rs` and **in no
    other file in `crates/`** — `passgraph` never receives it (`runner.rs:199-204`,
    deliberately). So a
    `reads_prev` declaration is **enforced by nothing**. `dc:field/head` declares
    `reads_prev:[Recorded]` and that is *true today* only because Kahn parks a ready node
    until it wins the id sort (`passgraph.rs:151-153`): head is ready once `drainage` writes
    `Routed`, while `deposition` waits on `isostasy`. **Rename the pass to any id sorting
    after `dc:deep/deposition` — `dc:deep/hydraulic_head`, say — and it silently starts
    reading THIS epoch's record.**
  - **Why this is the same defect one level up.** The last sweep caught a tie-break deciding
    a *read* (`weather_inventory`'s `BioMod`, fixed by declaring it). This one has a
    tie-break deciding **an epoch**, and it **cannot** be fixed the same way, because there
    is no declaration channel that binds. The honest fix is a **mechanism** — a
    reader-before-writer **anti-dependency edge** in `passgraph`, so a lagged read is
    ordered, not merely annotated.
  - ~~**⏳ STILL OPEN — second, smaller, and fixable today: `dc:field/head` UNDER-DECLARES.**~~
    **✅ DONE 2026-07-25 — journal/0107, user-ratified with the consequence attached.** The
    revision is **`Forced`** in every cfg path, and declaring it turned out to be **only half
    the fix**: a `reads` edge on a revision token pins you after its *producer* and says nothing
    about the pass that overwrites the same plane next (a different token = a different
    resource). So `dc:deep/transport`'s undeclared terrain mutation got its name
    (`DeepAxis::Incised`) and `head` declares the **pair** — `reads: [Routed, Forced]` +
    `reads_prev: [Recorded, Incised]`. **The auditor's *"declaring `Forced` is free, and it
    pins it"* was half wrong, and the *"verify that claim before trusting it"* below is what
    caught it.** Flux record byte-identical to journal/0098 on every figure; pass order
    unmoved; `dc:deep/climate`'s lag declared in the same slice. Full detail in
    `ROADMAP-history.md` § Shipped, journal/0107. *(The paragraph below is the original diagnosis, kept for the record.)*
  - **The hole (as diagnosed).** `reads:[Routed]`
    (`runner.rs:605`) does not cover the ground surface `R+H`, which the body builds via
    `grid.surf_at` (`runner.rs:428-430`) and the solve uses as its **seepage cap, lake datum
    and free-surface boundary** (`head.rs:434-467`). Which terrain revision it sees is again
    the id tie-break. The golden-order test's defence — *"its position never affects the
    terrain"* — is true and **is not the question**: it affects **the field's own values**.
    The auditor judges declaring `Forced` *"free, and it pins it"*. **Verify that claim
    before trusting it** — head is **ON by default** (`grid.rs:279`), so if the declaration
    moves which revision it reads, the **recorded flux changes** and that is a world change
    needing a walk. Acceptance: declare it, prove the flux record byte-identical; **if it
    moves, STOP.**

- **`DeepField::chapters` — built, exported, and called by nothing** (spine-audit
  2026-07-25; `field.rs:367`, populated `:421`). Its own doc comment has said *"today the
  table is exported and read by nothing"* since U8; readers are only tests. **Now listed in
  spines § 3, where it belongs.** The lesson is the reason it is here: **three consecutive
  sweeps added rows for its two immediate neighbours in the same struct and walked past
  it** — *a self-declaring comment is not an index*, demonstrated against § 3 itself.

- **OWED / next residency lever — a per-cell OWNING CONTAINER is a header × 297,025 before it
  stores anything** (journal/0100, 2026-07-25). The CSR conversion cut the ledger heap 9.5×, but
  the per-cell `FactLedger` **struct** grew 24 → 48 B (two `Vec` headers per cell) = **13.60 MiB
  paid whether or not a cell has a single fact** — an honest regression the slice flagged itself.
  The full `flux.rs` shape collapses it: **one record for the whole grid, with the cell as a CSR
  row**. Deferred only because it moves `DeepField::ledgers` and `ledger_at_voxel`, which sat in a
  live sibling's write-set. **The generalisation is the valuable part and applies far beyond this
  struct:** *any per-cell owning container in a 297 k-cell field costs a header per cell before it
  holds data* — so the default for anything per-cell is **one grid-wide record with CSR rows**,
  never `Vec<Something>` per cell. Same family as the `Vec<Vec<Fact>>` defect, one level up.
  **✅ DONE 2026-07-25 (journal/0102): struct overhead 13.60 MiB → 0, per-cell index 48 B → 4 B,
  flag-ON `DeepField` 186.07 → 173.61 MiB, flag-OFF identical to the byte, 1,033,189 facts across
  72,006 slots before and after, and no reader changed (`ledger_at_voxel` returns a borrowed
  `LedgerView<'_>`).**
  **⚠ MY GENERALISATION ABOVE WAS TOO BROAD — corrected by the build.** The defect is **not**
  "a per-cell owning container"; it is **one that is RESIDENT**. `FactLedger` **survives as the
  gen-time accumulator**, and must: a grid-wide insert would memmove every fact after the cell,
  every epoch. **The split is by CLOCK, not by shape** — grow per-cell while compiling (gen time
  is free), compact to one grid-wide record for residency (runtime is sacred). Read the rule that
  way, or it forbids the very structure the compile needs. *(Related expiry, also from the build:
  journal/0100's carve-out — "dense offsets are unaffordable for the ledger" — **expires at the
  CELL level**, because every cell exists even though every slot does not. The record now carries
  **both** compressions, each where its premise holds: dense cell offsets + keyed slot rows.)*
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

- ✅ **DONE 2026-07-25 — shipped, see `ROADMAP-history.md` § Shipped (journal/0102).** *a per-cell OWNING CONTAINER is a
  header × 297,025 before it stores anything* (filed by journal/0100 against itself). Measured
  result: the struct-overhead line **13.60 MiB → 0 B**, per-cell index cost **48 B → 4 B (12×)**,
  flag-ON `DeepField` **186.07 → 173.61 MiB**, cost of the flag **+29.91 → +17.45 MiB**; world
  byte-identical (1,033,189 facts in 72,006 slots, before and after; flag-OFF baseline the same
  integer, 163,748,661 B). **The generalisation survives the slice and is the valuable part:**
  *any per-cell owning container in a 297 k-cell field costs a header per cell before it holds
  data* — so the default for anything per-cell is **one grid-wide record with CSR rows**, never
  `Vec<Something>` per cell. Refined by the slice: the defect is a per-cell owning container that
  is **RESIDENT**; per-cell is the right *gen-time* shape (a grid-wide insert would memmove every
  fact after the cell, every epoch), so compact at the seam where the compile ends — which already
  exists in this codebase and is called `finalize_*`.

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
  - **CROSS-REF added 2026-07-25 (sweep row A-1): this is now on the WEATHERING-IS-ONE-PROCESS
    arc's critical path.** That arc's requisite R3 named **residency, not gen time**, as its
    blocker (`LedgerField` 17.45 MiB → **973 MiB**, 55.8×, resident), so the strata collapse is the
    next lever on **the same budget the weathering arc has to fit inside**. Read the two together;
    freeing 9 MiB here is not decisive against 973 MiB, but the two entries are competing for one
    number and neither should be planned alone.

- **✅ DIAGNOSED 2026-07-25 (journal/0103) — the front's voxel-tier mass error is NOISE, and the
  `+3.7 %` was the INSTRUMENT** (opened by the integrator's review of journal/0099; corrections
  #50). Re-run over **247 columns** (journal/0099 had 21), with the pipeline split into its **two
  quantizers** — the fill geometry and the eighth draw are not the same kind of error and only the
  second is an estimator — and with the census taught to exclude voxels it cannot attribute.
  - **INTEGRATOR'S HYPOTHESIS WAS WRONG, and it is worth recording which half.** I challenged
    journal/0099's *"unbiased estimator"* claim on the grounds that a `+3.7 %` mean and a `+16 %`
    median were **the signature of a floor effect** (a band thinner than one eighth cannot express
    as less than one eighth without vanishing). **The challenge was right and the hypothesis was
    wrong.** The stratification really does trace a textbook floor-effect curve — but the cause is
    **measurement contamination, not quantization**: the product is `CLASS_CLASTIC_FINE` and so is
    much of the pile directly above it, **222 of 247 columns** have a front voxel whose plan also
    holds a *non-front* event of the same material, and **voxel contents carry no provenance**, so
    the naive census credited overlying mudstone to the front. The decay with magnitude was *the
    contact voxel's share of the front shrinking as the front grows.* **journal/0055's doctrine is
    CONFIRMED, not falsified** — `allocate_to` is Cranley–Patterson systematic sampling with
    `P(extra) = remainder` exactly, and the one real floor (`clamp(1,7)`) cannot bite on a
    `[7,5,4,3,2,1,1,1]` profile. corrections #50 falsifies **the number, not the doctrine**.
  - **Second time today that asking the discriminating question mattered more than the hypothesis
    attached to it** (corrections #49 was the first — *"was it air, or `has_contents:false` read as
    air?"* offered two wrong answers and the truth was a third thing). Both times the *question*
    forced the measurement that produced the real answer. **Demand the measurement; hold the
    explanation loosely.**
  - **Stage 1, record → fill geometry: aggregate −0.02 %, mean +0.03 %, median +0.00 %, p5 −1.22 %,
    p95 +0.86 %.** The record-bottom round-to-nearest, which lands on the front *every time*
    because the front is what sits at the record's bottom, is centred.
  - **Stage 2, the draw over attributable voxels: aggregate −0.60 %, mean −0.10 %, median −0.00 %,
    p5 −30.00 %, p95 +28.72 %** (Mixed-plan voxels only, the only ones carrying a draw: −0.84 %).
    Symmetric about zero, spread **widening** as the front thins — which is what an unbiased
    estimator over a one-eighth quantum does, not a floor.
  - **Stratified by front magnitude the fake trend disappears.** Naive means run +76 % (2–4
    eighths) → +31 % (4–8) → +11 % (8–24) → +4 % (≥24), a textbook floor-effect curve. Stage-2
    means run +9.1 % → +2.9 % → −0.1 % → −0.7 %, no trend. The naive decay was **the contact
    voxel's share of the front shrinking as the front grows**, not quantization.
  - **The mechanism.** The product is `CLASS_CLASTIC_FINE` (mudstone) and so is much of the pile
    directly above it; at the top contact they share a `Mixed` voxel and **voxel contents carry no
    provenance**. **222 of 247** columns have a front voxel whose plan holds a non-front event made
    of the product's own material, so a material census credited the neighbour's mudstone to the
    front.
  - **From the code** (asked for separately, and it holds independently of the data):
    `fill::allocate_to` is systematic sampling / Cranley–Patterson with `P(extra) = remainder`
    exactly; `allocate_partial` floors the **cumulative**, so errors cancel along the run;
    `pore_rider_share`'s mean is exactly `cnt·k8/8`. **Genuinely stochastic-proportional; nothing
    rounds up at the floor.** journal/0055's doctrine is **confirmed, not falsified**.
  - **flow.md § 3's mass budget can build on this** — with one requirement that is the real
    deliverable: *a conservation audit at the voxel tier must compare against the **fill plan**,
    never against a census of the finished contents.* The plan knows which event owns which
    fraction of which voxel; the voxel does not.
  - **NOT fixed, and nothing to fix** — the expression is unbiased. **Residual, stated:** 222 of
    2 077 front voxels (10.7 %) are unattributable and are systematically the *top contact*, not a
    random tenth; this instrument cannot weigh them. The verdict rests on stage 1 covering 100 % of
    voxels and on the code-level proof. **Loose end filed:** `pore_rider_share`'s comment claims its
    offset is disjoint from `allocate_partial`'s ("a low digit… not its high bits"); the bits
    overlap (bits 10–12 vs the top 20), so the two draws are correlated. Not a mass defect — each
    is marginally unbiased and the measurement above is the joint case — but the comment is wrong
    and changing the address would move every contact voxel in the world, so it is **the user's
    call**.

- **`pore_rider_share`'s offset is NOT disjoint from `allocate_partial`'s, but its comment says it
  is** (found 2026-07-25 while diagnosing the mass claim, journal/0103; **not a mass defect**).
  `collapse.rs`'s `pore_rider_share` documents its draw as *"a **low digit** of the voxel's own fill
  draw, not its high bits: `allocate_partial` consumes the high end, and reusing it here would
  correlate 'this band won an extra eighth' with 'the product won an extra eighth of it' into a
  visible pattern."* The arithmetic does not deliver that: `(u * 4096.0) as u64 & 7` is **bits
  10–12** of the fraction, and `allocate_to`'s `uq = (u * ONE) as u64` is the **top 20**. They
  overlap, so `cnt` and the rider's offset **are** correlated — exactly the coupling the comment
  says it avoided. **Measured consequence on mass: none detectable** (journal/0103's stage-2 figure
  is the joint case: −0.60 % aggregate, median −0.00 % over 247 columns), because each draw is
  marginally unbiased. The open question is the one the comment actually cared about: whether the
  correlation is **visible** as a pattern at a contact.
  - **RESOLVED 2026-07-25 — decorrelated, and the visibility question answered NO** (journal/0105;
    user-ratified *"this needs fixed either way. Decorrelate."*). Quantified before the fix: the
    pore offset was not merely correlated with the allocation's offset, it was a **deterministic
    function** of it — 100.00 % predictable from bits 8–10 of the 20-bit offset, over 464,521 real
    decisions. But the coupling the comment feared measured **zero**: `r = +0.0006` between the two
    roundings' residuals, mutual information on the estimator floor, and the dither's entropy given
    everything the allocation decided still **2.999 of 3.000 bits** — bits 8–10 are a fast sawtooth
    across the allocation's contiguous decision interval, so they alias to uniform. **No banding,
    measured spatially**: on a 32×32 contact plane sharing one record and one fill plan, every
    autocorrelation at lags 1–4 is inside ±0.07 before *and* after. **The defect that was real was
    a different one**: `u` was drawn once per voxel and served **every** band in it, so a
    multi-band front voxel's riders rounded in lockstep (`r = +0.4878` over 187,701 sibling pairs)
    and their errors **added** — 1.488× the second moment independent roundings give. Fixed by
    `SALT_GEO_PORE` + the event index + a `PoreDraw` newtype the fill draw cannot be passed to.
    See `ROADMAP-history.md` § Shipped, journal/0105.

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
- **✅ FIXED 2026-07-25 (journal/0101) — the query can now say `UNRECORDED`.** `identify(pos)`
  landed as the arc's first slice: `has_contents` is a **per-voxel** fact, `classified` echoes
  the stored block for an unrecorded voxel instead of naming a mixture that does not exist,
  `character_sense_raycast` answers `None` (its documented promise) instead of `Some(<empty
  view>)`, and the F3 HUD's `(no contents record here)` branch is **reachable**. Census through
  the real query path (`dc-client/examples/identify_census.rs`, same lattice as the diagnosis
  probe): phantom air **702 → 0** of 10 985 solid voxels (6.4 % → 0.0 %), 39/169 columns →
  0; every phantom voxel converted to `UNRECORDED` (4 913 = 702 + the 4 211 already honest),
  recorded mixtures (6 072) and sky unmoved. At journal/0097's own station the band
  288–299 now reads `UNRECORDED` and **agrees with 287**, which the chunk floor used to
  split. **Zero
  dc-worldgen change** — the stored `Block` already disambiguates. *Entry kept, marked, because
  the numbers below are the measured baseline.*

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

- **✅ DONE 2026-07-25 — FREE WIN TAKEN: 54.02 MiB reclaimed, 33.2 % of all `DeepField`
  residency, zero behaviour change.** `build_field` now `shrink_to_fit`s every cell's
  `units` Vec at the end of the compile. **Measured before → after: 162.57 MiB → 108.55
  MiB**, slack 54.02 MiB → **0.00 MiB (0 %)**, `DeepField::resident_bytes` agreeing;
  asserted by `strata_is_shrunk_to_fit_after_the_compile` (capacity == len for every
  cell, so the reclaim is enforced, not merely intended). The one-time copy is gen-time,
  therefore free. *Note: `docs/spikes/S19-*` quotes the pre-fix 162.57 MiB baseline —
  that figure is now historical; today's total is 108.55 MiB, which is the correct
  denominator for the flow-record projections (every `× today` multiple in S19 is
  correspondingly ~1.5× larger against the new baseline).* The sibling defect — the
  `FactLedger` `Vec<Vec<Fact>>` at **98.8 % empty inner Vecs / 89 % of its heap in empty
  headers (+156.91 MiB when `weather_inventory` is on)** — is NOT fixed by this and
  remains the reason the flow record must be sparse/CSR.

  *Original entry:* **`strata` Vec capacity doubling wastes 54.02 MiB: 39 % of the record heap
  and 33 % of ALL current `DeepField` residency** (measured 2026-07-25,
  `docs/spikes/S19-flow-record-cost-results.md`). Live 84.47 MiB vs capacity 138.49 MiB
  on a production world (seed 1337, Medium). Pure allocator slack from growth doubling —
  the record is built once per world and then read-only, so a `shrink_to_fit` (or an
  exact-size second pass / arena) at the end of the deep-time compile should recover most
  of it with **no behaviour change and no content cost**. This is the cheapest residency
  win on the board and it is worth taking before the flow record adds a second large
  store. **Runtime perf is first-class and this is pure waste** — but measure the actual
  reclaim (and the one-time copy cost, which is gen-time, therefore free) rather than
  assuming. *Related, same measurement:* `FactLedger`'s `Vec<Vec<Fact>>` is **98.8 % empty
  inner Vecs, 89 % of its heap empty headers** (+156.91 MiB when `weather_inventory` is
  on) — the same allocation-shape defect one level up, and the reason the flow record must
  be sparse/CSR rather than per-(cell,slot) Vecs.

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

- **Two declaration defects on the new `dc:deep/weather_inventory` pass** (spine-audit
  2026-07-24; both free to fix, neither affects production — the flag is off by default).
  **(a) `reads_prev: &[BioMod]` is not what happens.** The pass reads `grid.bio_weather`, which
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

- **Dev slice — look-at-voxel contents inspector — SHIPPED 2026-07-24**
  (`f594580`, journal/0088; `world_get_contents` + `character_sense_raycast` contents
  + F3 HUD; `ContentsSource` seam re-derives contents since the runtime stores only
  `Block`). *(Original ask, for the record:)* (user-requested 2026-07-24;
  enabling the palette-quantization measurement above AND a standing dev tool):
  *"check the contents of a voxel just by looking at it."* Two parts: (a) a
  **full-contents query** returning a voxel's whole `VoxelContents` mixture —
  materials, forms, weights — NOT the classified `Block` (`get_block` returns only
  the winner, which is exactly what hides T1-vs-T2); (b) a **look-at readout** —
  `character_sense_raycast` already returns the hit voxel + face but only the block
  name, so extend it to contents and surface it as an on-screen HUD. Small (dc-api
  query + dc-client HUD/raycast), dual-use: it makes the T1/T2 measurement doable
  live and gives every future material diagnosis a direct instrument. **Sequence it
  ahead of the palette-quantization diagnosis — it is that diagnosis's instrument.**

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

- *(**Deep-cell-square surface-material frontiers checker the far field:
  RESOLVED** 2026-07-22, journal/0073 — the B1 shape-teacher. `surface_class`
  now dithers class membership from the top-window metre shares (S-4 move B), so
  the 460 m class frontier is a statistical gradient, not a razor-straight
  square. The record stays a non-interpolable unit list read NEAREST — the fix
  routed *around* interpolation exactly as spines S-4 flagged, by dithering
  membership rather than bilinear-ing the record. Re-shot at the `0070-*`
  vantages as `0073-*`; verdict in the entry.)*

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
  **UNWALKED** — nobody has seen the holes gone; re-walk
  `0056-holes-after-settle.png`'s coordinates, and if bands persist, distrust
  journal/0057 first.)*
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
- *(**"The world systematically under-expresses `H`": RETRACTED** 2026-07-21,
  same day it was filed — corrections #28, journal/0058. The alarm was the
  integrator's metres/voxels inversion, not a defect. Verified headlessly:
  the generated column matches `round(H/0.9)` with an error of **0 or +1
  everywhere, never negative**, the +1 being the top-of-column partial voxel
  that a *block* scan must count whole. The client world is also byte-identical
  to the probe world, proven block-for-block at four addresses — the
  stop-the-world hypothesis is dead.)*
- *(**"Surface material is quantized per chunk": FIXED** 2026-07-21,
  journal/0058 — the member is now drawn per voxel column inside the shared
  `surface_sample` kernel. Chunk footprints expressing more than one surface
  member went **0/169 → 147/169**. Block fingerprints unchanged in both
  recorded worlds, which is within-class invariance confirmed by an 80-chunk
  fingerprint that knows nothing about the argument. **Unwalked** — the fix
  landed after the user's session closed, so nobody has seen the patches
  gone.)*
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
- **~~Caves ↔ hydrology integration thread captured~~ — SUBSUMED 2026-07-25 by the FLOW arc**
  (sweep row B-1). All four of its work-shaped findings now have owners, and this entry is kept
  only as the pointer:
  - **two drainage opinions** → `flow.md` § 6 retires `pregen/hydrology.rs` **outright** —
    *"wrong resolution, wrong time, wrong topology"* — so there is no subsumption to design;
  - **"deep drainage is computed and discarded"**, wanting a per-chapter table recorder axis for
    erosional caves → **that is `DeepField::flux`, and it shipped** (journal/0096: 2,590,372
    entries, per chapter, 40.66 MiB, pinned by
    `per_chapter_history_is_retained_not_just_the_final_epoch`);
  - the **bounded-drainage-refinement spike question** that gated RiverSeg retirement → **S14 is
    superseded as posed** (flow.md § 9.6; water.md's own banner), and RiverSeg retirement is FLOW
    **continuation (e)**;
  - the **column-as-interval-log target contract** → flow.md § 7 promotes it from *proposed* to
    **necessary** (voids and conduits are intervals, not a heightfield).
  **Cave-specific residue rides FLOW continuation (c)** (the free/bound edge, void intervals, and
  the conduit pairing rule). *(Original capture, for the reasoning: full text in water.md § Session
  capture 2026-07-21 — nothing was decided there. Same session recorded the user's
  water-rendering directive (partials/structure, placeholder texture, data
  seams for flow/waves) in water.md, which is NOT subsumed and still stands.)*
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
- **UPDATE 2026-07-21 (second occurrence): the teleport-storm hypothesis is
  FALSIFIED as the trigger.** A `--horizon 6` session crashed with the
  IDENTICAL signature (DeviceLost 10:20:59 → buffer-map panic → cluster
  PoisonError) while **completely idle** — booted, streamed its field, sat
  untouched ~4.5 min, died. Two for two at `--horizon 6` (~10 min with
  activity, ~4.5 min idle); default-horizon sessions historically run long.
  Revised suspicion: resource/VRAM accumulation in the wide-horizon far-field
  path (or a driver interaction it provokes) — a leak-shaped bug, not a
  burst-load bug. Repro is now cheap: boot `--horizon 6`, wait five minutes.
  Diagnosis slice should instrument GPU memory over idle time. Until fixed,
  walks run `--horizon 3` (stations are close-range reads; only skyline
  vistas need 6+). **THIRD SYMPTOM (journal/0049 tour): texture SMEARING
  visible to the human eye after heavy teleporting at `--horizon 3`** —
  user field report, live session. The leak is not horizon-6-exclusive,
  just slower; smearing may be the pre-crash state. Strengthens the
  resource-accumulation hypothesis; the diagnosis slice should reproduce
  via teleport churn while instrumenting GPU memory AND watching for
  texture degradation as the early warning. **Lifetime bound (same day):
  the smearing `--horizon 3` session ran ~44 min through the whole tour and
  exited CLEANLY (verified: no DeviceLost in the log), vs 4.5–10 min to
  death at `--horizon 6` — accumulation scales with far-field size, and
  smearing is the degraded-but-alive state well before the cliff.**
- **CONFIRMED AT HORIZON 6, 2026-07-21 (journal/0065): the fix holds at the
  width that used to kill it, and the residual is horizon-independent.**
  Storm and idle regimes, alternated 6/3/6/3. Storm slope +0.542/+0.529
  MB/jump at 6 vs +0.531/+0.542 at 3 (200 jumps each, RSS high-water 1.10 GB,
  `host_chunks` 2 108–2 332 against `host_budget=2360`, ~19 000 evictions per
  run). Idle at 6: every probe count frozen, RSS drift < 7 MB in 7 min, at the
  full 704-tile far field. All exits **0**, no `DeviceLost`/panic/`ERROR`.
  Two findings ride along: (a) **the teleport storm is nearly blind to
  `--horizon`** — the field never fills under motion (`far_tiles` oscillates
  2–40 at *either* width), which is the mechanism behind 0050's unexplained
  "h3 and h6 slopes are near-identical", so idle is the only regime where the
  horizon is a real variable; (b) **`Authority::chunk_budget_for` does not
  scale with the horizon** — it derives from the *near*-field constant
  `UNLOAD_RADIUS_M`, resolving to 2 360 at every width (correct, but the
  opposite of what the brief assumed).
- **RESOLVED IN THE SAME ENTRY: the "+0.54 MB/jump residual" was a windowing
  artifact.** A single **700-jump / 23-minute** horizon-6 session shows RSS
  **sawtoothing** in a ~980–1 170 MB band — climb ~250 jumps, drop 120–180 MB,
  repeat. Full-run slope **+0.113 MB/jump** (jumps 50–700), **+0.074** over the
  last 300; every 200-jump window sat inside one tooth and read ~+0.54.
  0051's 0.00 and 0065's 0.54 are the same oscillation at different phases.
  Methodological rule now: **measure ≥ 250 jumps or you are measuring a
  tooth.** Final state 71 928 evictions, `host_chunks=2224/2360`, high-water
  1 166 MB, exit 0. The ~0.1 MB/jump that survives is plausibly allocator
  hysteresis; if anyone chases it, 0050's open gap is the candidate (the
  collapse caches' `coarse_surface` / `column_record` paths never trigger
  `evict()` — position-keyed and horizon-independent, exactly this shape).
- **FIXED 2026-07-21 (journal/0051): eviction landed, the march is flat
  (+27.4 → 0.00 MB/jump). See `ROADMAP-history.md` § Shipped. Two numbers from this diagnosis were
  corrected on the way: a chunk is 64 KB, not ~33 KB (`Block` is `repr(u16)`),
  so the fill rate is ~335 chunks/jump, not ~750.** Original diagnosis below.
- **DIAGNOSED 2026-07-21 (journal/0050): the leak is host-RAM, not the GPU
  and not the far-field pooling.** Instrumented our own allocation counts
  (`DC_MEM_PROBE` plugin, merged) beside external RSS + VRAM sampling.
  Findings, all measured: (1) **idle `--horizon 6` does NOT leak** — every
  count flat, RSS ~978 MB, VRAM ~3158 MB, survived 6.5 min (the "idle 4.5 min
  death" did not reproduce; the leak is motion-driven, not time-driven).
  (2) A **teleport storm to fresh distant coords leaks RSS linearly, ~25 MB/
  jump (~13 MB/s), unbounded**, while VRAM stays flat AND our render counts
  (meshes/entities/far_tiles) only oscillate, never grow — so the far-field
  pooling and wgpu/VRAM are NOT the site (pooling-doctrine failure-class
  hypothesis FALSIFIED). (3) The decisive cut: the same storm cycling **four
  FIXED coords keeps RSS flat within 4 MB for 142 jumps** — the leak is keyed
  by world *position*, not by render churn. Mechanism (as corrected at
  integration — the draft's "two unbounded caches" died against
  `collapse.rs`): **`HostWorld.chunks` is the one genuinely unbounded store**
  (`dc-api/src/host.rs:178`, materialized `chunk_at` `:367–373`: every chunk
  any query touches, retained forever, ~33 KB each). The `WorldGenerator`
  collapse caches are **bounded** by `evict()` caps (`collapse.rs:609–624`,
  fired per `generate_chunk`) and contribute steady-state footprint, with one
  real gap: far-field-only sampling paths (`coarse_surface`/`column_record`)
  never trigger `evict`. h3 and h6 jump slopes are near-identical (streaming
  budget caps new-world-per-jump); the horizon-scaled *lifetime* the tour saw
  is a steady-state-footprint + continuous-far-field-sweep effect. Fix =
  eviction on `HostWorld.chunks`, which must distinguish generated-untouched
  (droppable — "store only what the derivation cannot predict", the S11
  doctrine) from edited (persist/spill to the save layer); generator-cache
  work is residual (close the coarse_surface gap, distance-aware caps if
  measured). *(**Secondary defect RESOLVED** 2026-07-21, journal/0054 —
  DeviceLost now degrades loudly; the poison cascade is gone at source and
  the exit code no longer lies.)*
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
  substitution with no placeholder marker anywhere; a loud code comment is
  owed at `collapse.rs::ruin_posts` (deferred: collapse.rs is in the climate
  agent's write-set). Two audit additions to the decision's holdout list:
  igneous emplacement-depth constants and paleo-temp-is-present-day. Also
  flagged: the `field.rs` doc-comment claims the collapse tier "reads"
  exhum/t_crust when nothing does — do not trust it.

- *(**Sub-km relief / roughness decay: MEASURED** 2026-07-21, S13 + journal/0041
  — see `ROADMAP-history.md` § Shipped. Decay confirmed (5 % survives), bilinear falsified (#24), and
  the walk's own sampling corrected (#25). The remaining OPEN part is which
  recalibration to take — § Sequenced **above**, awaiting a user picture-pick.)*
- *(**`climate_at` half-cell offset: CONFIRMED BUG, FIXED** 2026-07-21,
  journal/0043 — climate sat exactly 7 372.8 m north-east of the terrain it
  tinted, live at every preset (all odd `w`). One-expression fix,
  regression-guarded at Small+Medium; 7 surface columns flip Stone→Dirt in the
  22–33° band (the rock line was 7.4 km off). Heights are climate-independent,
  so all 0040/S13 elevation numbers stand. No prior conclusion falsified —
  the 0038 desert-null mechanism (corrections #22) is registration-independent.)*
- *(**The far field cuts off at 1.2 km**: FIXED 2026-07-21, journal/0042 —
  `--horizon <km>`. Original report: user, 2026-07-21, "the cutoff is still too
  near, can't see macro shape of landscape".)*
- **The haze curve, not geometry, limits the usable vista** (journal/0042
  measurement, 2026-07-21). With `--horizon 8` the outer third of the field
  washes toward white and silhouette reading works only to ~5–6 km, even though
  the geometry is there and paid for. The fog *range* now scales with the
  horizon; the fog **curve** (its falloff shape) does not, and whether it should
  is a **user-owned visual call** the agent deliberately did not make. Cheap to
  change, needs the user's eye on a before/after.
- *(**Material placement rules are climate mocks: DECIDED 2026-07-21** — see
  geology.md § Expression of the ledger, ecology.md § DECIDED 2026-07-21, and
  ARCHITECTURE.md § Modularity and performance. Runtime gen is refinement over
  the ledger; everything recorded must be expressed; unexpressed only where the
  expresser is unbuilt, loudly temporary. The veneer rule is a placeholder —
  DO NOT BANDAID. Remaining OPEN engineering, now Sequenced: consume `exhum`/
  `t_crust`, carry the discarded `H` regolith plane, derive form from
  provenance, and the per-voxel provenance query. Original observation:)*
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
- *(**Console v1 field report: FIXED same day** — console v2 shipped, see
  `ROADMAP-history.md` § Shipped / journal 0035. Original report:)* **"still unusable" (user,
  2026-07-20, first test drive).** Two defects, both discoverability-of-what-exists rather
  than missing data: (a) the arg surface is invisible in practice — no
  per-arg help while typing, no visible arg shapes/expected inputs, so a
  user cannot form a valid command without already knowing it; (b) the
  output pane cannot scroll. DIAGNOSED at dispatch (same day): v0 rendered
  help only on explicit `help <cmd>` and completion stopped at param keys;
  the fix is presentational (inline signature + per-arg hints from the
  schemas already carried) plus wiring the merged `Completer` hook
  (decision #6) for value completion. Console-v2 agent dispatched — see In
  flight.

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

- *(**INSTRUMENT: "lit before/after is invalid because the sun moves" —
  RETRACTED same day**, corrections #18/#19. The sun is FIXED (S4: constant
  0.35 time-of-day); no day/night cycle exists, which is exactly why the
  sim-light design had to invent heavenly-body paths. The lit pass was the
  trustworthy register all along. The real defect was the opposite one, and
  is recorded below.)*

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

- **Loose materials do not exist in the world yet** (user, 2026-07-20:
  "needed — even if they don't fall with gravity yet"). The RENDERER is
  already waiting: partial-height loose rendering shipped built-but-dormant
  in 3c-2 (journal/0010) because loose deposition never emits sub-8
  columns. Missing half is content/simulation. User direction for when it
  lands: loose materials should **spread on being dropped** (granularity +
  fall height → partials displaced into surrounding empties) — angle of
  repose from the partials model rather than a physics solver. Sketch in
  ideas.md.


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

- **Walk report (2026-07-20, journal/0027): coal renders as pure black in the
  lit pass — a hole in the screen, not a rock.**
  **⚠ STILL A VALID LIGHTING/TONEMAP QUESTION, BUT UN-WALKABLE ON THE SHIPPED WORLD**
  (2026-07-25, sweep row A-2): there is **no coal to photograph** — 0 units across 297,025 cells
  on seed 1337 / Medium (corrections #51). The 2026-07-20 frames were shot on a world the client
  can still open only because coal existed then; today a re-shoot would find nothing. **Do not
  launch for it.** The defect is about the dark end of the lit path, not about coal, so it can be
  re-photographed on any sufficiently dark material — or it waits on the coal-content call
  (a)–(d). Photographed at world voxel
  (-76133, -80221) on the client's world: an 18-voxel seam four voxels under
  turf, cut to an open bench under full sky. `0027-coal-seam-cut-lit.png` shows
  grass / mudstone / carbonaceous mudstone and then black for the lower
  two-thirds of the frame — including the **bench floor**, which is an up-facing
  sunlit surface, so this is not shadowing. **The fullbright control
  (`0027-coal-seam-cut-fullbright.png`) shows coal as an ordinary mid-dark grey**,
  so the block, the atlas and the 29-layer palette are all correct. Mechanism:
  `meshing.rs` gives `Block::Coal` a vertex colour of `[0.07, 0.065, 0.06]` —
  7 % linear, ≈ 0.29 sRGB, which is exactly what fullbright draws. The lit path
  multiplies that by the directional term and tonemaps, and 7 % albedo has
  nowhere to go but zero. **The number is physically right** (real coal is
  0.04–0.08) — the defect is that the lit path has **no floor under the dark
  end**, so a correct dark material becomes an absence of image. **Do not fix by
  brightening coal.** This is a lighting/tonemap question (an ambient/sky floor,
  or a tonemap that preserves shadow separation), and it belongs with PBR-2's
  shadow work. Second-order finding, **CORRECTED by the user 2026-07-20** — the
  walk's "excavation interiors receive no light" reading (and the integrator's
  "the underground is unlookable-at" amplification of it) was WRONG. The user:
  *"underground is lit by global sun right now, depending which way the face
  faces it has one of six levels of face light... one block face is dark (idk if
  it's N, S, E, or W) whether on the surface or deep in a hole - the others are
  degrees of well lit."* So the real shape of the defect is:
  **(a) ONE face orientation is black everywhere** — on an open plain exactly as
  much as at the bottom of a shaft — because the face pointing away from the
  directional sun has no ambient floor under it; and
  **(b) there is NO DARKNESS UNDERGROUND AT ALL** — depth does not attenuate
  anything, because nothing occludes. `0027-pit-interior-unlit-lit.png` was
  photographing (a), not a property of pits. Being underground is currently
  *lit exactly like being outside*, which is the deeper problem and the one the
  darkness decision (visuals.md) is about.

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

- **User field report (2026-07-20, filed at the FF2a/0024 ratification): a
  razor-straight, kilometer-scale grass/dirt frontier cuts the far field**
  (visible in `0024-after-ne.png` / `0024-fb-ne.png`; present in fullbright,
  so it is surface-block DATA, not a seam or lighting). The user identifies
  this as **the original cause of the walk-8 complaint** — material families
  "appearing to change immediately across some kind of boundary" — now
  legible at full extent because the far field renders the surface rule at
  km scale: "obviously bad / not natural appearing." **DIAGNOSED
  2026-07-20 (read from source, not yet fixed)** — and it is NOT the
  quantization class this entry first guessed. `collapse.rs`
  `surface_sample`: `let bare = riverbed || precip < 0.10 || (fringe &&
  precip < 0.35)` — a **hard binary threshold on a very smooth field**.
  `climate_at` bilinearly interpolates precip between climate-cell centres,
  and a cell is `CELL_VOXELS` = 16 384 voxels ≈ **14.7 km** at N=2, so over
  any near-field view the field is essentially locally linear: its 0.10
  isoline is a geometrically straight line running for kilometres, and the
  threshold gives it **zero transition width**. Two independent defects
  (both must be fixed): (1) *no transition* — grass/dirt needs a
  probabilistic/fractional band around the threshold, not a step; (2) *no
  detail in the boundary itself* — even a soft edge would be a smooth
  km-scale arc, so the isoline wants domain warp / octave noise on precip
  (or on the threshold) to make the frontier wander at 10–100 m scale.
  The 3d member-contact dither is the material-tier precedent; surface
  BLOCK selection has no analogue. **Any fix must live inside
  `surface_sample`**, which near and far provably share (journal/0022), so
  the horizon heals with the ground. Related: walk-8 "material families cut
  hard on chunk lines" — the user identifies THIS as that complaint's
  origin (a different mechanism from the walk-10 per-chunk flow_energy
  rounding, which stays open separately). Couples to the biotic layer: the
  real cure may be that ground cover stops being a paint decision at all
  (see docs/design/ideas.md § the bio slot).

- *(**User field report (2026-07-20, post-FF2a): thin bright seams between far
  patches — RESOLVED** 2026-07-20, journal/0024, fix cycle, background agent;
  worktree branch for the integrator; gates green. Mechanism confirmed
  (corrections #11): the anti-z-fight push translated adjacent same-level tiles
  along their *own* center→viewer directions, differing by the tiles' angular
  separation, so mesh-space-watertight seams reopened at the TRANSFORM stage as
  0.08–0.9 m (L1→L4) world-space slots — see-through once the far field became a
  hollow top-surface sheet (0022/FF2a). Fix: **per-level UNIFORM push** — one
  shared vector per LOD level per frame, along the camera-forward axis, magnitude
  unchanged (`DEPTH_PUSH_FRAC × coarse voxel`). Every tile of a level undergoes
  the identical rigid translation, so shared edges cannot separate *by
  construction*; magnitude differs per level, so overlapping ring pairs still
  separate in the lap band (corrections #1 honored — a true world-space offset,
  never a bias). Applied to BOTH far paths (S1 chunks + FF2a tiles). World-space
  seam proof past the transform: `uniform_push_keeps_same_level_seams_watertight_in_world_space`
  (exact-zero shared-edge gap for a nasty off-axis high vantage; the retired
  radial scheme fails the same check). Walk-verified live (lit + fullbright, high
  vantage −45° yaw sweep): far field continuous, no bright light-through slivers
  anywhere; no z-fight at lap bands or the near/far boundary at grazing or
  top-down angles. Assets `0024-after-{ne,se}`, `0024-fb-ne`,
  `0024-fb-zfight-graze`, `0024-zfight-topdown`.)*

- *(**User field report: the far LOD sheet is buried under the near field —
  RESOLVED** 2026-07-19, FF2a journal/0023. Mechanism confirmed: the walk-17
  one-tile inner lap slid the L1 far sheet under the near field (from ~54 m),
  sunk half a coarse voxel — present but hidden, so digging exposed a phantom
  floor. Fix: **coverage logic, not buried geometry** — a far column the near
  volumetric field covers (`near_covers`, altitude-aware 3-D distance < 112 m)
  is CULLED, so a fully-covered tile meshes to *nothing*. Floor-quantized far
  tops (≤ near surface) let the near field win the thin [112, 128] m occluded
  overlap with no sink. Dig test photographed: only real near geology, no
  phantom floor (`0023-fb-dig-no-phantom-floor`); headless proof: fully-covered
  tile meshes empty.)*

- *(**User field report: clear pixel gaps between far-field tiles — RESOLVED**
  2026-07-19, FF2a journal/0023. The prediction held: voxelization cures the
  crack class **inherently**. Stepped prisms share face planes; a step's side
  face is emitted once by the taller column only, and a tile samples its
  neighbours' shared boundary columns, so same-level tile seams are watertight
  with no skirt (grazing-angle fullbright shot `0023-fb-grazing-horizon`: no
  cracks). Only differing-stride ring-to-ring edges and the near/far coverage
  boundary get a modest 2-coarse-voxel skirt — "skirt only what remains." The
  faint one-sided-normal stitch *lines* (cosmetic, haze-hidden) stay filed
  below, unchanged.)*

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

- Walk 17 (journal/0022 § walk 17 + § the holes were a partition): **far
  sheet parallelogram sky holes — RESOLVED** 2026-07-19 (fix cycle,
  worktree branch). Mechanism: a level's far tiles *partition* the ground
  plane (no overlap), so a point has exactly one tile per level; center-
  distance ring assignment let an inter-ring boundary cell be rejected by
  BOTH the finer ring (center past its outer edge) and the coarser ring
  (center short of its inner edge), punching a fixed-position sky hole with
  no fallback tile. NOT winding, NOT a missing index. Fix: each ring laps
  its inner edge one own-tile inward (`far_tile_in_ring`), restoring
  between-ring redundancy; the same lap under the near field also cured the
  **grazing-angle near/far slivers**. Headless proof:
  `far_tiles_cover_the_rings_without_seams` (88 463 uncovered points
  pre-fix → 0). Verified live hole-free at both walk-17 vantages + 3 yaw
  sweeps (fullbright). *(Phantom old world + empty horizon: RESOLVED
  earlier, verified at the 0018 framings.)* **Left filed:** faint
  tile-edge stitch lines (one-sided-normal seam) — cosmetic, haze-hidden;
  a cross-tile normal halo is deferred polish.


- Walk 16 (journal/0021 § walk 16): **the lit path erases low-fraction
  mixtures** — DIAGNOSED by same-framing lit/fullbright pair
  (0021-mixture-* assets): olivine wins whole cells in fullbright, zero
  pixels in lit; same splat attributes, so the heightlerp buries it
  (elevation = weight + texture height; a 1/8 accessory can't out-elevate
  a 7/8 host anywhere). Contradicts the ratified "grains poke through"
  intent. RESOLVED 2026-07-19: amplitude-by-rarity + jitter shipped and
  photographically verified (0021-*-v2 pair — olivine visible lit,
  statistical agreement with fullbright; amp constant is the tuning knob
  if ore should read louder). Cell quantization was rejected (grids shear
  authored features). Residual note:
  the houndstooth fix's narrowed albedo spread (0.17→0.11) reduced
  constituent contrast in the lit path generally.

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

- **Walk 12's "blocking regression" was a misdiagnosis** (corrections #10,
  journal/0016): a pose in meters cross-checked against block queries in
  voxels. `true_surface_m` was already deep-time-aware (its ceiling reads
  `ColumnRec`). The investigation still paid: `eye_in_solid` was answering
  from the legacy S1 world on any ChunkMap miss, and a failed surface scan
  silently returned `analytic − 220 m`. Both fixed (merge `81a87b8`;
  `Option`-typed misses, authoritative solidity).
- **Instrument fix: pose replies echo the voxel coordinate — DONE**
  (journal/0021, instrument batch, awaiting integration). `client_player_pose_
  {get,set}` and `dc:character/pose` now carry `pos_voxel` (feet, active-scale
  voxels via the authority's own `scale.voxel_at`) beside the meters `pos` —
  the walker's two languages both labelled, no mental unit conversion (the
  corrections #10 misread that cost a full agent cycle).
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
- Walk 12 residue: *(far-mesh visual assessment: DONE, walk 13 —
  journal/0018 § the empty horizon: at worldgen altitude the horizon is
  sky; the S1 phantom shows only from steep angles through haze. The
  far-field milestone is "build the horizon", not "fix the phantom".)*
  Still open: test-suite time +~6 min (deep-time on every Medium/Large
  pregen — wants a cost-insensitive fast path); Large extent runs a
  coarsened (~1.8 km) deep cell under the width cap until 3e-2's C
  refinement restores landform detail on approach.

- Walk 13 (journal/0018): *(**`surface_snapped` absent on the `surface:true`
  success path: RESOLVED** — journal/0021, instrument batch. The flag is now
  ALWAYS present in a `surface:true` reply: true when the scan seated the feet,
  false on a miss (position left as requested, `surface_error` string). Both
  paths leave through one `surface_teleport_reply` helper, so the flag can't be
  set on only one branch again.)*

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

- Walk 5 (journal/0005, character surface): characters have **no auto
  step-up** — a one-voxel rise halts a grounded walker until it jumps
  *(DECIDED 2026-07-19: stays jump-required; mover-feature vs
  controller-skill is the NPC-intelligence design's question)*.
  *(Disconnect policy: DECIDED 2026-07-19 — freeze for v1, NPC-tier
  degradation as a later controller binding; API.md § Characters,
  bodies.md formerly-open Q3. Attach placement guard: fixed,
  journal/0006.)*

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

- The embedded HostWorld never evicts chunks (~64 KiB per chunk ever
  streamed/edited); never-edited chunks are pure generator output and could
  be dropped freely (journal/0002).
- Far field doesn't see edits — the worldgen far field is a coarse *summary*
  (not a cache), so a broken block un-breaks beyond the full-detail radius; a
  summary that tracks edits is a follow-on (journal/0002, 0022). (Same for the
  S1 far mesh on keys 3/4.)
- Edits don't survive a 2/3/4 scale switch (authority rebuilt); the command
  log is the eventual persistence answer (journal/0002).
- *(**~2/3 of far-mesh triangles were sealed cave surfaces (S3): RESOLVED for
  the worldgen far field** — journal/0022. The worldgen horizon is a **top-surface
  heightfield** (2048 tris/tile, no interiors), not a volumetric shell. The S1 far
  mesh on keys 3/4 is unchanged / still volumetric.)*
- Far meshing is main-thread, budgeted (S1/S3, and the worldgen heightfield) —
  wants async tasks (journal/0022 keeps it budgeted/incremental).
- Rivers are straight cell-chords (S7); course refinement needs the 2-ring
  argument re-proved at finer levels.
- Terrain amplitude conservative — no voxel-scale cliffs (S7).
- Site cap 240 (u8 RegionId) — concrete instance of S2's ledger-scale
  question (S7).
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

- *(**`client_player_pose_set` outside the one door: RATIFIED same day** —
  API.md Decisions log #5. Retirement = player controller through
  controller-verb commands; sequenced after the session-4 agents land,
  since it rewrites files they are touching. See Sequenced.)*

- *(**Console follow-ups: RATIFIED same day** — API.md Decisions log #6
  (`completions` hook) and #7 (registry macro/derive). Dispatched as a
  session-4 background agent — see In flight.)*

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

- **We cannot see where runtime goes — the perf observability gap** (user,
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

- **Material identity is illegible under splat blending — heightmap SHAPE as
  a fix candidate** (user, walk 0071, station 4, 2026-07-22): the charcoal
  specks pass their regression check but are hard to *identify* because
  "everything is honestly so blended." The user's sketch: most material
  heightmaps are currently a random scramble; if each material's heightmap
  carried a **characteristic shape**, heightmap-based splat blending would
  let the eye decode *which* materials were blended, not just that blending
  happened. Explicitly "not the only possible answer, just a thought" — a
  visuals/materials design thread, not a decision. Couples to the
  form-dependent-texture visuals decision already accepted as deferred cost
  in the fill contract (loose vs structural clastic indistinguishable).

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

- **The dominance flip quantizes smooth gradients — a potential S-4 edge**
  (user, walk 0071, 2026-07-22) — **RESOLVED by the susceptibility blend
  (journal/0072), shipped 2026-07-22.** The candidate continuous variant is
  now the mechanism: erosion's four consumption sites blend the per-agent
  susceptibility table by the near-surface window's per-`Litho` *shares*
  (`providers::outcrop_shares`) instead of argmax-then-lookup, so the rate
  field is continuous where the plurality crossover stepped it — argmax is
  its limiting case (a uniform window blends to that rock's rate bit for bit).
  Measured (`examples/outcrop_blend_probe.rs`, Medium): ~18 k former-flip
  adjacencies dropped into the sub-0.1 rate-jump buckets while genuine
  basement↔sediment contacts stay sharp; per-epoch refresh +6 % (negligible).
  Goldens re-baselined (all Medium hashes + the Small block hash, geometry
  only). Audit site A1, shape-teacher #1 of the threshold-quantization
  migration.
  - **NEEDS RATIFICATION (world-scale gameplay consequence):** coal
    diggability fell. A coaly near-surface window is now recessive (coal is
    the softest rock, so its *share* pulls the blended rate up, where the old
    argmax handed a coal-minority window the dominant rock's slower rate), so
    near-surface coal is preferentially stripped. Medium seed
    `0x0D5EED572026` census dropped from 88 record seams over 3 m (strongest
    16 diggable) to **10 record seams over 3 m, strongest 11 diggable
    collapse-voxels** (≈ 10 m — still eminently diggable; the walk-0071 seam
    the user cut and called "looks great" was 3). `MIN_DIGGABLE_COAL_VOX`
    re-baselined 15 → 10 with the census printed (organic.rs), not slid
    silently. The mechanism rides as-built (no-bandaid); this flags the
    reduced-coal *appearance* for the user's blessing.
    **RATIFIED AS-BUILT (user, 2026-07-22, live session: "bless coal
    as-built").** Exposed coal is genuinely recessive; scarcity reads as
    value; deep coal below the window is untouched. If scarcity ever feels
    wrong at play, the lever is calibration (contrast/cap or coal's property
    sheet), never the blend.

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
  earth-processes § 2 (Igneous) is a sketch; nothing is built. Arc/rift
  provenance raises elevation but builds no edifices. What it would buy:
  the fastest legal short-gradation mountain on Earth (a stratovolcano is
  ~3 km of relief in a ~20 km footprint), calderas, lava caprock → mesas,
  ash beds as strata events, hotspot island chains. Constructive
  point-process that feeds the existing erosion sim naturally. Note:
  hotspot tracks REQUIRE plate motion — couples to the tectonic-history
  question (one-shot upheaval, below).

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

- **Olivine reads as exceedingly common and surface-visible (user field
  report, 2026-07-20, during the ore conversation).** Olivine is the SOLE
  accessory-inclusion member (CLASS_ACCESSORY_MAFIC's only entry), so every
  igneous accessory event in the world is olivine — mono-culture by roster,
  not by mechanism. Its presence gate + fixed pore-eighths
  (`emplace_accessory`, geology.rs ~207) and any-depth formation window are
  the tuning surface; whether "exceedingly common" is a gate constant, the
  sole-member effect, or surface exposure bias of extrusives is UNDIAGNOSED
  — measure before touching. The ore work (geology.md § ore DECIDED) will
  both diversify the inclusion roster and make grade meaningful, which may
  resolve the perception without a tuning bandaid.

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

- **The `history.rs` reject-don't-crash skip is SILENT** (2026-07-20,
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

- **Walk 0059 — the holes and the chunk patches are gone; the skin is still
  two flat colours** (2026-07-22, live walk at `--horizon 3 --fullbright
  --edges`; assets `0059-*`). Two of the session-close checklist's five
  questions answered YES: **no sky-holes** at two partial-rich stations
  (journal/0057 confirmed by eye), and **no 28.8 m chunk patches** — a 120 m
  top-down frame shows organic blobs with wandering contacts (journal/0058
  confirmed). The third answer is the defect: the surface reads as **exactly
  two flat colours with a hard one-voxel contact**, no mixed voxel anywhere on
  the skin. Cause (user-diagnosed, integrator-confirmed in code): the surface
  voxel is **not sliced from the column at all** — `ColumnFill::build`
  (`fill.rs:115`) lays the record's top at the surface voxel's *floor*, so
  plans[0] is the voxel *below* it, and `surface_class` (`collapse.rs:786`)
  paints the surface voxel with the **dominant class of that lower voxel**,
  dithered to one member and emitted as a one-element `mixed_contents` call.
  journal/0055 changed the surface's *source* and kept its *branch*. Fix shape:
  `ColumnFill::build` takes the top partial, depth 0 covers record metres
  `[0, frac)`, `allocate_partial` (already exists, `fill.rs:329`) fills its `n`
  eighths from the units actually overlapping. Golden fingerprints move — that
  is the slice's deliverable, as already filed. **HELD** pending the seam-first
  cleanup, at the user's direction: the branch is a leaked LOD requirement and
  wants a declared provision, not another bespoke call.

- **The bare-cell fallback: a walker stood on paint over nothing**
  (2026-07-22, `record_hole_probe.rs`, uncommitted). At deep cell (488, 278)
  — world metres (106 938, 9 953) — the column reads **one voxel of
  `dc:dirt` over 49+ voxels of `dc:stone`**. `dc:dirt` twins only `LOAM`,
  which is registered in **no** class, so it cannot come from the record: it is
  the year-zero fallback, i.e. veneer paint, still under the player. `dc:stone`
  there is *contents-free* (granite/basalt/mudstone/sandstone/coal/peat/carb-
  mudstone all have their own blocks), i.e. unrecorded basement. The cell is
  real but thin: `H 0.25 m · 7 units`, beside a neighbour at **8.0 m / 379
  units**. Census: **0.2 % of land (91 of 44 265 cells) expresses 0 voxels**;
  **0.03 % of adjacent pairs (23 of 87 962)** are bare-beside-≥4-voxels. The
  user walked onto one on the first walk. **Hypothesis FALSIFIED in the same
  probe**: the integrator predicted north-south banding from `wind`'s per-row
  1-D transport lanes (`erosion.rs`: `load` is declared inside the `gy` loop
  and never crosses rows). Measured anisotropy **1.08× ON / 1.06× OFF** — the
  null. Wind does raise overall roughness ~29 % and *halves* the bare count
  (428 → 91) by depositing into scoured cells. Two real defects remain, both
  expression: (a) `regolith_at_voxel` samples **NEAREST** cell while
  `surface_at_voxel` beside it is **bilinear**, so soil depth is a hard-edged
  460 m Voronoi mosaic under smooth terrain — the DECIDED "no simulation-
  resolution edge may reach the eye" doctrine; (b) a column under half a voxel
  of record falls off the record path entirely into fallback paint, so the rare
  scoured cell renders as dirt-over-nothing rather than as honestly thin
  ground. The fractional-top slice fixes (b) by construction. Open design
  tension: `H` is a scalar and interpolable, the **record is not** (a
  variable-length unit list has no midpoint — the documented reason nearest
  was chosen).

- **Charcoal's premise expired and the code still encodes the conclusion**
  (2026-07-22, user-flagged, integrator-confirmed). `geology.rs:312`
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

## NEXT SESSION — written at the 2026-07-27 close (supersedes every earlier block)

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
- **journal/0117 and 0120 are deliberate gaps** — the rename slice and the sweep each judged
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

*(The 2026-07-25 block below is consumed; preserved as history.)*


