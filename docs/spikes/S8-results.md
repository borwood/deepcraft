# S8 — Material volume model storage: results

Status: spike complete, 2026-07-18. Code: `dc-core/src/materials/` (kept —
this is the v0 materials layer over the v1 container, not throwaway),
`dc-core/tests/{s8_properties,s8_measurements}.rs` (spike-grade harness).
Design under test: docs/design/materials.md. Exit criteria met: go/no-go
below, bytes/m³ measured with debris present, sidecar schema v0 specified.

## How to run

- `cargo test -p dc-core --release --test s8_measurements -- --nocapture` —
  all measurement tables below.
- `cargo test -p dc-core --release --test s8_properties` — the property
  suite (500 seeded cases per property).
- Adds `fastnoise-lite` as a dc-core **dev**-dependency only (deposition
  terrain); the library itself gained no dependencies.

## Verdict first: **GO on free-form mixtures** (with one guardrail)

Free-form mixtures survive palette compression under every realistic
deposition process we could throw at them, and the reason is structural, not
lucky: **the eighth quantization caps the state space combinatorially.** A
voxel's debris is an unordered multiset of ≤ 8 eighths, so a locale mixing
*k* materials can produce at most `C(k+8, 8) − 1` distinct states no matter
how smooth or adversarial the mixing gradient is:

| Materials in the locale | Hard cap on distinct states | Observed |
|---|---|---|
| 2 (wind: snow+sand; scree: scree+gravel) | 45 | **exactly 45** (saturated, then flat) |
| 4 (alluvial; the engineered gradient) | 495 | 23 (alluvial), **exactly 495** (gradient) |
| 5 (midden) | 1,287 | 196 |
| 12 (per-eighth uniform noise) | 125,970 | 79,524 — **explosion** |

The design's feared failure mode — "smooth gradients explode distinct-state
counts" — **does not exist for realistic material diversity**. A continuous
4-material gradient engineered to maximize states hit its 495-state ceiling
and stopped. The explosion requires ~8+ materials mixed at per-eighth
randomness, which no plausible deposition process produces (deposition is
episodic: each event lays one or few materials; diversity accumulates in
*layers*, which canonicalization then dedups regionally).

Guardrail: because a hostile plugin or degenerate content pack *could* dump
12-material noise, the region table should carry a soft cap (e.g. 4,096
entries) with nearest-mixture snapping as backpressure. That is a safety
valve, not a design pillar. **Curated mixture recipes are not required for
storage feasibility** — they remain available as a game-design choice
(nameable, learnable strata) on top of free-form storage, not instead of it.

## Sidecar schema v0

Canonical-form definition (what makes interning and palettes work): a
`VoxelContents` is `(shape, structure multiset, pore-fill multiset, debris
multiset)`; each multiset is stored sorted by ascending material id, unused
slots normalized. Equal contents ⇒ equal representation ⇒ equal hash ⇒ equal
encoding; the decoder *rejects* non-canonical bytes, so bytes and values are
bijective. Deposit order inside a voxel is deliberately destroyed — that is
what mixing is (docs/design/materials.md).

`VoxelContents` compact encoding: 2 header bytes (shape 2 bits;
structure/pore/debris lengths 4 bits each) + 1 byte per occupied eighth.
Empty = 2 bytes; a full 8-eighth mixture = 10 bytes. Measured table entries
average 7.3–9.4 B.

**`"materials/slots-v0"`** — per-chunk sidecar on the S3 format-v1 container
(postcard):

```text
[palette: seq of MixtureId (u32 varint)]  -- distinct interned ids, first-appearance order
[indices: PackedIndices]                  -- bits = ceil(log2(P)), len = 32^3, straddle-free words
```

- A palette entry **is** an interned mixture id: chunk storage references the
  region table through exactly the palette discipline S3 built (reuses
  `PackedIndices`, same canonical-padding validation, uniform chunk = 0 bits).
