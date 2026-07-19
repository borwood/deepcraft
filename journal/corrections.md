# Corrections — claims made and falsified

Check here before re-deriving. Each entry: the claim, why it was wrong, the
mechanism, where the fix lives.

## 1. "Bevy `depth_bias` resolves coplanar z-fighting" (2026-07-18)

**Claim**: giving each LOD level a `StandardMaterial` with progressively
negative `depth_bias` would make finer geometry win depth ties.
**Falsified**: user still saw z-fighting; `depth_bias` only adjusts draw
*sort order* — it never changes the depth fragments write. Coplanar-face
shimmer comes from two tessellations of one plane interpolating depth with
different per-pixel float error, which no draw order fixes.
**Fix**: radial push of each far chunk away from the viewer by 0.15 of its
coarse voxel (`64729c5`, farmesh.rs module docs).

## 2. "Spawn transform doesn't matter; the positioning system runs before rendering" (2026-07-18)

**Claim** (in-code comment): chunks could spawn with `Transform::default()`
because `position_chunks` sets the real transform "before rendering".
**Falsified**: the positioning systems run *earlier in the same frame's
chain* than the streamers, so a newly spawned chunk rendered one frame at
render-space (0,0,0) — the floating origin, up to 256 m behind the player —
producing movement-correlated geometry flashes.
**Fix**: compute the origin-relative transform at spawn (`64729c5`).

## 3. Walk-3 misdiagnosis: "floating LOD shards and a horizon seam line" (2026-07-18)

**Claim** (journal/0003 findings #3): far-mesh defects — detached geometry
hanging in the sky and a hard seam line across the horizon.
**Falsified by user review of the screenshots**: the camera was **inside a
block**. The "seam line" was the outline of the face being viewed from its
reverse side; the "floating" faces were outcroppings whose toward-facing
faces were visible from within the terrain. Observer error, not renderer
error.
**Walk 4 follow-through (journal/0004) falsified the rest of the report:**
- Finding #1 "inverted haze": FALSE — distance haze is normal; the near
  white is flat shading under a near-vertical sun blowing out pale top faces
  (user diagnosis; art calibration, not a shader bug).
- Finding #2 "MCP edits invisible / receipts-vs-events seam": FALSE twice —
  `tick_authority` applies changes from ALL receipts regardless of source
  (code reading), and a pillar placed over MCP photographs correctly from
  open air (`0004-pillar-verified.png`). The walk-3 pillar was invisible
  because the photographer was buried.
**Lesson**: a walker must know whether its own camera is inside solid —
now shipped as `eye_in_solid` + `surface:true` teleport (`0ba292f`). Do not
diagnose renderer defects from a viewpoint you haven't verified is in air.

## 4. "serde `skip_serializing_if` is safe on wire types" (2026-07-18)

**Falsified during S5**: postcard is positional; omitting fields silently
corrupts the stream. Standing rule in API.md § v0 implementation notes;
regression-tested in dc-api.
