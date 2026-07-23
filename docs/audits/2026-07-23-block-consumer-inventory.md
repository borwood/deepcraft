# The `Block` consumer inventory — for the block↔material collapse (Crux 1)

Read-only recon, 2026-07-23 (rescued from the session that shaped journal/0081).
Grounds the block↔material collapse (materials.md DECIDED 2026-07-22; north-star
step 1). **The collapse is ~80% already done at the core**; this names the tail.

## A. The core is already there
- `dominant_material(&VoxelContents) -> Option<MaterialId>` is the collapse's
  core; `classify` is literally `dominant_material().map(block_twin)`
  (`classify.rs:151-156`). The end-state is **return the material, delete
  `block_twin`.**
- The **atlas is already material-keyed** (all 26 materials have a layer;
  `terrain_material.rs:234-240`, journal/0010). `block_layer`'s geology arms are
  pure `Block→Material` re-translation the collapse deletes — the render path gets
  *shorter* (confirmed by journal/0082: the near field already routes
  `contents → material → material_layer`; the residual block-only paths are far
  field / benches / absent-contents).
- The occupancy primitives (`is_occupancy_solid`, `free_eighths`, …
  `classify.rs:319-329`) already exist and absorb the solidity checks.

## B. Consumers, categorized (135 sites / 31 files in the 0052 audit; 146 raw
`is_solid|==Air` hits / 34 files today)
1. **Solidity-shaped (~80/146)** → migrate to the occupancy primitives, NOT to
   material identity. `collision.rs:18-27`, `authority.rs:420-431`,
   `column.rs`, `lod.rs:56-68`, `farfield.rs:112`, `meshing.rs:307-318`,
   `water/vox.rs`. **Can drain first and independently.**
2. **Storage / wire palette (~4 + the enum)** — `voxel.rs:9-41` (`Block` =
   `#[repr(u16)]`, 12 variants, Air=0, positional postcard), `chunk.rs:85`,
   `palette.rs:191-194`, `format.rs`, the block LOD pyramid. **The heaviest
   coupling** (§ C).
3. **Render / atlas (~3)** — already material-keyed; `meshing.rs:236-261`
   (`block_layer` geology arms = pure re-translation, deleted by the collapse).
   The collapse *simplifies* this. What stays block-only: the four legacy S1 packs.
4. **Far-field span / pyramid (~4)** — `farfield.rs:52` (`ColumnSpan{block}`,
   persistence-shaped POD), `farpyramid.rs`, `far.rs`. Content refinement already
   flows through `classify`; the carried atom is still `Block`.
5. **Player-facing name / UI (3)** — `host.rs:46-79` (`dc:*` string tables),
   `schema.rs`, `payload.rs:104`.
6. **Genuine identity/logic (~4)** — `meshing.rs:269-280` (`block_uses_contents`
   trust gate), `block_twin` itself (the deletion target), `host.rs:404-410`
   (legacy S1 `TerrainGen`), demos.

## C. The heaviest coupling + the Air asymmetry
- `Block` = `repr(u16)`, 12 variants, **Air=0**. `MaterialId` = `u8`, 26
  materials, **no Air**. Many-to-one, disjoint id spaces. "Collapse into
  **material + Air**" is precise: the unified atom needs an air sentinel
  `MaterialId` lacks.
- **RATIFIED atom (2026-07-23):** `Block = { Air, Material(MaterialId) }` (air a
  peer, keep the token for `is_solid`/`==Air`); niche `MaterialId` for a **1-byte
  atom** (64→32 KB/chunk — verify).
- **No save layer exists** (`host.rs:172`) → the byte-layout migration has no
  persisted worlds to migrate; cost is in-tree (goldens + consumers).
- **The deep-cell inventory is the same question one tier up** (S16 spike): the
  deep cell stores R/H heights + a strata record, not a material multiset — so a
  behavior "consumes bedrock, produces regolith" has no inventory. **This is the
  keystone Crux 1 and the behavior model both converge on.**

## D. Ratified plan + open forks
- **Order:** render-first (subsumed — done as guard test 0082) · **redefine
  `Block={Air,Material}`** (the real first slice) · drain solidity→occupancy ·
  storage atom · retire legacy S1 · then categories-registrable.
- **Crux 2 RATIFIED:** the four legacy S1 blocks + `TerrainGen` are **retired,
  not enshrined** (verify no live fallback first, re-point at real worldgen).
- **9 materials lack a block-tier face** (fall through `_ => Stone`): SNOW,
  LEAF_LITTER, POTSHERD, KNAPPING_DEBRIS, ASH, SCREE, BONE, GOLD_DUST, OLIVINE —
  but the atlas already gives all 26 a mixed-face layer; the gap is the *block-tier
  one-name summary* only.
- **Downstream, parked:** the `block_uses_contents` trust gate (does the collapse
  force edits-write-materials?).