- A chunk whose voxels are all-empty attaches **no sidecar at all** — the
  container bytes are byte-identical to a materials-unaware writer's (tested).
  Debris-free terrain pays zero; readers that don't know the name keep
  working and rewrites preserve it (S3's sidecar rules, already tested there).

**`"materials/mixtures-v0"`** — the region-level intern table:

```text
[count: u32 LEB128]
count x [VoxelContents compact encoding]   -- entry i is mixture id i; entry 0 is EMPTY
```

Decode validates: entry 0 empty, every entry canonical, no duplicates, no
trailing bytes. Ids are first-intern order, so deterministic fills produce
deterministic tables. Where this table *lives* is pinned to S3 open question
7 (region-file grouping): one table per region file, alongside its chunks.
Until region files exist it is stored/measured standalone.

## Deposition simulator

Headless, seeded, deterministic (splitmix64 streams; no wall clock). Region:
8×8 chunk-columns (256×256 voxel columns at 0.9 m), S1 noise parameters
(hills + ravine, read-only copy from dc-client/src/worldgen.rs), region
origin auto-scanned to include both cliffs and open ground (landed at voxel
(−512, −512), terrain seed 1337, sim seed 0x585eed). Deposits are per-column
stacks of eighth-layers starting at the terrain surface; voxelization slices
stacks on the world grid, so mixtures arise from real layer/voxel
misalignment. Four processes, each with plausible local rules:

- **wind**: snow/sand storm runs; sheltered (hollow) columns accumulate by
  exposure vs neighborhood, crests erode downwind; repose relaxation.
- **rockfall/scree**: cliff-base sources (terrain drop > 16 eighths to a
  neighbor) emit scree/gravel that walks downhill to a stable column;
  angle-of-repose relaxation builds the talus apron (repose threshold is
  cohesion-dependent per material).
- **midden**: flattest central column hosts a settlement; each pass dumps 6
  eighths of potsherd/ash/bone/leaf-litter/loam with 2-column scatter.
- **alluvial fan**: steepest column feeds a flow carrying
  gravel/sand/silt/clay; energy decays along the downhill path and each
  grain settles when energy drops below its grain-size-dependent threshold —
  graded sorting by flow distance, coarse near the mouth, clay at the toe.

All numbers below from the recorded run (Windows dev box, release,
single-threaded; whole suite 4.6 s).

## Measurements

Baselines (S3): base block grid 0.144 B/m³ region average, 0.173 B/m³ for
mixed chunks, 2.74 B/m³ raw dense. A naive dense 8-slot-bytes-per-voxel
material grid would be 10.97 B/m³ on every chunk. "B/m³" below is sidecar
bytes over the volume of chunks that have any deposit (all other chunks pay
0); "+table" amortizes the region mixture table over the same volume.

### Each process alone (fresh field)

| Process | Deposit eighths | Deposit chunks | Table entries (bytes) | Distinct/chunk min/med/max | Bits | Sidecar B | B/m³ | +table |
|---|---|---|---|---|---|---|---|---|
| wind, 96 passes | 288,451 | 107 | **45** (331 B) | 2 / 22 / 45 | 6 | 427,120 | 0.167 | 0.167 |
| rockfall, 200 passes | 1,391,800 | 60 | **45** (331 B) | 5 / 34 / 44 | 6 | 540,746 | 0.377 | 0.378 |
| midden, 400 passes | 2,400 | 2 | 196 (1,623 B) | 112 / 141 / 141 | 8 | 8,734 | 0.183 | 0.217 |
| alluvial, 300 passes | 3,000 | 2 | 23 (167 B) | 22 / 23 / 23 | 5 | 6,649 | 0.139 | 0.143 |

- Wind and scree both hit **exactly 45 table entries** — the complete
  multiset space of their 2 materials. Palette-friendliness is not
  statistical luck; it is the cap.
- Alluvial's grain sorting actively *reduces* local diversity (each reach of
  the fan is nearly one material): 23 states from 4 materials, the
  best-behaved process measured.
- The midden is the diversity-richest realistic deposit (5 materials, point
  mixing): 196 of 1,287 possible states, still an 8-bit palette.

### Marginal cost per additional pass block (wind, same field)

| After passes | Table entries | Distinct/chunk med/max | Bits | Sidecar B/chunk | B/m³ |
|---|---|---|---|---|---|
| 24 | 45 | 9 / 39 | 6 | 2,784 | 0.117 |
| 48 | 45 | 14 / 41 | 6 | 3,259 | 0.136 |
| 72 | 45 | 18 / 44 | 6 | 3,723 | 0.156 |
| 96 | 45 | 22 / 45 | 6 | 3,992 | 0.167 |

