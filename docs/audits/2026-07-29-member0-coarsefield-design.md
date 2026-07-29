# Member #0 — CoarseField as the base reconstruction kernel (design pass)

**Read at `4f773cc`.** The first member design pass under `refinement.md`'s ratification
qualifier (*"first members will require their own dedicated design attention… revisited in
light of this design, not taken as wherever they landed prior"*). Produced by a read-only
design agent; verbatim report body below. **Dated artifact — immutable body, mutable
header.** Consumed by the member-#0 ratification conversation (main session, 2026-07-29
evening).

Null re-verified at `4f773cc`: `grep -rn "CoarseField|DitherSource|sample_dithered"
--include=*.rs crates/` excluding `coarse.rs` → 12 hits, all prose. Zero production
callers. Only `ShareVec<N>` is consumed (`lithology.rs:551,565,575,615`).

---

## 1. What member #0 IS — three kernels, not one

`refinement.md` § 3's "base reconstruction = CoarseField sampling" fuses three separable
things:

| kernel | what it does | who calls it | today |
|---|---|---|---|
| **K1 · sample / sample_dithered** | fine read of a coarse cell at a voxel — initializes the working state | the refinement runner, per voxel/column | hand-rolled at 2 sites |
| **K2 · summarize** | *coarse* read — area-weighted mix over a region | the **far/LOD synthesizer**, per node | uncalled; consumer in `dc-client` (`farmesh.rs:1485`) |
| **K3 · midpoint refinement** | synthesizes sub-cell detail the field does not contain, amplitude gated by a coarse quantity | `collapse.rs::lattice` (`:1089-1143`) | built, unnamed, **not a CoarseField move** |

K1 and K2 are two *executors*, not one kernel at two resolutions (`coarse.rs:37-43`;
`octree-substrate.md:96-106`). **Proposed ruling: member #0 is K1 only.** K2's home is the
octree node contract (MM-4); K3 is move C (MM-5).

**`DitherSource` placement:** invoked per voxel, twice, inside `sample_dithered`
(`coarse.rs:489,500`) → per-voxel draw = engine primitive (granularity table). "Caller-owned
entropy" is a **crate-layering seam** (dc-core holds no RNG — `coarse.rs:51-56`), not a
plugin seam. **`DitherSource` is engine-internal with engine impls (white / coherent /
octaves); a pack SELECTS a source by declared id, never supplies one. P6 is engine work.**
The concrete impl already exists: `draws.rs::interp_corner_field` (coherent) + `Draws::unit`
(white). API friction (one salt dimension vs domain+tag) noted, not blocking.

## 2. The adoption path under the bones

**The old P4 plan is superseded on its mechanism, not its target.**

### 2a. Far site (U22) — clean in-place kernel swap; the honest first slice

`collapse.rs::surface_class` (`:900-968`) is `ShareVec::draw` written by hand over a NEAREST
cell. Conversion: shares → `CoarseField<ShareVec<6>>` (six exact classes per
`deep_class_of_species`, `geology.rs:415-424`); `draw_class` + `interp_corner_field` →
`sample_dithered` with a `Coherent` source. **The cake law arrives free** (step 1 dithers
which cell wins the contact — `coarse.rs:487-498`; law-tested at `coarse.rs:799`). **Cost
neutral**: `sample_dithered` reads four cells for weights, shares from ONE cell. `draw_class`
and its unbiasedness test retire *into* `ShareVec::draw`.

### 2b. Near site (U3) — the plan does not survive contact with the record

U3's site is `collapse.rs:1406-1410` (ONE chunk-centre NEAREST sample) → `:1440` (ONE
`StrataRec` per chunk) → `:1455-1467` (all 1024 columns). Three stacked defects: chunk-centre
point sample · NEAREST · one record per chunk with nowhere for a per-column answer to land.

