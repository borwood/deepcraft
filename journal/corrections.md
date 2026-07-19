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

## 5. "`surface_height_m` under-reports ~7 m — likely a missing detail octave" (2026-07-19)

**Claim** (journal/0004, carried into ROADMAP Observed): the analytic helper
under-reports the voxel surface by ~7 m, "likely a detail octave present in
voxel generation but missing from the heightmap helper."
**Falsified**: there is one field — `block_in_column` calls `surface_height_m`
directly, so a same-column disagreement of meters is impossible *by
construction* (in-column error is bounded to +½ voxel by the center-solidity
rule). The real mechanism is two compounding effects: (1) the solid **top
face** sits up to half a voxel above the analytic height everywhere, and
(2) a body's **footprint spans columns** and rests on the highest of them —
on the chasm walls neighbouring columns differ by many meters. Measured worst
gap 14.49 m; 61% of near-surface columns would embed a body placed at
`analytic + 0.05` (journal/0006).
**Fix**: `true_surface_m` — footprint-max over per-column voxel scans, edits
included; everything that seats a body routes through it (merge `8aafc3a`).
**Lesson**: an analytic generator field and its own voxelization are
different surfaces; never seat a body on the former. Also: the plausible
single-cause story ("missing octave") survived three walks because nobody
priced the footprint; quantify before naming mechanisms.

## 6. Chunk-line family cutover: "cell-stepped climate context flips selection on chunk borders" (2026-07-19)

**Claim** (ROADMAP Observed, walk 8): material families cut hard on chunk
lines because "at N=2 the chunk (28.8 m) equals the S7 column-quantization
cell, so cell-stepped climate context flips selection exactly on chunk
borders."
**Half falsified (mechanism refined).** The pregen climate/provenance **cell**
is `CELL_VOXELS` = 16 384 voxels (512 chunks, ~14.7 km) — *not* the chunk — and
`climate_at` is already **bilinear** between cell centres, so climate is a
smooth field that does not step at chunk borders. Climate is not the culprit.
The half that is right: the quantization cell that *does* equal the chunk at
N=2 is the **chunk-column collapse unit** (`L_COLUMN`, 32 voxels), and the
strata passes run **once per chunk-column**. So every field the passes consume
— the single centre climate sample, the single centre `flow_energy`, the
footprint-mean elevation, the per-chunk selection hash, and the integer-rounded
thicknesses — is piecewise-constant across the whole 32×32 footprint and
uncorrelated with its neighbours. Member identity (and, for the widened roster,
which member of a class) is therefore uniform per chunk and flips on the grid.
It is **selection quantization**, not "cell-stepped climate."
**Fix** (3d mechanic 2, journal/0011): a material-tier **boundary dither** —
per voxel-column the host member is re-selected from a bilinear field whose
corners are the chunk-column selection hashes (C0-continuous across borders),
so a class's member contact wanders like a facies boundary and can fall inside
a chunk. Proven by a transect test (`family_contacts_wander_off_the_chunk_grid`).
The thickness/class-presence quantization (e.g. a sandstone cap appearing via
`flow_energy` rounding) is a *separate* per-chunk artifact, left as a loose end
(ROADMAP Observed) — it needs per-voxel-column context, not member dither.
**Lesson**: "the cell equals the chunk" was true of the *wrong* cell. When a
quantization has several candidate cells (pregen cell vs collapse unit),
measure which one the artifact actually rides before naming the field that
steps.

## 7. "env!(CARGO_MANIFEST_DIR) is a fine way for tests to find the workspace" (2026-07-19)

**Claim** (implicit in dc-host test plumbing since S5): compile-time
`env!("CARGO_MANIFEST_DIR")` locates the workspace for spawning the nested
demo-plugin build.
**Falsified during S9 integration**: agent worktrees share one
`CARGO_TARGET_DIR`; the workspace-feature flavor of the parity test binary
was last compiled inside a since-deleted worktree, so the baked path pointed
at nothing (OS 267 NotADirectory at spawn). Solo reruns used a different
cached feature flavor compiled from main and passed — deterministic per
flavor, masquerading as flaky.
**Fix**: runtime `std::env::var("CARGO_MANIFEST_DIR")` with compile-time
fallback (dc-host tests/common). Standing rule: no `env!`-baked absolute
paths in anything that outlives its compilation directory.

## 8. "B is minutes, paid once" and "erosion is bounded" (2026-07-19)

**Claims** (earth-processes.md § S9 framing, Claude): full-resolution
global deep-time (B) costs "minutes, paid once"; and "bounded processes
(erosion, deposition — not drainage)" can refine regionally.
**Falsified by S9 measurement**: B at Medium on the scalar engine is
**~63–80 minutes + ~3 GiB** (minutes only at Small extent or with ~30×
parallelism — the named flip condition). And only **hillslope** erosion is
bounded (clean 21-cell decay); **fluvial** erosion inherits drainage's
global reach (isolated spikes to 26 cells as reroutes teleport along
receiver chains). C is licensed solely because drainage is decided
coarse-globally.
**Lesson**: cost claims and boundedness claims are measurements, not
adjectives; the halo theorem applies per-process, never to "erosion" as a
lump.

## 9. "A parallel priority-flood could pull B to ~2 minutes" (2026-07-19)

**Claim** (S9-results.md recommendation, carried into earth-processes.md):
the deep-time verdict's flip condition — parallelizing the flood could make
brute-force B (~27 M cells) cost ~2 minutes, where its simplicity wins.
**Falsified by S9b measurement (on this 6-core/12-thread machine)**: the
flood (65–72% of the step, 98.5% at B scale) has **no byte-identical
parallel form** — level-gather reordering breaks fp-sum determinism, and
the open-seam tiled variant diverges up to 110 m at ~0.4% of cells. The
phases that DO parallelize deterministically (route 9×, diffusion 5.7×)
are a bandwidth-bound minority: whole-step speedup 1.18–1.26×, saturating
by 8 threads. Realistic parallel B ≈ 15–70 min vs the 2-min target.
(Also: S9's recorder empty-header estimate was 648 MiB; measured 833 MiB.)
**Standing**: A+C stands. The question reopens only on ~32-core hardware
with a deterministic parallel flood (Barnes spill-graph unbuilt) AND
parallel transport — `examples/deeptime_par.rs --full-b` re-measures it
anywhere.
**Lesson**: "could be parallelized" is a claim about an algorithm's
existence, not its cost under a byte-identity mandate — the determinism
tax must be priced per phase before it prices your architecture.