**Marginal state cost of more deposition ≈ zero after saturation** (45
entries at pass 24, 45 at pass 96). Byte growth is purely coverage: more
chunks touched, more per-chunk palette entries drawn from the same fixed
regional pool. Deep-time worldgen can run deposition as long as it likes.

### Marginal cost per additional process (one field, stacked)

| Field state | Table entries (bytes) | Distinct/chunk max | Bits | Sidecar B | B/m³ | +table |
|---|---|---|---|---|---|---|
| wind | 45 (331 B) | 45 | 6 | 431,090 | 0.167 | 0.167 |
| +rockfall | 227 (1,985 B) | 133 | 8 | 910,192 | 0.284 | 0.285 |
| +midden | 501 (4,242 B) | 192 | 8 | 914,112 | 0.286 | 0.287 |
| +alluvial | 597 (5,093 B) | 192 | 8 | 916,553 | 0.286 | 0.288 |

Stacking processes costs the union of their states **plus interface
cross-mixtures** (wind-drift over talus, refuse into snow): 597 combined vs
309 summed-solo — cross-terms roughly double the table but stay in the same
order of magnitude; palettes stay at 8 bits. The fattest chunks in the
combined field (scree/wind interfaces): 192 mixtures, 8 bits, 6.5 KB,
0.27–0.47 B/m³.

### Pathological cases (engineered)

| Case | Table entries (bytes) | Distinct/chunk med/max | Bits | B/m³ | +table |
|---|---|---|---|---|---|
| 4-material continuous gradient, 4 voxels deep, per-eighth roll | **495** (4,160 B) | 148 / 397 | 9 | 0.351 | 0.352 |
| 12-material uniform noise, 4 voxels deep, per-eighth roll | **79,524** (749,119 B) | 853 / 4,337 | 13 | 0.709 | 0.940 |

- The gradient case is the design's stated fear and it is **capped**: 495 is
  the entire multiset space of 4 materials (sizes 0–8). Continuous mixing
  cannot exceed it, at any region size, forever.
- The 12-material noise case is the real explosion: ~1 state per deposit
  voxel until the 75,582 full-voxel multisets are exhausted, 13-bit
  palettes, a 749 KB region table that would keep growing with area. This is
  the case the table cap + snapping guardrail exists for. Even here the
  absolute cost (0.94 B/m³) stays 2.9× under the raw block grid — the model
  degrades, it does not detonate.

### Cost structure (what the bytes actually are)

The dominant cost is the **per-chunk index array**, not the table: once a
chunk holds any debris, it pays 32,768 indices × palette bits (4 KB at 1
bit… 24 KB at 6 bits before postcard, in practice 2.8–9 KB measured because
most deposit chunks are thin drapes with small palettes). Table entries cost
~7–9 B each and are shared region-wide. Deposit chunks ran 0.14–0.47 B/m³ —
i.e. an extra ~1–3× the S3 mixed-chunk block-grid cost on the chunks that
have deposits, ~38× cheaper than a naive dense slot grid, and zero on
everything else.

## Extraction prototype — outcomes

`extraction_sequence(contents, damage)`: sustained typed damage yields
materials in ascending per-type resistance, ties by ascending id. Verified
by 500-case seeded property sweeps (plus targeted unit tests):

- **mass conservation**: yields equal the eligible pools exactly, under all
  5 damage types;
- **ordering**: strictly ascending (resistance, id) — the design's
  leaf→sand→gravel dig sequence falls out of the property values;
- **sieve separates by grain size**: enforced *by registry invariant* (sieve
  resistance ≡ grain size, tested), so the general ordering rule is also the
  grain separator — no special case;
- **structure slots are never yielded by debris damage** (dig/cut/sieve);
  smash/chop reach structural fill. A material present both as pore fill and
  debris pools into one yield.