**`CoarseField<T>` cannot host it**: `from_cells`/`stencil` are `impl<T: Copy>`
(`coarse.rs:360`); `DeepStrata` is non-`Copy` (`recorder.rs:280`) and the largest resident
structure — `CoarseField<DeepStrata>` is a type error today, a residency catastrophe if
forced. The legal move (membership dither, generic in `T`) exists as step 1 of
`sample_dithered` and is **not separately callable** (MM-1).

Reshaped adoption: per chunk enumerate touched deep cells (≤ 9, typically 1) → `run_strata`
once per touched cell → per **voxel column**, seeded bilinear membership dither picks which
record skins it → `ColumnRec.strata` becomes `records: Vec<StrataRec>` + `per_column_idx`.
Expensive pass stays per-cell; only the cheap index is per-voxel. Blast radius: `column()`,
`ColumnRec`, `material_ids`, `mixed_at`, `surface_voxel_contents`, `generate_chunk*` —
every golden moves. Cost up to 9× `run_strata` on boundary chunks, unmeasured.

### 2c. Framing

Adoption IS the base-reconstruction step of a runner — **but the runner does not exist, and
building it is not this slice.** Seam-first: far site first (no new engine API), then near
site (needs MM-1 + MM-3). `collapse.rs` touched twice on purpose — the old independence
argument survives as *sequencing*.

## 3. U22 / U3 re-verified

- **U22** (user, 2026-07-22, "the cake observation"): fix holds, unchanged and exact —
  `sample_dithered` step 1 is the cure the entry itself named. ✅ acceptance test.
  ⚠ Its named sibling (member-dither guillotine) will **NOT** be fixed by 2a:
  `dithered_member` (`geology.rs:936-957`) re-picks per column but under **chunk-centre**
  formation context — different site, different cause, still unexamined.
- **U3** (user, 2026-07-24; reference pose feet `71291.7, 372.1, −2420.9`, yaw `21.9968`,
  pitch `−1.5475`): target holds; fix changes shape (§ 2b). ⚠ **Unpriced risk:** the
  2026-07-24 audit's own INFERRED section says the visible squares may be the **28.8 m
  member-fitness stepping**, not the 460 m tiles — never probed. If so, a perfect § 2b fix
  leaves the checkerboard on screen. Probe FIRST (`palette_quant_tour`, already
  `test = true`, one flag away).

## 4. Missing machinery (the "cautiously" clause, discharged)

- **⚠ MM-1** — Move B's step 1 (membership dither, generic in `T`) not separately callable;
  the near site needs `sample_source_cell(pos, src, salt) -> Option<&T>`. Tension named:
  returns a cell payload — not the forbidden raw read (the caller cannot choose the cell)
  but one refactor away, and the `compile_fail` proof won't catch it.
- **⚠ MM-2** — `CoarseField<T>` is eager/owning/`T: Copy`; both sites want borrowed or
  lazy. Cheap escape for the first slice: per-chunk 4×4 local field (768 B, weights
  bit-identical away from clamps). Do NOT grow a borrowed/lazy variant until a second
  consumer demands it (A-4).
- **⚠ MM-3** — the working sub-cell state has no declared type (`ColumnRec` + `StrataRec` +
  `ColumnFill` + `SurfaceSample`, nothing named as "what operators transform"). § 2b is the
  moment it must be written — owed, not invented silently inside a defect fix.
