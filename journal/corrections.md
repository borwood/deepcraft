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
error. The finding #1 "inverted haze" diagnosis is also suspect for the same
reason (white wash may be far-plane fog showing through culled near faces)
— re-verify from provably-open air before fixing any shader.
**Lesson**: a walker must know whether its own camera is inside solid. A
"camera in solid" indicator (or teleport surface-clamping) is walker
infrastructure, not polish.

## 4. "serde `skip_serializing_if` is safe on wire types" (2026-07-18)

**Falsified during S5**: postcard is positional; omitting fields silently
corrupts the stream. Standing rule in API.md § v0 implementation notes;
regression-tested in dc-api.
