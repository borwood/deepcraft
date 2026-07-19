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

## In flight

- **Client through dc-api + observability harness**: player input as
  commands against a hosted world; in-client MCP surface; screenshot capture
  to journal/assets; agent self-walk practice (orogeny's 0033 pattern).

## Sequenced
2. Character MCP surface (`dc-mcp-character`): grant-scoped embodied agent
   play — same generator as dc-mcp-dev, different grants.
3. Geology deep-dive — backbone seeded in docs/design/geology.md
   (classes/processes/contexts as registry data); v1 content selection is the
   remaining conversation. Mine orogeny's stratigraphy-v1 — source of ideas,
   not a spec.
3a. Form archetypes + drop distributions (materials.md § forms) — implement
   with the first inventory/interaction milestone.
3b. Bodies/sockets design doc (body-as-plugin, controller-driven, transmog) —
   after character MCP.
4. Biomes-as-diagnosis design (consumers of climate/substrate/disturbance
   axes; registry-defined).
5. Ecology design (succession as derived-from-disturbance state; populations
   as statistical-tier distributions).

## Observed (undiagnosed or deliberately unfixed)

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