- **⚠ MM-4** — `summarize` has no home in the bones: its only consumer is the far/LOD
  synthesizer (second executor the bones don't model; cross-crate into `dc-client`). Either
  the bones grow a far-field/synthesis clock or `summarize` is reassigned to the octree
  node contract. **User call.**
- **⚠ MM-5** — a THIRD coarse→fine move exists, oldest and unnamed: `collapse.rs::lattice`
  midpoint jitter — synthesizes detail, amplitude from a recorded cause. Legal under S-4,
  **not expressible in the two-move vocabulary**; all sub-460 m relief comes from it. Call
  it **move C — bounded stochastic detail synthesis**. Recommend: narrow member #0 to the
  material/record axis; file move C with a named heir.
- **⚠ MM-6** — `Registration` 2D is sufficient for #0 (record grid is 2D), NOT for #1
  (flux/head are 3D face quantities at half-integer positions; heir = member #1). **Units
  hazard**: docs say world units, sampling is voxels; the type's own test models 460 as
  voxels while `DEEP_CELL_M` is metres — an 11 % error waiting. Cure: `DeepField` hands out
  its own `Registration` derived from `deep_coords` (`field.rs:720-734`).

## 5. First build slice (proposed)

**Far site only: `surface_class` behind `CoarseField<ShareVec<6>>` + the dc-worldgen
`Coherent` DitherSource.** Zero new engine API; the site the type was extracted from;
discharges U22 end-to-end; lands P6's socket impl.

Contents: ① `Coherent` impl wrapping `interp_corner_field` (empties half the spines § 3
DitherSource row) · ② `DeepField` exposes `Registration` (voxels, from `deep_coords`) ·
③ per-chunk 4×4 local field of top-0.9 m class shares · ④ `surface_class` →
`sample_dithered`; `draw_class` + test retire into `ShareVec::draw` · ⑤ same-commit doc
updates (spines § 3, `collapse.rs:890,949` comments, graph P4/P6).

Acceptance: goldens move, re-capture with the why (scratch-pad rule) · keep
`far_surface_class_dither_splits_multiclass_deep_cells` and ADD the cake tripwire (each
frontier cell's minority appears on both sides with monotone fraction shift — scale-free) ·
measured `coarse_surface` throughput (far tier is per-frame; expected neutral, measured not
asserted) · `palette_quant_tour` caption is part of the diff if it goes stale.

NOT in slice: U3/near path (MM-1+MM-3) · `summarize` (MM-4 ruling) · move C · the
working-state type · `collapse.rs` decomp. **Continuation slot:** (a) the checkerboard
probe (460 m tiles vs 28.8 m member stepping); (b) MM-1 + MM-3 + the near-path restructure.

## 6. Law checks

- **Law 2**: adoption path clean (`argmax` not in it; `classify` is per-voxel over per-voxel
  mixture; downsample rules run fine→coarse). ⚠ Residual: `coarse_surface` returns one
  `Block` painted across a `level_stride` footprint (1.8–14.4 m) — the smaller square that
  remains after this slice; owned by MM-4. ⚠ Trap for the near slice: membership dither
  evaluated per CHUNK instead of per COLUMN trades a 460 m square for a 28.8 m one.
- **Law 1**: clean — expression varies only with recorded shares + position noise; the far
  field renders whatever the record holds, including nothing.
- **Law 3**: `summarize`'s area-weighted-mean contract IS the conservation statement
  (already tested). ⚠ Do NOT "fix" `regolith_at_voxel` NEAREST→bilinear here: `H` and
  `record_at_voxel` are the two terms of a difference (`field.rs:749-758`); interpolating
  one and not the other leaks/invents loose material at every boundary — a Law 3 violation
  dressed as an S-4 fix. Move both terms or neither.

## 7. Doc defects (dispositions by main session, same day)

1. `dependency-graph.md` placed P6 in the pack table AND called it engine four rows later —
   **fixed 2026-07-29 (P6 → engine, per the granularity table + refinement.md § 3).**
2. `docs/audits/2026-07-24-palette-quant-generation-diagnosis.md` had no staleness banner;
   citations drifted (substance verified correct at the drifted addresses) — **banner
   stamped 2026-07-29.**
3. `spines.md:1432` CoarseField row cites `collapse.rs:890,949` — both still exact.
4. The audit's INFERRED item (460 m vs 28.8 m checkerboard) unprobed since 2026-07-24 while
   load-bearing for U3's acceptance — **re-filed in ROADMAP Observed 2026-07-29.**
