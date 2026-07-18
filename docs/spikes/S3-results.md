# S3 — LOD-aware chunk storage: results

Status: spike complete, 2026-07-18. Code: `dc-core/src/{palette,format,lod,column}.rs`
(kept — this is the v1 storage layer, not throwaway), `dc-client/src/farmesh.rs`
and `--bench-storage` (spike-grade). Exit criteria met: chunk format v1 spec
below, size/derive costs measured at the S1 scale (N=2, 0.9 m voxels), far-mesh
path renders in the S1 skeleton.

## How to run

- Interactive: `cargo run --release -p dc-client` — full detail to 128 m, LOD
  rings out to 1.2 km. S1 controls unchanged (click/Esc, WASD, F, Space, 2/3/4).
- Headless S3 measurements: `cargo run --release -p dc-client -- --bench-storage`
- S1's `--bench-scales` still works and is untouched.

## Chunk format v1 spec

### Palette compression (`dc_core::palette`)

Per chunk: a **palette** of distinct block states (ordered by first appearance
in index order — compression is deterministic, equal contents ⇒ equal bytes)
plus one palette index per voxel, bit-packed at minimal width:

| Palette size P | Bits/voxel | Example |
|---|---|---|
| 1 (uniform: all-air, all-stone, …) | **0** — no index words at all | 100 of the 600 bench chunks |
| 2 | 1 | stone/air boundary chunks |
| 3–4 | 2 | typical surface chunk |
| 5–8 | 3 | full block variety (S1 has 5 blocks) |
| 257–512 | 9 | future data-driven registry; packing layer tested past 256 |

Packing is **straddle-free** (Minecraft 1.16-style): each `u64` word holds
`floor(64/bits)` indices, an index never crosses a word boundary, padding bits
MUST be zero (validated on decode — the encoding is canonical). Voxel order is
the dense chunk's `x + z*32 + y*1024`. Index width ceiling is 15 bits
(32768 = one distinct state per voxel).

`PalettedChunk` supports point reads (`get`) without decompression; dense↔
paletted conversion is identity-round-trip tested.

### Container (`dc_core::format`), postcard-encoded

```text
[version: u16 varint]      -- decoded FIRST; anything != 1 is a clean error
[voxels:  PalettedChunk]   -- palette (block ids) + bits + len + packed words
[sidecars: seq of (name: String, data: bytes)]
```

- **Version field.** `decode` reads the leading varint before touching the
  payload; a v1 reader refuses v2 files cleanly, a v2 reader can dispatch per
  version. Tested by patching the version byte.
- **Sidecars — the growth hook.** Named opaque byte sections, meaning assigned
  by name only (`"area/thing-vN"` convention; a sidecar's payload versioning
  lives in its name). This is how v1 grows *without a breaking rewrite*: the
  per-voxel material model under design (mixed granular materials, up to 8
  material slots/voxel) ships as e.g. `"materials/slots-v0"` sidecars next to
  the v1 voxel payload — readers that don't know it keep working. Rules,
  both enforced by test:
  - reading a container with an unrecognized sidecar MUST NOT fail;
  - rewriting preserves unrecognized sidecars byte-for-byte.
- Payloads are structurally validated on decode (palette non-empty/duplicate-
  free, bit width matches palette, word counts, index range, canonical
  padding); corrupt/truncated input errors, never panics.

### LOD pyramid (`dc_core::lod`)

Octree-style, cubic: level-L+1 chunk P (32³ voxels of size `2^(L+1)`)
summarizes the 2×2×2 level-L chunks `2P + {0,1}³`; each L+1 voxel summarizes an
aligned 2×2×2 cell of L-voxels (each cell lies inside exactly one child).
Levels 0–4 supported (L4 chunk = 460.8 m cube at N=2).

**Downsample rule — pluggable (`DownsampleRule` trait), default
`MajorityNonAir`:**

- output voxel is non-air iff ≥ 4 of 8 child voxels are non-air (ties → solid,
  so 1-voxel floors/walls survive downsampling);
- representative block = most frequent non-air child, ties broken by **higher
  block id** (later-registered = more specific/surface-y: a 4-grass/4-dirt
  cell reads as grass, so distant surfaces stay green).

**Incremental:** insert/remove dirties only ancestor positions; derivation is
lazy and memoized. Changing one L0 chunk re-derives ≤ 1 chunk per level
(asserted by a derive-counter test). A uniform-children fast path makes deep
all-stone/all-air derivation near-free.

**Coverage vs knowledge** (load-bearing distinction, found the hard way): a
level-0 voxel is *known* iff some **inserted** chunk's cube covers it —
full-res inserts, or LOD chunks inserted directly at level ≥ 1
(`insert_lod_chunk`, the disk-loaded-LOD path). Derived chunks summarize
inserted data but never extend knowledge: a coarse chunk derived from one
loaded child does not make its seven unloaded siblings read as "known air".

### Per-column summaries (`dc_core::column`)

Schema (`ColumnInfo`), per (x,z) level-0 column over a caller-bounded y range:

