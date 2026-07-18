# S1 — Voxel scale walking skeleton: results

Status: measurements collected, **game-feel judgment deferred to the human**
(see "What to try" below). The voxel:meter ratio decision is NOT made in this
file; it gets recorded in ARCHITECTURE.md once the interactive pass has been
done.

## How to run

- Interactive: `cargo run --release -p dc-client`
- Headless measurements (no window/GPU needed): `cargo run --release -p dc-client -- --bench-scales`

Controls: **left click** captures the mouse, **Esc** releases it, **WASD** +
mouse look to move, **F** toggles fly/walk. In fly mode **Space/Shift** go
up/down (no clipping); in walk mode **Space** jumps and swept-AABB collision +
gravity apply. **2/3/4** rebuild the world at player-height = 2/3/4 voxels
(same seed, same landscape — only resolution changes). The window title shows
the current scale and mode.

## Measurements

Headless bench (`--bench-scales`), release profile, single-threaded, seed
1337. Region: 256 m x 128 m x 256 m (x/z in [-128, 128), y in [-96, 32) —
includes the surface at ~-6..+22 m and the chasm floor at ~-80 m). Machine:
Windows 11 dev box, 2026-07-18.

| Player height | Voxel size | Chunks | Voxels | Raw chunk memory | Gen time | Mesh time | Triangles |
|---|---|---|---|---|---|---|---|
| 2 voxels | 0.900 m | 600 | 19,660,800 | 37.5 MiB (39,321,600 B) | 1.07 s | 0.44 s | 2,555,016 |
| 3 voxels | 0.600 m | 1,372 | 44,957,696 | 85.8 MiB (89,915,392 B) | 2.59 s | 0.79 s | 4,355,932 |
| 4 voxels | 0.450 m | 3,240 | 106,168,320 | 202.5 MiB (212,336,640 B) | 5.90 s | 1.62 s | 7,504,446 |

Normalized to the volume each scale actually covered (chunk grids overshoot
the meter region differently per scale — 1.43x at N=2, 1.16x at N=3, 1.13x at
N=4):

| Player height | Voxels / m^3 | Raw bytes / m^3 | Triangles / horizontal m^2 | Gen / chunk | Mesh / chunk |
|---|---|---|---|---|---|
| 2 voxels | 1.37 | 2.74 | 30.8 | 1.78 ms | 0.73 ms |
| 3 voxels | 4.63 | 9.26 | 60.3 | 1.89 ms | 0.58 ms |
| 4 voxels | 10.97 | 21.95 | 111.7 | 1.82 ms | 0.50 ms |

Notes on reading the table:

- Region is the same world volume in meters at every scale; chunk grids don't
  align to meter boundaries, so coverage overshoots slightly and differently
  per scale. The table reports what was actually generated.
- "Raw chunk memory" is the dense in-memory form (32^3 x 2-byte block ids =
  64 KiB/chunk, air included). This doubles as the save-size-per-m^3 upper
  bound for S1; palette compression (S3) will shrink both dramatically, but
  the *ratio* between scales is the durable datum: each +1 to player height
  multiplies voxel count per world volume by ((N+1)/N)^3, and 2 -> 4 voxels is
  exactly 8x.
- Mesh time is culled meshing (skip faces between two solids), single-threaded,
  release profile. Triangle counts are what the culled mesher actually emits.

## Decisions and implementation notes

### Bevy version: pinned `=0.19.0`

Latest stable on crates.io at spike time (2026-07-18). No API friction worth
noting: the client compiled against 0.19 cleanly on the first attempt
(`CursorOptions` as its own window-entity component, `AmbientLight` as a camera
component, `shadow_maps_enabled` on lights are the notable 0.19-isms). Pinned
exact so renderer work in S3/S4 happens against a fixed surface; upgrades are
deliberate events.