Not modeled (deliberately): tool-dependent destruction ("a careless pick
pulverizes the potsherds") — that is game design layered onto this ordering,
and it needs the damage-application loop, not the storage model.

## Stratification — outcomes

`stratify(contents, time_undisturbed)`: pure, integer-only (bit-identical
across platforms), never ticked. Of `n` debris eighths, `k(t) = n·t/(t+τ)`
(τ = 1,000, saturating to `n` at 16,000) have settled into density-sorted
bands at the bottom; the rest is one mixed band on top. Property-tested:
deterministic; mass-conserving; `k` monotone in `t`; **bands refine, never
rearrange** (the settled prefix at `t₁` is a prefix of the settled region at
`t₂ > t₁`); fully banded at saturation; only debris stratifies. Agitation =
resetting the input clock; worldgen deposits pass a huge time and arrive
banded — both behaviors are free consequences of the signature.

## LOD downsample rule — outcomes

The S3 `DownsampleRule` reduces `[Block; 8]` and cannot see sidecars, so
material LOD gets a parallel pluggable rule with the *same octant/cell
geometry*: `DominantClassDebrisAware` votes occupancy in **eighths** (parent
non-empty iff ≥ 32 of 64 child eighths occupied, ties→solid, mirroring
MajorityNonAir), picks the dominant class (structural vs loose) by volume,
folds the minority in rather than dropping it (loose survives as pore fill
under structure dominance; rubble joins the mix under debris dominance), and
apportions parent slots by largest remainder — so the parent mixture is the
child mixture at 1/8 resolution, canonical by construction, interned into
the same region table.

Composition with the S3 pyramid is proven by test over two levels: on a
scene inserted into a real `LodPyramid` (MajorityNonAir), wherever material
LOD is non-empty, block LOD is solid — ≥ 32 eighths forces ≥ 4 non-empty
child voxels, which is exactly MajorityNonAir's threshold; the invariant
survives L0→L1→L2 by induction. Material LOD never claims volume the block
pyramid dissolved. Derivation is deterministic (chunk bytes and table bytes
equal across rebuilds).

Known and accepted: debris drapes thinner than half a cell vanish at L1,
exactly like 1-voxel grass under MajorityNonAir — the distant view loses
surface dusting. The render-blend prototype (heightmap-blended debris
layers) is still owed and is a dc-client concern.

## Open questions

1. **Sparse sidecar encoding.** Thin drapes pay for the full 32,768-index
   array (the dominant cost above). A y-cropped or sparse `(index, id)`
   section for chunks with < ~5% coverage would likely cut deposit-chunk
   bytes 3–5×. Worth doing when the save layer lands; schema-v0's named
   versioning makes it a non-breaking `slots-v1`.
2. **Region table lifecycle.** The table only grows; digging away the last
   voxel of a mixture strands its entry. Rewrite-time compaction (re-intern
   live ids, remap chunk palettes) is straightforward but undesigned, and
   belongs with the region-file grouping (S3 OQ 7).
3. **The guardrail policy.** Cap-and-snap needs a distance metric on
   mixtures (per-material eighth deltas?) and a decision on whether snapping
   is silent or surfaces to gameplay (curated-recipe strata as the visible
   form of the cap). Numbers here say the cap will not trigger under normal
   play.
4. **Pore packing rules unexercised.** The representation admits pore fill
   and the constructor enforces capacity, but the deposition sim never packs
   pores (overflow rule from the design: free slots → packable pores → stack
   above). Needs the fluid/packing spike; property tests cover the
   representation only.
5. **Structure fill provenance.** Heterogeneous structural fill
   (rubble/breccia) is representable and LOD-handles correctly, but nothing
   yet *produces* it (compaction of debris under overburden closes this loop
   in the design; untouched here).
6. **Stratified-view ↔ extraction interaction.** Mining a banded deposit
   should yield clean bands (band order), a fresh mix by resistance. Both
   halves exist; the dispatcher (when does a dig read the stratified view vs
   the extraction order?) is gameplay logic that does not exist yet.
7. **Eighth-alignment sensitivity.** Mixtures at layer boundaries depend on
   where the world voxel grid slices the stack; a worldgen that deposited in
   voxel-aligned bands would produce *fewer* states than this sim (which
   used worst-case continuous alignment). Numbers above are therefore
   conservative.
