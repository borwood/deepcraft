# Roadmap

The living sequence — the only non-append-only document besides code. Update
in the same commit as any journal entry: finished work moves to **Shipped**;
walk/field findings land in **Observed** first (the walk reports, the
diagnosis measures); only diagnosed work gets **Sequenced**.

## Shipped

- 2026-07-18 — Repo, architecture, CI (3-OS), five-crate workspace.
- 2026-07-18 — S1 voxel scale (N=2 decided) · S2 constraint ledger (GO) ·
  S3 chunk format v1 + LOD + far mesh · S4 Forward+ + shader packs ·
  S5 dc-api parity (wasm/MCP/native) · S7 worldgen pregen + lazy pyramid +
  year-zero handoff · S8 materials storage (GO, free-form mixtures).
- 2026-07-18 — Rendering fixes from walk 2: spawn-frame flash, LOD z-fight.
- 2026-07-18 — S6 physics bubble: dc-physics (rapier3d 0.34 direct,
  enhanced-determinism), 4³-voxel collider tiles with set-difference refresh,
  bit-identical replay, detach→settle→reattach via the 24 integer lattice
  rotations, G-key client demo. **Spike era closed: all eight spikes shipped.**
- 2026-07-18 — Client through dc-api + observability harness (journal/0002):
  HostWorld embedded as the client's edit authority (pluggable generator over
  the S1 TerrainGen; ChunkMap demoted to receipt-driven cache); LMB/RMB edits
  as player-class `dc:world/set_block` commands (dc-core DDA raycast,
  crosshair + target gizmo); edit→remesh dirty sets + S6 collider-tile
  invalidation wired; in-client MCP over streamable HTTP :7777 (registry-
  generated tools shared with dc-mcp-dev + client_screenshot to
  journal/assets + player pose get/set); direct/MCP-layer/HTTP-wire edit
  parity proven headless.

- 2026-07-18 — First agent self-walk (journal/0003): MCP session against the
  running game — pose/scan/edit/screenshot loop proven; six findings filed
  to Observed. The practice is established.