One wrinkle: bevy 0.19 uses glam 0.32 internally while the workspace (dc-core)
pins glam 0.30, so two glam versions exist in the tree. dc-client converts at
the boundary (f64 sim math in workspace glam, f32 render math in bevy's). Fine
for a spike; worth revisiting when the workspace next bumps glam.

### Floating origin

- All simulation state is **f64 meters** in world space: player feet position
  and velocity (`Player` resource), chunk placement derived from `ChunkPos`
  (i32 lattice) x voxel size.
- A `FloatingOrigin(DVec3)` resource anchors render space. Every frame, camera
  and chunk transforms are recomputed **from f64** as `(world - origin)` and
  only then truncated to f32. The GPU never sees a coordinate larger than the
  rebase radius.
- When the player wanders more than 256 m from the origin, the origin snaps to
  the player. Because transforms are recomputed from f64 every frame anyway, a
  rebase is a single resource write — no entity is touched, nothing jumps.
  Recomputing a few hundred chunk transforms per frame is noise; if it ever
  isn't, the system can trivially become change-triggered.

### Chunk size: 32^3

- Cubic chunks want a cube; 32^3 = 32768 voxels = 64 KiB raw at 2-byte ids.
- Big enough to amortize per-chunk overhead (hash entry, entity, mesh, draw
  call), small enough that gen+mesh of one chunk fits comfortably in a frame
  slice and a single block edit remeshes only 64 KiB of world.
- Power of two keeps voxel->chunk math to shifts/masks.
- 16^3 doubles chunk-count overhead along every axis of our 3D streaming
  radius; 64^3 (256 KiB) makes remesh/gen granularity noticeably worse.
- Rationale also lives in `dc-core/src/chunk.rs` module docs. S3 owns palette
  compression and LOD pyramids; S1 stores a plain boxed array.

### Cubic-chunk proof

Chunk streaming loads a true 3D radius (a sphere of chunk positions, meters-
based so every scale streams the same world volume) around the player. The
terrain includes deep winding chasms (~90 m deep, carved where a low-frequency
noise mask crosses zero) plus 3D noise caves: descending into a chasm loads
chunk layers below exactly the way walking north loads them ahead. There are
no heightmap/column assumptions anywhere in the loaded-chunk path.

### Terrain determinism across scales

All noise is sampled in **meter** space (voxel centers converted to meters), so
seed 1337 produces the same hills/chasms/caves at every scale — only sampling
resolution changes. This is what makes the 2/3/4 comparison apples-to-apples,
and it also means cross-chunk neighbor queries during meshing can fall back to
the generator for not-yet-loaded chunks: borders mesh correctly on first
contact with no remesh bookkeeping (valid while the world is read-only; world
edits arrive with S5's `dc-api` slice and will need dirty-neighbor remeshing).

### Collision

dc-core's swept-AABB (per-axis sweep, Y then X then Z) runs in voxel units;
the client converts meters<->voxels at the call site via `VoxelScale`. The
sweep scans every cell in the swept range, so high-speed movement cannot
tunnel (tested at 10,000 voxels/step). Player box is 0.6 m x 1.8 m x 0.6 m at
every scale; jump impulse is computed to clear ~1.3 voxel heights, so jump arc
scales with voxel size **on purpose** — that difference is part of what the
human should evaluate.

## Observations (from the headless numbers)

- **Volume costs land exactly on theory**: bytes/m^3 go 2.74 → 9.26 → 21.95,
  i.e. x3.375 from N=2→3 and x8.0 from N=2→4 — precisely (N_b/N_a)^3. There is
  no hidden constant softening the cubic blowup; the scale choice really is a
  world-resolution budget decision.
- **Surface costs scale ~quadratically**: triangles per horizontal m^2 go
  30.8 → 60.3 → 111.7 (x3.6 from N=2→4, vs x4.0 for pure area scaling; caves
  and chasm walls keep it slightly sub-quadratic). At N=4 the 256x128x256 m
  region already meshes to 7.5 M culled triangles — greedy meshing and S3's
  LOD stop being "nice to have" well before large view distances at that
  scale.
- **Per-chunk costs are scale-independent** (~1.8 ms gen, 0.5-0.7 ms mesh,
  single-threaded): the cost of a finer scale is purely that the same world
  volume contains more chunks. Streaming the same meter radius therefore costs
  ~5.4x more chunks at N=4 than N=2 (600 vs 3,240 over the bench region).
- The interactive app's synchronous budget of 8 chunks/frame can consume up to
  ~19 ms of main-thread time in the worst case — fine for a spike, but chunk
  gen/meshing wants to move to async tasks before this is a real client.
- Raw dense storage (64 KiB per chunk regardless of content) is the dominant
  memory number; most loaded chunks are all-air or all-stone, so S3's palette
  compression should collapse the in-memory and on-disk cost by an order of
  magnitude or more at any scale. The 8x *ratio* between N=2 and N=4 survives
  compression, though — it's fundamental.

## Open questions for the human (game-feel — deliberately not decided here)

Take the interactive pass and judge:

1. **Readability**: at which scale does the terrain (hills, chasm walls, cave
   mouths) read best from eye height? Fly up ~50 m too — does the world still
   read as "blocky on purpose" at 4 voxels/player, or does it start to look
   like low-res smooth terrain?
2. **Reach & block placement feel**: no block editing exists yet, but stand
   next to a chasm wall and judge: at this voxel size, would placing
   individual blocks feel like building or like pixel-tweezing? (This sets
   how badly we'll want multi-voxel brushes/blueprints at the chosen scale.)
3. **Jump arc**: jump clears ~1.3 voxels at every scale, so at N=2 you vault
   1.17 m and at N=4 you hop 0.59 m. Which reads as "a satisfying jump"?
   Should jump height instead be fixed in meters (climbing N voxels of stairs
   per jump)? This decides stair/slab conventions.
4. **Walking on slopes**: natural terrain at N=2 produces 0.9 m ledges you
   must jump; at N=4, 0.45 m steps. Does N=4 walking feel pleasantly smooth or
   does the terrain lose its voxel crunch? (No auto-step is implemented; say
   if you want step-up assist to evaluate this properly.)
5. **Corridor/door instinct**: imagine digging a player-sized tunnel: 1 voxel
   wide at N=2 (0.9 m) vs 2 wide at N=3 (1.2 m) vs 2 wide at N=4 (0.9 m).
   Stand in the caves and judge which cross-section feels like a "door."
6. **Chasm vertigo check**: walk (not fly) down into the chasm. Does chunk
   streaming keep up at descent speed? Any pop-in that breaks the sense of
   depth at a given scale?

Keys to use while judging: 2/3/4 to flip scales in place (same landscape), F
to toggle fly/walk, Space to jump. The window title always shows the current
scale.

Also open (engineering, not feel):

- Whether the ~8x memory/mesh cost of N=4 vs N=2 is acceptable pre-S3; the
  final budget table should be produced *after* the scale is chosen, against
  S3's compressed chunk format.
- Greedy meshing was skipped (culled meshing met the exit criterion); the
  triangle counts above are therefore upper bounds and the between-scale
  ratios, not the absolute counts, are the durable numbers.
