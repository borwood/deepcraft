# Roadmap

The living sequence — the only non-append-only document besides code. Update
in the same commit as any journal entry: finished work moves to **Shipped**;
walk/field findings land in **Observed** first (the walk reports, the
diagnosis measures); only diagnosed work gets **Sequenced**.

## Shipped

- 2026-07-19 — **PBR-1: the real material renderer** (journal/0019, merged
  `c49566f`, gates green on merged main; **walk 14 done** — which caught a
  ±Z-face NaN in the shader's tangent frame within three frames, fixed in
  integration, journal/0019 § walk 14). The interim vertex-color dither/mosaic
  (journal/0010) is replaced by a **custom Forward+ LabPBR material**: three
  `texture_2d_array`s (basecolor / normal+AO / specular), layer index = material
  id, blended per fragment by height/AO contrast (heightlerp). **Mixed faces
  return to single quads** — the mosaic's 16× geometry is gone
  (196 608 → 12 288 tris on a fully-mixed 32³ chunk); constituents ride as
  per-vertex splat attributes (`Uint32x4` layers + `Float32x4` weights), top-N
  chosen by the world-anchored hash (the >N path is test-only headroom).
  **Uniform-contents voxels now sample their MATERIAL pack, not their block's**
  — this **kills the walk-10 "member identity is render-invisible" item**
  (siltstone renders differently from mudstone). The atlas is widened past the
  material count with four **block-only** layers (grass/dirt/stone/wood) so a
  *single* material renders the whole lit world — near geology, uniform strata,
  the far LOD rings, and the legacy S1 terrain — with no seam. `--fullbright`
  path unchanged: the same mesh carries both vertex color and splat data, and
  the streamer picks the unlit `StandardMaterial` under the flag — but walk 14
  found mixed-face *appearance* changed (speckle loss, Observed below). Directional sun + hemispherical ambient only (PBR-2 owns shadows,
  point lights, tonemap/HDR, POM, water; never GI). Placeholder packs regenerated
  21 → 26 to cover the full `MaterialId` registry (the 3d roster widening:
  siltstone/conglomerate/diorite/andesite/olivine got packs). **NEEDS
  RATIFICATION**: `SPLAT_N = 4` (the per-face material cap, like 0010's 4×4 dither
  flag) and the sun/hemi-ambient calibration constants (aesthetic, user-owned —
  the walk judges them). Files: `terrain_material.rs`, `shaders/terrain.wgsl`
  (new); `meshing.rs` (splat rewrite); `app.rs`/`streaming.rs`/`farmesh.rs`/
  `authority.rs` (material wiring); `tools/gen_placeholder_textures.py` + the 5
  new packs + manifest.

- 2026-07-19 — S1-fallback sweep (journal/0017, merged `--no-ff` to main,
  gates re-run on merged main by the integrator): closed the defect class
  journal/0016 named. The client now
  has ONE world-answer surface for solidity — `Authority::is_solid_voxel`
  (lazily generating the hosted world, edits included) — and the six near-field
  consumers that answered from the legacy S1 `TerrainGen` on a ChunkMap miss now
  route through it: player collision, character foot-IK grounding, crosshair
  edit raycast, physics collider tiles, and mesh-border culling
  (`streaming.rs` + `remesh_dirty`). Guard: `ChunkMap::is_solid` (the fallback
  method) deleted; the ambient `Terrain` resource deleted (no system can
  `Res<Terrain>` a wrong world); tripwire test
  `empty_cache_solidity_paths_read_the_worldgen_authority` fails on old main and
  catches all six sites; doctrine drafted in ARCHITECTURE.md (**NEEDS
  RATIFICATION**). Meshing measured *faster* through the authority (0.93 vs 1.17
  ms/chunk — generate-once-and-memoize beats per-voxel S1 noise). Keys 3/4 (S1
  authority) unchanged; all 38 suites green. **Far mesh left on S1** (residue
  below) — sourcing coarse far rings from worldgen is renderer/storage-scale
  work, not cheap. **Walk-13 verified live** (journal/0018): unstreamed-edge
  collision (mid-air character spawn → landing at a never-streamed column),
  authoritative `eye_in_solid`, edits-included `surface:true` seating, and
  the first deep-time cut-face photographs (0018 assets — mudstone/basalt/
  olivine-speckled granite at outcrop scale).

- 2026-07-19 — Body staircase step 3 (journal/0014, walk 11, merge
  `f0e9f2d`): trunk-follows-travel / head-follows-look via the neck
  (walk-8 orientation gap closed, photographically verified thanks to
  the new v0 brow face cue); two-bone leg IK with stepped output and
  half-voxel offset cap; parametric crouch split exactly on the firewall
  (`dc:character/set_posture`, 0.6× collider, stand-up guard,
  PostureBlocked receipt; procedural pose cosmetic-side); posture replay
  bit-identity proven.

