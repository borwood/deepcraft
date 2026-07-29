# Near-field per-chunk material-palette quantization — generation diagnosis

> **⚠ STALENESS BANNER — stamped 2026-07-29 by the member-#0 design pass; CORRECTED the
> same evening.** (Immutable body, mutable header; this doc had no banner and its
> `file:line` citations have all drifted — e.g. `collapse.rs:1474` → `:1406`,
> `field.rs:442-450` → `:779-787`. **The substance was re-verified correct at the drifted
> addresses at `4f773cc`.**)
>
> **Its § 7 INFERRED question — 460 m tiles or 28.8 m member-fitness stepping? — was
> ANSWERED THE SAME DAY it was asked and this doc never learned.** The 2026-07-24 close
> ruled it *"settled — member stepping"*; the primary evidence is **corrections #45** (the
> member dither is world-anchored and C0-continuous; the defect is **single-octave**, not
> anchoring) and the same-day haunting diagnosis (*"the member squares are one octave of
> value noise at chunk wavelength — **fix = octaves, not resolution**"*). Both defects are
> real — the 460 m facies point-sampling AND the single-octave member dither — but the
> **dominant visible signal is the member stepping**, so the near-path fix alone will not
> clear the checkerboard; **the octaves `DitherSource` is a co-requisite** (now E5
> member #0's own socket).
>
> *The first stamping of this banner said "unprobed for five days" — wrong: the resolution
> existed and lived only in a close block, which is a handoff, not an authority. The
> failure here is the one-directional pointer, and the 2026-07-29 design pass re-derived
> the question because this doc held no link to its own answer. User memory caught it.*
> See `docs/audits/2026-07-29-member0-coarsefield-design.md` § 3 + header rulings.
>
> **⚠ THE FAR-FIELD HALF OF THIS DOC'S MECHANISM IS SUPERSEDED IN CODE, 2026-07-29
> (journal/0125).** Every `surface_class` → `draw_class` chain this body describes
> (`§ 4`, `§ 5`, `§ 6`) is gone: `draw_class` retired into
> `dc_core::coarse::ShareVec::draw`, and `surface_class` now samples a
> `CoarseField<ShareVec<6>>` window through `sample_dithered`, which **dithers which of
> the four surrounding deep cells supplies the shares** before drawing. So the far
> field's *"reads `record_at_voxel` NEAREST at the ~460 m grid"* is **no longer true**
> and the 460 m frontier is no longer a step there. **The NEAR path this doc is actually
> about — one chunk-centre point sample shared by 1024 columns — is UNCHANGED**, and so
> is everything in § 0's TL;DR. Read the body as accurate about the near field and
> historical about the far one.

**Date:** 2026-07-24
**Mode:** DIAGNOSIS ONLY (read-only on code; no fix, no plan). Every claim carries
`file:line`. A final section separates **PROVEN** (static from the code) from
**INFERRED / needs-live-probe**.

Commit base: `main` (7b0db5f). Paths under `crates/`.

---

## 0. TL;DR

The near-field surface reads as a grid of squares each with a **distinctly
different overall material composition** because a chunk's **entire strata
composition is built from ONE point-sample of the deep-time record, taken at the
chunk centre**, and that record is sampled **NEAREST at the ~460 m deep-cell
grid**:

> `deep_units = self.pregen.deep.record_at_voxel(cx*32+16, cz*32+16)`
> (`collapse.rs:1474-1478`), where `record_at_voxel` rounds to the nearest deep
> cell (`field.rs:442-450`, `deep_coords` `field.rs:383-397`), and the deep cell
> edge is `DEEP_CELL_M = 460.0` (or coarser for very large extents —
> `field.rs:41`, `field.rs:110`).

That single sample feeds `run_strata` (`collapse.rs:1479-1498`), which produces
**one `StrataRec` for the whole 32×32 chunk footprint**. Every voxel column in
the chunk is skinned from that one record. So:

- **Composition (which classes, in what thicknesses) is a per-chunk scalar
  computation** — uniform across all 1024 columns of a chunk.
- Its only **hard-stepping inputs** are `record_at_voxel` (the strata stack) and
  `regolith_at_voxel` (the veneer thickness), **both NEAREST at ~460 m**
  (`field.rs:425-433`, `:442-450`). Every other per-chunk input (climate, mean
  elevation, flow energy) varies **smoothly**.
- Therefore the composition is **identical for all chunks sharing a deep cell**,
  and can only change across the **~460 m deep-cell grid**. Because the sample is
  taken at the chunk **centre**, that ~460 m boundary is **quantized to the
  28.8 m chunk grid** — a staircase of chunk-edge composition jumps tracing each
  deep-cell boundary.

The **weathering/dune pattern runs continuous across the seams** because the
things that carry the pattern are per-voxel, world-anchored, and C0-continuous:
the elevation lattice (`collapse.rs:1109-1162`), the within-class member dither
(`dithered_member`, `geology.rs:653-674`, over the shared-corner bilinear field
`interp_select_draw`, `geology.rs:198-213`), and the mesher splat
(`voxel_seed`/`top_splat`, `meshing.rs:345-397`). Those never step at a chunk or
cell boundary. Only the **class stack** does — the axis the user sees as
"materials discontinuous while pattern continuous."

This is the **same class of mechanism as the LOD "blobs"** (coarse deep-facies
field point-sampled and expressed to the eye as tiles), but a **different code
site and a different stride** — see § 4.

---

## 1. The square size — what tile the discontinuity aligns to

### 1.1 The composition is a per-chunk scalar computation

`column(cx, cz)` (`collapse.rs:1386-1537`) builds the whole chunk footprint. The
strata context it hands to the passes is **all chunk-level scalars**, each
sampled at the chunk centre `(cx*32+16, cz*32+16)` or as a footprint mean:

- `temp_sl, precip = climate_at(cx*32+16, cz*32+16)` (`collapse.rs:1392`) —
  bilinear (`collapse.rs:1196-1215`), **smooth**.
- `regolith_m = deep.regolith_at_voxel(cx*32+16, cz*32+16)` (`collapse.rs:1400-1403`)
  — **NEAREST at ~460 m** (`field.rs:425-433`).
- `provenance` from the parent pregen cell (`collapse.rs:1450-1459`) — steps at
  the province grid (`CELL_VOXELS = 16384` voxels ≈ 14.7 km, `pregen/mod.rs:40`).
- `elev_m = mean_h` over the footprint (`collapse.rs:1460-1461`) — **smooth**.
- `flow_energy` at the footprint centre (`collapse.rs:1465-1470`) — **smooth**
  (spikes near rivers).
- `deep_units = deep.record_at_voxel(cx*32+16, cz*32+16)` (`collapse.rs:1474-1478`)
  — **NEAREST at ~460 m** (`field.rs:442-450`).

`run_strata(&mut strata_ctx)` (`collapse.rs:1497`) runs once and yields one
`StrataRec` (`collapse.rs:1498`) shared by all 32×32 columns. The surface skin is
then painted from that one record's top span for every column
(`collapse.rs:1512-1524`), and the buried fill from the same record
(`generate_chunk` `collapse.rs:457-471`, `material_ids` `collapse.rs:573-618`).

### 1.2 Only two inputs step hard, and both step at ~460 m

Of the inputs in § 1.1, only `record_at_voxel` and `regolith_at_voxel` are
NEAREST; the rest are bilinear/means (smooth). Both NEAREST inputs are read at
the **deep grid** resolution:

- `DEEP_CELL_M = 460.0` (`field.rs:41`); production edge is
  `max(extent/DEEP_MAX_WIDTH, 460)` (`field.rs:110`) — **≥ 460 m**, coarser for
  very large worlds.
- `record_at_voxel` → `deep_coords` → `.round()` on the deep-grid coordinate
  (`field.rs:446-449`, `:394-395`). The strata **doc states the sampling choice
  outright**: "a variable-length unit sequence cannot be interpolated, so the
  facies story steps at the ~460 m deep-cell grid — a geologically legitimate
  scale, coarser than the 28.8 m chunk grid" (`field.rs:435-441`).
- The far-field test comment restates the same fact from the other side: "A deep
  cell's top-window class shares are constant across the whole cell
  (`record_at_voxel` is NEAREST at the 460 m grid)" (`collapse.rs:2269-2270`).

### 1.3 Verdict on the square

**PROVEN:** the **hard composition boundary aligns to the ~460 m deep-cell grid**
(and, for the igneous basement class only, to the ~14.7 km province grid), and it
is **quantized to the 28.8 m chunk grid** because the deep record is
point-sampled at each chunk's centre. Two chunks inside one deep cell are
compositionally **identical**; the jump happens only where their centres fall
into different deep cells.

**The user's "per chunk" reading is the chunk-quantized edge, not a per-chunk
tile.** The composition *tiles* are ~460 m deep cells; their *boundaries* land on
28.8 m chunk lines (staircase), which reads as "the squares are chunks." A single
deep cell is 16×16 chunks. Whether the user is seeing whole 460 m cells or the
chunk-granular staircase along a cell boundary depends on the vantage (§ 7).

The locale tile (`L_LOCALE = 512` voxels = 460.8 m, `collapse.rs:59-60`) is the
**same scale** as the deep cell but is a **different registration** — the
composition switch is specifically the **deep grid** (`deep_coords`, anchored to
the pregen grid centring), not `vx.div_euclid(512)`. So "deep cell", not
"locale", is the tile to probe against.

---

## 2. The selection site and its sampling stride

The composition (class/member) of a surface voxel is decided in two stages, only
the first of which steps:

**Stage 1 — the class stack (the stepping stage).** The set of content classes
present in a column and their thicknesses is `run_strata`'s output, and it is a
pure function of the chunk-level context of § 1.1. The classes come from:

- `deep_class(u.tag)` for each recorded deep unit (`geology.rs:355-372`), inside
  `deposit_deep_history` (`geology.rs:420-481`) — the units are `ctx.deep_units`,
  i.e. the **one chunk-centre NEAREST deep sample**.
- the igneous province class (`igneous_pass` `geology.rs:254-282`), from
  `provenance` (~14.7 km).
- the clastic veneer's fixed `CLASS_CLASTIC_COARSE` / `CLASS_CLASTIC_FINE`
  (`clastic_pass` `geology.rs:496-591`), whose **thicknesses** come from
  `regolith_m` (NEAREST 460 m) + smooth `flow_energy`.

**The effective stride of Stage 1 is the deep grid (~460 m), point-sampled once
per chunk at the chunk centre.** That is the exact origin of the per-tile
variation.

**Stage 2 — the member within a class (the smooth stage).** Given a class, the
host member is `dithered_member(&geology, seed, &event, cx, cz, x, z)`
(`geology.rs:653-674`), evaluated **per voxel column** off `interp_select_draw`
(`geology.rs:198-213`), whose four corners are chunk-grid points shared with
neighbours → **C0-continuous across chunk borders** (`geology.rs:188-194`).
Surface: `collapse.rs:686-705`, `:1519-1521`. Buried: `collapse.rs:457-467`,
`:598-608`. This axis does **not** produce square boundaries.

Note the subtlety that resolves the brief's puzzle ("if both mesher and contents
dither are world-anchored per-voxel, per-chunk uniformity cannot come from
either"): `dithered_member` **is** world-anchored in its coordinates, but the
`event` it dithers *within* — its class, `sel_salt`/`sel_tag`, and formation
context `temp_c/precip/depth_m` (`geology.rs:65-79`, `:665-670`) — is a
**per-chunk product of the chunk-centre deep sample**. The per-voxel dither only
walks the member around *inside* the class the chunk already fixed; it cannot
cross the class boundary the deep sample drew. So the uniformity comes from
**neither the mesher nor the dither**, but from the **Stage-1 class stack** that
both of them are downstream of.

---

## 3. The "hard per-chunk lookahead bounds" (`collapse.rs:76-93`)

`LOOKAHEAD_BOUNDS` (`collapse.rs:87-93`) and the `Trace` instrumentation
(`collapse.rs:107-139`, asserted `collapse.rs:1065-1076`) cap the number of
*distinct cells/regions/locales/columns* one `generate_chunk` may consult. They
bound **breadth of lookahead**, not the sampling stride, and they are **not** the
source of the quantization:

- The bound `columns: 2` (`collapse.rs:91`) confirms a chunk consults essentially
  only its own column record — reinforcing that the composition is a
  **per-chunk** computation with no cross-chunk averaging that would smear a tile
  boundary.
- The deep record is consulted through `record_at_voxel` at a **single voxel**
  (the chunk centre), so it touches one deep cell and never trips the bound.
  Nothing here changes *which* coarse cell a voxel consults per-voxel; the whole
  chunk consults the **one** cell its centre lands in.

So the lookahead bounds are consistent with (and quietly corroborate) the
per-chunk-single-sample finding, but they are not themselves the mechanism. The
mechanism is the **point-sample at chunk centre of a NEAREST-quantized field**,
not a bounded neighbourhood.

---

## 4. Relation to the LOD cold-synth blobs

**Same class of mechanism, different site and stride.**

- **Common root:** the deep-time facies record is a **coarse field, NEAREST at
  ~460 m** (`field.rs:442-450`). Neither the near tier nor the far tier
  **area-summarizes** it; both **point-sample** it and express the point value as
  a tile/blob. That is why both look like "coarse class expression reaching the
  eye as a square."

- **Near-field site (this diagnosis):** `record_at_voxel(chunk centre)` →
  `run_strata` → the whole chunk's class stack (`collapse.rs:1474-1498`). Stride =
  the deep grid, sampled **once per chunk**. Boundary = ~460 m, chunk-quantized.
  The near path does **not** go through `surface_class` at all — journal/0074
  removed the near ground's class consult; the near surface is the record's top
  span through `ColumnFill` (`collapse.rs:1500-1524`, and the doc
  `collapse.rs:900-905`).

- **Far-field site (the LOD diagnosis,
  `docs/audits/2026-07-24-lod-pre-post-visit-diagnosis.md` § 3):** `coarse_surface`
  → `surface_sample` → `surface_class` → `draw_class` (`collapse.rs:920-988`,
  `:798-871`, `:1011-1019`), point-sampled at the **coarse LOD voxel stride**
  (2–16 base voxels). `surface_class` *itself* also reads `record_at_voxel`
  NEAREST (`collapse.rs:923`) and then adds a **class-membership dither** over
  `interp_select_draw` at `SALT_GEO_CLASS` (`collapse.rs:970-974`) — the extra
  dither the far tier has and the near tier does not.

So: **the near-field squares and the far-field blobs are two expressions of the
same coarse deep-facies field, at two tiers.** The near tier renders it as a
~460 m deep-cell tile with chunk-quantized edges (one point-sample per chunk); the
far tier renders it as blobs (one point-sample per coarse voxel, plus the class
dither). They are **not literally the same code path** — near = `record_at_voxel`
→ `run_strata` per chunk; far = `surface_class` → `draw_class` per coarse voxel —
but they share the same underlying coarse-field-point-sampling *shape* (spines
S-3 / S-9 family: a coarse summary expressed without area-averaging).

---

## 5. Is it one mechanism with the LOD blobs, or near-field-only?

**One mechanism at the level of root cause; distinct at the level of site.**

- Root cause (shared): the deep facies record is coarse (NEAREST ~460 m) and is
  **point-sampled rather than summarized** wherever it reaches the surface skin.
- Near-field expression (this doc): per-chunk-centre point-sample → the whole
  chunk inherits one deep cell's composition → ~460 m tiles, chunk-quantized
  edges. Code: `collapse.rs:1474-1498`.
- Far-field expression (LOD doc): per-coarse-voxel point-sample of `surface_class`
  → blobs. Code: `collapse.rs:920-988` + `farmesh` stride.

A fix targeting one site (e.g. area-summarizing the deep record for the near
tier) would not automatically fix the other, so they are **distinct sites of one
shared root**. The near-field variant is genuinely near-field-only in its *code
path* (`run_strata` per chunk), even though it is the same *phenomenon family*.

---

## 6. What the render is NOT doing (corroboration)

The render/mesh is faithful and per-voxel, so it does not introduce or amplify
the tiling:

- `top_splat(c, voxel_seed(wx,wy,wz))` is world-anchored per voxel
  (`meshing.rs:345-397`); the `> SPLAT_N` reduction is unreachable in the current
  world (`meshing.rs:359-378`, `:401`). Confirms the brief's T2 refutation.
- The near contents the mesher reads are the per-voxel `ContentsGrid` from
  `chunk_contents`/`material_ids` (`collapse.rs:707-724`, `:521-620`), also
  world-anchored per column. So a voxel at a fixed world position has one
  identity; the tiling is upstream, in **which class the chunk's single deep
  sample assigned that column**, not in how the voxel is drawn.

---

## 7. PROVEN vs INFERRED / needs-live-probe

### PROVEN (static, from the code)

- The whole chunk's strata composition is built from **one** deep-record
  point-sample at the chunk centre, and shared by all 1024 columns.
  `collapse.rs:1474-1498`; surface skin `:1512-1524`; buried `:457-471`, `:573-618`.
- `record_at_voxel` and `regolith_at_voxel` are **NEAREST** (`.round()`) at the
  deep grid; `surface_at_voxel` alone is bilinear. `field.rs:425-433`, `:442-450`,
  `:402-405`, `deep_coords :383-397`.
- Deep cell edge is `DEEP_CELL_M = 460.0`, or `max(extent/DEEP_MAX_WIDTH, 460)`
  for large extents. `field.rs:41`, `:110`.
- All other per-chunk inputs are smooth (bilinear climate `collapse.rs:1196-1215`;
  footprint-mean elevation `:1460-1461`; centre flow energy `:1465-1470`), so they
  cannot produce a hard tile boundary. The province/igneous class steps at the
  ~14.7 km grid. `collapse.rs:1450-1459`, `pregen/mod.rs:40`.
- The member axis is per-voxel and C0-continuous across chunk seams; it varies
  *within* a class only and cannot cross the class boundary the deep sample drew.
  `geology.rs:198-213`, `:653-674`; `collapse.rs:686-705`.
- Therefore composition is uniform within a ~460 m deep cell and switches only at
  the deep-cell grid, chunk-quantized. Directly corroborated by the deep-field doc
  (`field.rs:435-441`) and the far-field test comment (`collapse.rs:2269-2270`).
- The render is faithful/per-voxel and not the source. `meshing.rs:345-397`;
  `collapse.rs:707-724`.
- Same root cause as the LOD blobs (coarse deep-facies field point-sampled), but a
  distinct code site (`record_at_voxel`→`run_strata` per chunk vs
  `surface_class`→`draw_class` per coarse voxel). `collapse.rs:1474-1498` vs
  `:920-988`; cross-ref `docs/audits/2026-07-24-lod-pre-post-visit-diagnosis.md § 3`.

### INFERRED / needs-live-probe

- **Whether the visible squares are ~460 m deep cells or 28.8 m chunk staircase
  edges.** Static code proves composition is *uniform within a 460 m cell*, so a
  genuine grid of *distinctly different 28.8 m squares* cannot come from the class
  stack; it would require the **subtle within-class member-fitness stepping** —
  the chunk-centre formation context (`temp_c/precip/depth_m` baked into each
  event, `geology.rs:665-670`) shifting which member a continuous `u` maps to
  across a chunk line via `GeologySet::select`'s fitness weights. That is a real
  but *second-order* per-chunk effect (same class, different member ⇒ different
  albedo but not "different overall composition"), and it is **not proven to be
  visually dominant**. A live probe settles which grid the user sees: read the
  **dominant surface material per chunk** over a near-field patch at the east
  coast (`world_scan_region`, or the inbound contents inspector) and check whether
  the dominant material changes at **every** 28.8 m chunk edge (⇒ member stepping
  is the visible signal) or **only at ~460 m intervals** (⇒ deep-cell tiles are
  the signal, chunk-quantized edges).
- **The exact phase/alignment of the visible tile.** The deep grid registration
  (`deep_coords`, `field.rs:383-397`) is anchored to the pregen grid centre, not
  to `vx.div_euclid(512)`. A probe reading the deep-cell index
  (`round(deep_coords)`) alongside per-chunk dominant material would confirm the
  tiles are the deep grid, not the locale grid.
- **Whether the east-coast vantage places one deep-cell boundary or several in
  view.** "A grid of squares" implies several tiles visible at once; at a 460 m
  tile this needs a wide/high-altitude downward view. The probe should record the
  camera pose and the number of distinct deep cells intersecting the near field at
  observation time.

*(Per A-5: the runtime-dependent items above are not asserted as fact; they are
the questions a live probe must answer. The core mechanism — per-chunk single
deep-record sample, NEAREST at ~460 m, driving the whole chunk's class stack — is
in the PROVEN set.)*