- 2026-07-18 — Walk-3 corrections + walker proprioception (journal/0004,
  `0ba292f`): three of four walk-3 "renderer defects" were camera-inside-
  block misdiagnoses (corrections.md #3); shipped eye_in_solid, surface-
  clamped teleport, pitch docs, open-ground spawn; MCP edit pipeline
  photographically verified.

- 2026-07-19 — 3c-1: geology walkable at block tier (journal/0008, walk 7,
  merge `ee639f9`): dc-worldgen through the client seam (PregenSource
  Arc-opening, Rc→Arc caches, one Mutex two worlds — seam-level order-
  independence proven); Mudstone/Sandstone/Granite/Basalt blocks; N=2 boot
  restored (scale keys ratified: 2 = worldgen, 3/4 = legacy S1 dev
  affordance); surface machinery authority-aware; freeze-on-disconnect
  (Drop-fires-once → zero-intent command on the receipted rail), proven
  live mid-stride in walk 7. First geology photographs: quarry cut showing
  granite → basalt → mudstone → soil (assets 0008-*).

- 2026-07-19 — Geology v1 backbone (journal/0007, merge `1df2666`): content-
  class registry (classes-as-contracts; the Payload opening as open keys
  over a closed value vocabulary — NEEDS RATIFICATION with the class-sheet
  fields and pass vocabulary, marked in API.md); pass graph with
  creator/modifier/reader semantics replacing `Pregen::run` (output-
  preserving, S7 byte-identity unchanged); strata recording with climate-
  at-deposition tags; clastic/igneous/placer passes (placer from property-
  derived settle energies — gold-dust lands mid-gravel with zero
  ore-specific code); registration-order independence proven as fingerprint
  equality; perf cost is noise (cold chunk 0.711 ms mean post-geology).

- 2026-07-19 — Surface-truth fix + attach guard (journal/0006, walk 6): the
  walks-3–5 "under-report" diagnosed and closed — analytic field vs its own
  voxelization (½-voxel top-face offset) compounded by footprint-over-slope
  (14.49 m worst; 61% of columns would embed a body). `true_surface_m`
  (footprint-max per-column voxel scan, edits included) now seats spawn,
  `surface:true` teleport, and character attach; embed-guarded attach with
  `obstructed` receipt + opt-in surface-snap; photographically verified at
  the measured worst case. Corrections #5 (the "missing octave" story).

- 2026-07-19 — Bodies/sockets design doc (docs/design/bodies.md), Sequenced
  3b: body plans as registry contracts (anims bind to the plan; fork
  inherits by retained joints; verb→anim-slot validated at define time),
  the animation-is-cosmetic determinism firewall (sim sees parametric
  posture states only), IK as retargeting glue, 12 fps stepped-animation
  aesthetic, clothing as segment-copy shells — all ratified 2026-07-19;
  sockets/transmog mechanics remain PROPOSED.

- 2026-07-19 — Character MCP surface (journal/0005): the second surface from
  API.md § Characters, embodied sessions on :7778. Character primitive in
  dc-api (named body, host-tick swept-AABB stepping, controller verbs as
  commands, diegetic pose/raycast/surroundings senses, all schema-
  registered); `Grant::CharacterControl` with attenuate-to-one-character
  sessions (cage proven at capability/host/HTTP layers; dev surface keeps
  full reach incl. any-character control); two-cuboid companion rendering;
  bit-identical scripted-session replay. Walk: companion attached, driven,
  sensed, photographed standing/walking/jumping (assets 0005-*) — and it
  walked off a cliff, which is the feature.

## In flight

- Body-plan staircase steps 1–2 (bodies.md § staircase; agent launched
  2026-07-19): joint-tree skeleton, plan-as-registry-def, companion
  re-expressed as a biped, MCP-authored data clips; locomotion set +
  crossfade blending + the verb→anim-slot define-time contract.
- Placeholder LabPBR texture packs (code-only agent, 2026-07-19):
  procedurally generated 16×16 three-texture sets per material, assets-in-
  waiting for the splat milestone (visuals.md § mixture road).

## Sequenced
3a. Form archetypes + drop distributions (materials.md § forms) — implement
   with the first inventory/interaction milestone.
3c-2. Geology client integration, material tier (visuals DECIDED
   2026-07-19, visuals.md § mixture road): sidecar channel through the
   seam + interim world-anchored dither materialization (registry albedos,
   eighths-weighted), partial-height loose-material rendering with binary
   threshold collider; subtle ore. Runs after body steps 1–2. Watch the
   order-dependent MixtureTable ids at the seam (journal/0008). Data-driven
   block registry (API.md `define block_type`) supersedes the appended enum
   here or soon after.
4. Biomes-as-diagnosis design (consumers of climate/substrate/disturbance
   axes; registry-defined).
5. Ecology design (succession as derived-from-disturbance state; populations
   as statistical-tier distributions).
6. Body plans implementation staircase (docs/design/bodies.md § staircase,
   ratified 2026-07-19) — six steps from joint-tree skeleton to authoring
   editor; not yet scheduled against the geology track.

## Observed (undiagnosed or deliberately unfixed)

- Walk 5 (journal/0005, character surface): characters have **no auto
  step-up** — a one-voxel rise halts a grounded walker until it jumps
  (mover feature vs controller skill: undecided, belongs to the NPC-
  intelligence design). **Disconnect policy**: a session's last move intent
  persists after disconnect, so an abandoned body keeps walking until it
  hits something — freeze vs NPC-tier degradation needs deciding (API.md
  open question, now concrete; interacts with bodies.md open question 3).
  *(Attach placement guard: fixed, journal/0006.)*

- Geology v1 loose ends (journal/0007): sea-floor/wilds columns keep empty
  strata records (subaqueous sedimentation = the carbonate milestone); no
  `def_changed` events for class/member defines (hot-reload signal; the
  events schema still enumerates only the original three kinds); dev MCP
  token owns only `dev:` — defining `dc:*` vanilla over MCP needs an
  explicit grant decision; worldgen `MixtureTable` is per-generator,
  unbounded, and its ids are generation-order-dependent — a save layer must
  persist the table, never re-derive it (lifecycle belongs to region-file
  grouping, S3 OQ 7).

- Walk 7 loose ends (journal/0008): **unloaded-neighbour and far-mesh
  fallbacks still sample S1 `TerrainGen`** — near-field loaded chunks are
  worldgen, but the far LOD rings and load-radius border faces show the old
  hill-field; a visible seam until the far field becomes worldgen/summary-
  shaped (pairs with the existing far-mesh Observed items). Single-material
  faces under fullbright are featureless color fields — information arrives
  with the 3c-2 dither and later the splat pipeline. The `Terrain` resource
  is retained solely as the 3/4-key legacy fallback. *(Scale-3 boot default:
  fixed in 3c-1; freeze wire-drop path: proven live in walk 7.)*

- Walk 6 loose ends (journal/0006): the surface scan window is
  S1-terrain-sized (8 m headroom / 220 m depth) — structures stacked >8 m
  above the analytic surface won't be snapped to; revisit when worldgen
  amplitude grows. `find_open_spawn` uses terrain-only solidity — correct
  at startup, wrong if ever reused post-edits. Dev `spawn_character` stays
  unguarded (dev keeps full reach) — **pending ratification**, along with
  `surface:true` attach semantics (API.md § Characters, marked).

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
- Far mesh doesn't see edits — farmesh samples TerrainGen directly, so a
  broken block un-breaks beyond the full-detail radius; will be subsumed by
  the summary-shaped far field (journal/0002).
- Edits don't survive a 2/3/4 scale switch (authority rebuilt); the command
  log is the eventual persistence answer (journal/0002).
- ~2/3 of far-mesh triangles are sealed cave surfaces (S3) — column-summary
  skip estimated 3–5×; far field should become summary-shaped (adaptive
  volume), not spherical.
- Far meshing is main-thread, budgeted (S1/S3) — wants async tasks.
- Rivers are straight cell-chords (S7); course refinement needs the 2-ring
  argument re-proved at finer levels.
- Terrain amplitude conservative — no voxel-scale cliffs (S7).
- Site cap 240 (u8 RegionId) — concrete instance of S2's ledger-scale
  question (S7).
- Sparse sidecar encoding for thin debris drapes (S8) — index array dominates.
- Seed-stable worlds across releases: versioning policy undecided (S7).
- HDR/exposure: v0 post grades LDR; sky-as-pass needs hook format 1 (S4).
- S2 checkpoint facts (deep-time re-derivation cost) — design owed before
  ledgers densify.