- 2026-07-19 — S9b parallelism spike (journal/0013, merge `9910c45`):
  the determinism tax measured — flood has no byte-identical parallel
  form (98.5% serial floor at B); deterministic-parallel phases 1.2×
  whole-step, bandwidth-saturated by 8 threads; **A+C confirmed**,
  corrections #9 (S9's 2-minute flip condition falsified on ≤12-thread
  hardware; reopenable via deeptime_par --full-b elsewhere). Scatter→
  gather diffusion reformulation, byte-identity proven scalar-vs-parallel.

- 2026-07-19 — S9 deep-time spike (journal/0012, merge `3fd00bb`): the
  spike era reopens and pays off — two-plane erosion + measurement-tagged
  strata recorder + orographic march, additive in dc-worldgen. A/B/C
  measured (A 14 s; B ~63 min/~3 GiB scalar — corrections #8; C region
  refinement 3.6 s); decay length 21 cells (hillslope), drainage the sole
  advect; 500 m columns already tell true stories. Verdict: A+C unless
  parallelism flips B (S9b decides). Integration fought and won a
  stale-worktree env!-path bomb in the parity tests (corrections #7).

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

- 2026-07-19 — 3d: geology post-v1 slice (journal/0011, walk 10, merge
  `ba666e6`): igneous fitness weather-blind (per-class contracts —
  corrections #6: the seam's real quantizer was the chunk-column collapse
  unit, not cell-stepped climate); member-contact boundary dither
  (data-proven wandering contacts); two-layer class-satisfiability
  enforcement with named culprits; olivine pore-partial inclusions
  (photographed, assets 0011-*); roster proof 17→22 materials, 10 vanilla
  members across 6 classes.

- 2026-07-19 — 3c-2: material-tier geology visible (journal/0010, walk 9,
  merge `8c9f67f`): registry albedos on all 17 materials; sidecar channel
  through the seam (order-dependent MixtureTable ids resolved to a
  render-only ContentsGrid at the generator boundary — no intern id
  crosses the lock); 4×4 world-anchored deterministic dither on mixed
  faces (subtle ore with zero ore-specific code); partial-height loose
  rendering built-but-dormant. First mixture photographs from a placer
  cut 60 km out (assets 0010-*); `examples/river_cells.rs` dev tool born
  of the hunt.

- 2026-07-19 — Body staircase steps 1–2 (journal/0009, walk 8, merge
  `dd9b802`): body plans + anim clips as namespace-owned registry data
  (fourth roles-as-contracts instance; clips standalone-before-plan, flagged
  in API.md); the 11-segment `dc:body/biped` replacing the two-cuboid
  companion; stepped 12 fps sampler (32-step rotations, 5 mm bob — the bob
  quantization also fixed a ULP loop-wrap divergence), crossfade idle↔walk,
  verb→anim-slot contract enforced at define time. Walk-photographed
  standing and mid-stride (assets 0009-*).

- 2026-07-19 — Placeholder LabPBR texture packs (merge `78e9d95`, asset-
  only): 21 deterministic 16×16 three-texture sets generated from the
  property sheets; seamless tiling verified after user query and now
  asserted by the generator's self-check (`ea7251d`).

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

- **PBR-1 render polish awaiting integration** (journal/0020, background agent,
  separate worktree branch): the two walk-14 ratifications — the placeholder
  tiling cleanup (world-anchored UVs + de-directionalized textures) and the
  fullbright mixture-speckle variant (a second unlit custom material). Gates green
  on the worktree branch (fmt, clippy `-D warnings`, full workspace test suite);
  live-verified lit + fullbright on the RTX 3070 with no shader/pipeline errors.
  Merges on top of PBR-1. Owed to the milestone walk: the geology-outcrop
  before/after and the in-world fullbright speckle on a real mixed face (see
  Observed). No NEEDS RATIFICATION flags (the SPLAT_N=4 + calibration flags from
  PBR-1 still ride; this pass added none).

- **PBR-1 awaiting integration + milestone walk** (journal/0019, background
  agent): gates green on the worktree branch (fmt, clippy `-D warnings`, 105+
  workspace tests; live-verified lit + fullbright on Vulkan/RTX 3070 with no
  shader/pipeline errors). The integrator merges `--no-ff`, re-runs gates on
  merged main, and the main session runs the milestone walk (shot list in
  journal/0019). Two **NEEDS RATIFICATION** flags ride: `SPLAT_N = 4` and the
  sun/hemi-ambient calibration (both aesthetic/user-owned). Next Sequenced after
  it: **PBR-2** (shadows/point-lights/HDR/POM/water/weather + the `opaque.terrain`
  pack hook), or the far-field summary-pyramid work (journal/0017 § far mesh —
  walk 13 showed the worldgen horizon is empty sky, so that milestone is building
  the horizon, not fixing an artifact).

## Sequenced

**PBR-2 — lit-world completion** (the deferred half of the renderer, opened by
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

**S10 — biotic-layer spike** (the gate for ecology work; design in
docs/design/ecology.md): community vector + the six processes on the
3e-1 A-tier; measure cost against the ~14 s world-creation ritual and
read-quality — do we get coal seams, paleosols, charcoal bands,
retrogressive surfaces? Evolution explicitly out of scope.

**3e-2 — C refinement** (needs deciding first, per the 3e-1 report): the
coarse→fine drainage handoff (decide drainage coarse at A, inherit area
as a fixed boundary field); how a refined region's strata stitch into the
collapse pyramid within LOOKAHEAD_BOUNDS given the measured 16–24-cell
halo; the approach trigger; whether C supersedes the Large-extent deep
grid width cap; and the iteration↔Myr / cell↔km calibration.

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
   - **S10 biotic-layer spike** is the gate: community vector + six
     processes on the A-tier; measure cost vs the ~14 s ritual and
     read-quality (coal seams, paleosols, charcoal, retrogression).
     Evolution explicitly out of scope for S10.
6. Body plans implementation staircase (docs/design/bodies.md § staircase,
   ratified 2026-07-19) — six steps from joint-tree skeleton to authoring
   editor; not yet scheduled against the geology track.

## Observed (undiagnosed or deliberately unfixed)

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
    whole lit world). The phantom-far-world defect (below) is unchanged — it now
    paints a *textured* phantom ~1 km down; still needs the summary-shaped far
    field, not a renderer fix.
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
    Shipped line still reads "21 deterministic packs" (not amended — dated
    record).
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
- **Instrument fix owed**: pose replies should echo the **voxel** coordinate
  beside the meters — the walker speaks two languages and nothing labels
  which. This misdiagnosis cost a full agent cycle.
- **Far mesh still generates from S1 `TerrainGen`** (journal/0017 — the one
  residue the S1-fallback sweep left standing; the six *near-field* consumers
  are shipped). Under the worldgen authority the far rings paint a phantom old
  world ~1 km below the real terrain that dissolves on approach. It is now
  isolated in a single loudly-named resource (`farmesh::FarFieldTerrain`) whose
  doc names the defect — honestly wrong, not silently wrong in a shared cache.
  **Shape of the fix** (renderer + storage scale, not cheap — investigated and
  deferred): the far mesh samples a *coarse voxel scale* (2^L base voxels), but
  the worldgen generator is N=2-baked with no coarse-sampling path. The right
  answer is a **persisted coarse summary pyramid** — worldgen emits per-column
  height/material summaries at coarse LOD levels on save; the far mesh reads
  cached summaries. Couples to "far field should become summary-shaped"
  (Observed below), S3 region-file grouping, and the LOD-derive-on-save path
  exercised in `--bench-storage`. Full-res-generate-then-downsample is the
  non-starter alternative (~5 min for 1 km). Until then: also blocks fully
  sealing `TerrainGen` behind `pub(in crate::authority)` — the far mesh is its
  last near-namer. Visual assessment of the phantom vs deep-time terrain still
  owed (walk 12/13).
- Walk 12 residue: *(far-mesh visual assessment: DONE, walk 13 —
  journal/0018 § the empty horizon: at worldgen altitude the horizon is
  sky; the S1 phantom shows only from steep angles through haze. The
  far-field milestone is "build the horizon", not "fix the phantom".)*
  Still open: test-suite time +~6 min (deep-time on every Medium/Large
  pregen — wants a cost-insensitive fast path); Large extent runs a
  coarsened (~1.8 km) deep cell under the width cap until 3e-2's C
  refinement restores landform detail on approach.

- Walk 13 (journal/0018): **`surface_snapped` is absent from the
  `surface:true` teleport reply on success** — the snap is only inferable
  from y moving; absence-means-success is instrument ambiguity (pairs with
  the owed voxel-coordinate echo in pose replies, same fix batch).

- Walk 11 + step-3 loose ends (journal/0014): *(crouch factor: DECIDED
  2026-07-19, bodies.md § Crouch semantics — 0.6× stands; crouch is
  posture + the future sneak edge-walk-block verb, never crawlspace
  access; sub-standing clearances belong to prone/crawl, sketched in
  ideas.md § posture ladder.)* Max-bend leg fold reads
  tangled — knee pole/fold distribution wants a photo-driven tuning
  pass. Posture is not exposed in `character_pose` (a driver can't read
  its own posture back). Foot-IK ground read can momentarily see the S1
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