| Field | Meaning |
|---|---|
| `top_solid_y: Option<i64>` | heightmap: top voxel of highest solid cell found |
| `resolution: u8` | LOD level that answered; surface exact to `2^resolution` voxels |
| `min_solid_y` / `max_solid_y` | lowest/highest solid seen in the range |
| `sky_exposed: bool` | everything above the surface was *known* air |
| `fully_resolved: bool` | false iff any part of the range had no resident data |

Built lazily (`ColumnSummaries` cache) by scanning downward through
`best_block` — the finest resident data per position — consuming whole cells
at whatever LOD resolution answers (a coarse region costs one probe per `2^L`
voxels). `open_air_below(x, z, y_start, y_min)` returns the topmost known-air
cell below a height: the adaptive-load-volume probe ("does this column keep
going down?"), tested against an overhang encoded purely in LOD-2 data with no
full-res chunks resident.

### Skylight query contract

Stated here as the spike's exit requirement; also in `column.rs` module docs:

1. A sky/light query consults ONLY (a) chunks already resident at some LOD
   level and (b) cached column summaries. It never loads, generates, or walks
   "the column above".
2. Every query is bounded to a caller-supplied `[y_min, y_max)`; no unbounded
   vertical scans exist anywhere.
3. Unknown volumes are **non-occluding** (optimistic sky) and poison the
   answer's `fully_resolved` flag. Consumers needing certainty schedule LOD
   derivation or widen the range — they cannot force loads from inside a
   light query.
4. Answers carry the resolution they were derived at; a consumer choosing a
   coarse answer knows it is coarse.

## Measurements

Headless `--bench-storage`, release, single-threaded, seed 1337, N=2 (0.9 m
voxels), S1's 256×128×256 m region (600 chunks), Windows 11 dev box,
2026-07-18.

### Palette compression

| Class | Chunks | Encoded bytes | B/chunk | B/m^3 | vs raw 2.74 B/m^3 |
|---|---|---|---|---|---|
| all-air (uniform) | 100 | 900 | 9.0 | 0.0004 | 7272.5x smaller |
| mixed (surface/caves) | 500 | 2,062,302 | 4124.6 | 0.1727 | 15.9x smaller |
| **all** | 600 | 2,063,202 | 3438.7 | 0.1440 | 19.0x smaller |

raw dense: 37.5 MiB (39,321,600 B); compress 0.467 s, encode 0.005 s.

Notes:

- **0.144 B/m³ overall — 19x under the raw baseline** (2.74 B/m³), before any
  general-purpose compression (zstd on top would bite further; not measured).
- A uniform chunk is **9 bytes** total (version + 1-entry palette + empty
  words + empty sidecars).
- **There are no all-stone chunks in this region**: the 3D cave noise carves
  everywhere, so every subsurface chunk has an air pocket somewhere and lands
  in "mixed" (2-entry palette, 1 bit/voxel, ~4.1 KiB ≈ 6.4% of dense). The
  all-uniform-solid collapse is unit-tested but the *bench* class is empty —
  deep-region savings in this terrain come from 1-bit chunks, not 0-bit ones.

### LOD pyramid derive time per level

| Level | Chunks | Derive time | ms/chunk |
|---|---|---|---|
| 1 | 108 | 270.9 ms | 2.509 |
| 2 | 32 | 62.6 ms | 1.955 |
| 3 | 8 | 16.6 ms | 2.078 |
| 4 | 8 | 9.1 ms | 1.139 |

Insert of 600 level-0 chunks: 0.001 s. Whole pyramid over the region:
~0.36 s, dominated by level 1 (it reads 8 full-res children per chunk through
palette point-reads). ~2.5 ms per L1 chunk ≈ one full-res chunk gen+mesh, and
it is incremental — an edited chunk costs ≤ 4 re-derives in the background.

### Column summaries (102,400 columns, y ∈ [−128, 64))

| Source | Build time | us/column |
|---|---|---|
| full-res + LOD | 2.04 s | 19.91 |
| LOD-1 only (no full-res loaded) | 1.31 s | 12.78 |

LOD-1 surface vs full-res surface: 93.5% within 1 voxel, 94.6% within 2,
95.6% within 4; max deviation 39 voxels (102,400 columns).

- ~20 µs/column lazy build means summaries are effectively free on demand;
  nothing needs to precompute them.
- The LOD-1-only row is the load-bearing claim: **surface and open-air queries
  answer without any full-res chunk resident**, at 2-voxel resolution, 36%
  cheaper. The disagreement tail (max 39 voxels) sits at chasm walls and cave
  mouths where the majority rule moves which cell is "top" — acceptable for
  load-volume and far-light decisions, which is all LOD answers are for.

### Far mesh, 1.2 km field (viewer at the interactive spawn, (0, −80, 0) m)

| Level | Ring (m) | Chunks | Gen time | Mesh time | Triangles |
|---|---|---|---|---|---|
| 1 | 112-256 | 340 | 0.39 s | 0.35 s | 2,612,474 |
| 2 | 256-512 | 316 | 0.33 s | 0.36 s | 3,810,750 |
| 3 | 512-1024 | 324 | 0.32 s | 0.44 s | 6,001,734 |
| 4 | 1024-1200 | 28 | 0.03 s | 0.05 s | 595,232 |

Far field total: 13,020,190 triangles, 2.26 s gen+mesh (single-threaded).
Full-res same annulus (extrapolated from S1's 30.8 tris/m²): ~138 M triangles
(**11x more**), ~302,762 3D-shell chunks = **18.5 GiB** raw dense
(vs zero retained bytes — far chunks keep meshes only), ~221 s mesh time
(**98x more**).

- Interactive: the field streams at 3 chunks/frame (~2.5 ms each), filling in
  a few seconds while staying responsive; ring membership has 48 m hysteresis.
- The triangle counts *rise* with ring 1→3 because rings are 3D spherical
  shells (cubic-chunk pillar: the far field extends below the viewer too) and
  the deep shells are full of coarse-sampled **cave surface** that is sealed
  underground and never visible. This is measured waste — see open questions.

## Far-mesh data source: what's proven where

The dc-core pyramid derives LOD from real chunk data and is measured above.
The *client's* far rings sample the deterministic generator at coarse
`VoxelScale`s instead (a level-L voxel is just a `2^L`-sized voxel — S1's
meter-space noise makes this exact for the terrain surface): without a save
layer, driving 1 km of rings from the pyramid would require full-res
generating the entire 1 km first (~5 min single-threaded), which is exactly
what LOD exists to avoid. Production path: derive pyramid levels at
save/unload time, far mesher reads cached LOD (`insert_lod_chunk` is that
ingest path, tested). Coarse sampling vs derived data differ only in
sub-cell block choice (majority vote vs point sample) — both are 32³ paletted
chunks through the same mesher, so the streaming/seam/perf conclusions hold.

## Seam approach (chosen tradeoff: overlap + downward bias)

- The LOD-1 ring starts **16 m inside** the full-detail radius: the boundary
  band is double-covered, so there is never a sky-gap, only redundant
  geometry.
- Far meshes are biased **down by half a coarse voxel**, so where the two
  disagree the coarse surface pokes *under* the fine surface instead of
  through it.
- Cost: far terrain sits up to ~0.9 m (L1) to ~7 m (L4) low; occasional coarse
  corners still show in the overlap band; ring-to-ring boundaries (no overlap,
  same generator both sides) can show one-coarse-voxel cracks. At ≥ 112 m
  these read as terrain noise. Skirt geometry or true stitching is the
  polished fix; deliberately out of spike scope.

## Failure modes / open questions

1. **Sealed-cave far geometry.** ~2/3 of far triangles are underground cave
   surfaces that can never be seen. Column summaries already know the surface;
   a far-mesh pass should skip cells with no sky path (or mesh only within N
   voxels of the heightmap) — likely a 3–5x far-triangle cut. Design the far
   mesher against summaries, not raw LOD.
2. **Majority rule at cliffs.** The 5–6% surface-disagreement tail (max 39
   voxels) clusters at chasm walls/cave mouths. Fine for rendering; if the
   statistical sim tier reads LOD heightmaps for e.g. pathfinding costs it
   will need the `resolution` field to know when to distrust them.
3. **Downsample rule vs future materials.** MajorityNonAir votes on block ids.
   With 8 material slots/voxel (sidecar model), "majority" needs a real
   definition (dominant material by volume?). The rule is pluggable and the
   sidecar hook exists, but the LOD story for mixed materials is undesigned.
4. **Pyramid memory policy.** The pyramid stores every derived level forever;
   fine at 600 chunks (levels 1–4 add ~1/7 of level-0), but a streaming world
   needs eviction (LRU per level?) and persistence of derived levels in the
   container (as `"lod/level-N"` sidecars of a region file?) — not designed.
5. **Mixing direct LOD inserts with later full-res inserts** in one subtree:
   a level-0 insert dirties ancestors, and re-derivation from partial children
   *replaces* richer direct-inserted LOD data. Correct for "loaded region
   upgrades its LOD", wrong if disk LOD was authoritative over unloaded
   siblings. Needs a per-chunk provenance bit when the save layer lands.
6. **Optimistic sky is a policy, not a truth.** Unknown volumes don't occlude;
   `fully_resolved: false` is the only warning. A consumer that ignores the
   flag will light caves as if open to sky until data arrives. The contract
   makes this explicit; the lighting spike must decide re-light-on-load.
7. **Uniform-chunk container overhead is 9 bytes**; a region-file wrapper
   (many chunks per file, shared headers) would amortize even that. Format v1
   is a per-chunk container by design; the region grouping is future work and
   can wrap v1 unchanged.
8. **Far field is spherical.** Below-viewer LOD is the point of cubic chunks
   (megachasm vistas), but a full sphere of deep rock is the naive version —
   the adaptive load volume (ARCHITECTURE § chunk shape) should shape the far
   field too (sky-exposure-driven downward extension), reusing exactly the
   summaries built here.
