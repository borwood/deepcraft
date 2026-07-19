# 0003 — First self-walk

*2026-07-18 · agent self-walk (Claude, via the in-client MCP surface)*

> blogworthy: an AI walked a game engine it helped build the same day the
> observability harness merged — connected over MCP, teleported, read the
> world, edited it, screenshotted what it saw, and caught four rendering
> defects and one integration bug that every headless test had passed over.

The harness works: initialize → tools/list (12 tools, schemas accurate) →
pose, scan, edit, screenshot all round-tripped over plain HTTP. Screenshots
landed in `journal/assets/0003-*.png` by convention. `world_scan_region`
came back palette-coded and legible (stone to y=12, dirt 13, grass 14 —
surface at 8.4 m, matching S1's hills), and receipts carry from/to per
block. The loop orogeny proved for a human walker works with the agent as
the walker.

## Field report

**Positive**: the MCP session is solid; the target-voxel gizmo and crosshair
render; the scan/edit/receipt path is coherent; teleport + screenshot is a
workable survey loop (with a ~2 s settle for streaming).

**Defects, in severity order:**

1. **Near-field white wash — haze reads as inverted** (`0003-pillar.png`,
   `0003-pillar-close.png`): terrain within ~5–30 m blows out to white while
   distant hills render crisp. Suspect the post stage's depth linearization
   misreads reverse-Z — maximum haze applied to *near* pixels. S4's visual
   check compared packs side-by-side but nobody looked at the ground at
   walking distance. Worst defect; makes near-field survey shots useless.
2. **MCP-sourced edits are invisible** — an integration bug the parity tests
   couldn't see: `world_set_block` receipts log correct from/to and
   `world_get_block` confirms `dc:stone` in the authority, but the placed
   pillar never appears in the render (`0003-pillar-far.png`,
   `0003-pillar-close.png`; the broken block also shows no hole). Hypothesis
   with a mechanism: the client cache applies effects from receipts of *its
   own* submissions; a different consumer's edits only reach it via
   `block_changed` events, which nothing in the client subscribes to yet.
   Headless parity passed because it hashed the *authority*, not the cache.
3. **Floating LOD geometry** (`0003-pillar-far.png`): vertical shards and a
   detached slab hanging in the sky above the horizon, plus a **hard
   horizontal seam line** across the whole frame at horizon height —
   far-mesh ring artifacts the S3 walk didn't surface.
4. **White speckle on complex cliff faces** (`0003-pillar-far.png`, right
   side): the chasm walls sparkle white — possibly residual overlap-band
   z-fighting at vertical faces, or the same haze defect interacting with
   depth discontinuities.
5. **Spawn is inside the chasm**: surface at the origin is the chasm floor
   (−82 m), so the first thing a new walker sees is a wall
   (`0003-spawn.png`). Cosmetic, but spawn should seek open ground.
6. **Doc gap**: `client_player_pose_set`'s pitch documents range but not
   sign (negative = down — discovered by wasting a screenshot on sky,
   `0003-vista.png`).

## Note for the loop

Same lesson as orogeny 0019, now with the agent as walker: three kinds of
miss again — a perceptual defect no hash can see (the wash), an integration
seam between two correct systems (receipts vs events), and absent
capability documentation (pitch sign). All were obvious within ~15 tool
calls of actually looking. The screenshot-to-journal-assets convention made
this entry nearly write itself.

## Status

- [x] first agent self-walk completed; six findings filed to ROADMAP Observed
- [ ] fix the near-field haze inversion (first — it blinds all future walks)
- [ ] subscribe the client cache to block_changed events (cross-consumer edits)
- [ ] far-mesh floating shards + seam line diagnosis
