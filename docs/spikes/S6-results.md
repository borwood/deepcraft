# S6 — Physics bubble: results

Status: complete. Adds `crates/dc-physics` (rapier3d island for the dynamic
tier) and a minimal client demo (**G** tosses a rigid-body cube). Characters
stay on dc-core's swept-AABB path, untouched.

## How to run

- Bench (headless, no GPU): `cargo run --release -p dc-physics --example s6_bench`
- Tests: `cargo test --release -p dc-physics`
- Demo: `cargo run --release -p dc-client`, press **G** to toss a cube; cubes
  collide with loaded terrain (and with the generator-sampled fallback for
  unloaded chunks — the same solidity query the player uses).

## Rapier version and the bevy_rapier question

**Pinned `rapier3d = 0.34.0`** (latest stable at spike time, 2026-07-18), used
**directly, not through bevy_rapier**. Rationale:

- The parts a Bevy integration layer owns — the stepping loop, collider
  lifecycle, transform sync — are exactly the parts this design must own: our
  static colliders are transient bubble tiles keyed to voxel data, our
  stepping is a fixed accumulator in our own schedule, our sim positions are
  f64 world meters under a floating origin, and everything below dc-client
  must stay headless. bevy_rapier's value proposition (ECS component sync,
  Bevy-scheduled stepping) is precisely the coupling we would fight.
- dc-physics stays a headless workspace crate testable in CI without a GPU;
  dc-client only ever consumes poses.
- 0.34 note: rapier's math types are now glam-based (`glamx`: `Vec3` vectors,
  `Pose3` isometries, quaternion rotations; `BroadPhaseBvh` is the broad
  phase). Conversion at our API boundary is trivial component copying —
  rapier's internal glam 0.33 vs the workspace's glam 0.30 never touch.

**`enhanced-determinism` is enabled.** In 0.34 the feature swaps
transcendentals to `libm` (`simba/libm_force`) and is mutually exclusive with
the (opt-in, not default) SIMD features. Cost observed at our scales:
negligible — the perf table below was measured *with* the feature on, and step
times are a fraction of a 60 Hz frame at N = 500. The real cost is forgoing
`simd-stable`/`parallel` later; re-measure before assuming that matters.

## Collider-bubble strategy

The voxel world is never a global collider set. Instead:

- **The lattice is tiled into 4³-voxel tiles** (config: `BubbleConfig`). Tiles,
  not per-body radii, are the unit of collider lifetime.
- **A body's bubble** is its collider AABB expanded by a margin (default 1 m)
  plus one fixed step of its current velocity, converted to the covering tile
  range. Sleeping bodies keep their bubbles — a sleeper woken mid-step by an
  impact must find its support colliders already present.
- **Refresh is a set difference, so it is incremental by construction**: each
  fixed step recomputes the wanted tile set (a `BTreeSet` union over all
  dynamic bodies); tiles entering it (the leading edge of motion) are built,
  tiles leaving it (the trailing edge) have their colliders removed. The
  bubble interior is untouched — a stationary body causes zero tile work
  (asserted in tests). Tiles wanted by several bodies are built once and live
  until *no* bubble wants them.
- **Building a tile** scans its 64 voxels through the caller-supplied solidity
  closure (`dc_core::VoxelQuery` — dc-physics never learns what a chunk is),
  greedy-merges solid voxels into cuboids, and inserts **one fixed compound
  collider per non-empty tile**. Tiles that scan to all-air are cached as
  known-empty so they are not rescanned every step.
- **Greedy merge choice** (documented per scope): greedy rectangles per
  horizontal layer (extend along +x, widen along +z), then a third cheap pass
  merges rectangles with identical footprints on adjacent layers into taller
  boxes. O(cells), deterministic, and collapses floors/walls/slabs to a
  handful of cuboids; optimal box cover is NP-hard and not worth chasing. A
  32³ flat-floor slab becomes 1 cuboid; the bench's noise terrain averages
  ~2–3 cuboids per active tile.

Determinism relies on ordered iteration everywhere colliders are created or
destroyed (`BTreeMap`/`BTreeSet`), so insertion order into rapier's arenas is
a pure function of the command history.

## Perf numbers (N=2 scale, the decided 0.9 m voxel)

Headless bench, release, `enhanced-determinism` on, Windows 11 dev box,
2026-07-18. Pit floor replicates the S1 hills noise layer verbatim
(OpenSimplex2 FBm, 4 octaves, freq 0.008, seed 1337, 8 m ± 14 m) with a wall
ring; N cubes (0.4 m, CCD on) tumble in for 600 steps @ 60 Hz:

| N cubes | mean step | max step | mean bubble refresh | peak tiles | peak cuboids | asleep at end |
|---|---|---|---|---|---|---|
| 10 | 0.023 ms | 0.425 ms | 0.009 ms | 43 | 142 | 10/10 |
| 100 | 0.175 ms | 1.484 ms | 0.066 ms | 169 | 483 | 100/100 |
| 500 | 0.836 ms | 3.386 ms | 0.223 ms | 291 | 694 | 499/500 |

Readings:

- **Step cost scales sub-linearly in N** (50× the bodies ⇒ ~36× the mean
  step) and even the N = 500 worst-case step (3.4 ms, the initial pile-up
  frames) fits a 16 ms frame with room to spare. Sleep detection does the rest:
  by 10 simulated seconds essentially everything is asleep and steps cost
  ~µs.
- **The bubble refresh is a rounding error** (≤ 0.25 ms mean at N = 500,
  ~25 % of step cost at worst) — the tile cache and known-empty cache work.
  Collider population peaks at ~700 cuboids across ~300 tiles for 500 bodies;
  colliders drop to zero when bodies are removed (tested).
- Peak tiles grow much slower than N because settled cubes pile up and share
  tiles.

## Detach → settle → reattach (100-voxel prop)

One rigid body, compound collider of greedy-merged cuboids at voxel-perfect
fidelity (the 100-voxel test prop — 6×4×4 slab + asymmetric 4-voxel tail —
merges to 2 cuboids). Dropped ~10 m onto the S1-noise terrain:

| phase | result |
|---|---|
| detach (merge + compound build) | 0.036 ms |
| settle to sleep | 516 steps (8.60 sim s), 10.7 ms wall |
| reattach (snap + write-back set) | 0.005 ms |
| voxel count preserved | 100 → 100 |

Re-attach snaps the asleep pose to the lattice in two exact moves: nearest of
the **24 axis-aligned rotations** (max |quaternion dot|; each is an integer
signed-permutation matrix, so voxel offsets map to voxel offsets with no float
error), then the anchor voxel center snaps to the nearest lattice center —
with the rotation integral, one translation snap aligns every voxel at once.
Because an integer rotation is a lattice bijection, the round-trip preserves
voxel count and shape *by construction*; the test additionally verifies
count, no duplicates, and congruence (equality under one of the 24 rotations)
after a real tumble.

Caveat recorded: the snap is only correct because the compound's local frame
is the detach-time minimum voxel corner and rapier's body pose tracks the
local frame (not the center of mass). If the local frame convention changes,
the snap must change with it.

## Determinism findings

Same spawn script (24 items + 1 detached prop, 600 steps over a bumpy world)
⇒ **bit-identical final poses** across two fresh `PhysicsWorld`s (f32 bit
patterns compared, test `identical_scripts_produce_bit_identical_final_poses`).

Rapier caveats found:

- Determinism is conditional on **identical command order**: rapier iterates
  its arenas in insertion order, so any hash-ordered container feeding
  body/collider creation would silently break replay. That is why every
  bubble structure is a `BTreeMap`/`BTreeSet`; keep it that way.
- Within one machine/build, plain f32 rapier is already reproducible;
  `enhanced-determinism` buys *cross-platform* reproducibility (libm
  transcendentals) — the property future networked replay wants — at no
  measurable cost at our scales (table above measured with it on).
- One 500-cube run left 1 body awake at the 600-step cutoff — sleep timing is
  part of the deterministic state, not noise, but don't write tests that
  assume "everything sleeps by step K" across code changes.

## Asleep/far item handoff design (documented, not implemented)

Dropped items should not hold rigid bodies forever. Design:

- An item body that (a) is asleep and (b) has no player/observer bubble near
  it is **demoted to an inert item record**: `{ item id, pose (f64, snapped
  to nothing — items rest at arbitrary poses), chunk pos }` stored with the
  chunk (S3 sidecar or the S5 entity surface), and the rigid body + its
  bubble tiles are freed. This is the same shape as the sim tiers: full
  physics is a bubble, everything outside it is a record.
- **Promotion back** when a dynamic bubble or observer approaches: recreate
  the body at the recorded pose, asleep; it only wakes if something actually
  disturbs it. No impulse is replayed — an asleep body's future is "stay
  put", which the record preserves exactly.
- **Unloaded-chunk fall-through**: if the voxel under a demoted item is later
  edited away, the item re-checks support on promotion (cheap solidity query
  at its footprint), falling only when observed — consistent with the S2
  committed/fluid philosophy.

## Client demo

`dc-client` gains `physdemo.rs` (~130 lines): **G** spawns a cube tossed along
the view direction; `PhysicsWorld::advance` runs on the fixed 60 Hz
accumulator inside the existing Update chain; poses sync to render transforms
origin-relative like chunks and the camera. The solidity closure is
`ChunkMap::is_solid` — loaded chunks first, deterministic generator fallback
for unloaded ones, so cubes never fall through a not-yet-streamed chunk.
Switching voxel scale (2/3/4) resets the demo physics world (the lattice
changed). No pickup, no detach UI — detach is proven headless.

## Open questions

- **f32 far from origin**: rapier is f32; at 10 km from the world origin,
  positions quantize at ~1 mm. Fine for the spike, but the dynamic tier will
  eventually want physics in origin-relative space (rebase the physics world
  with the floating origin) or per-island local frames. Decide when the world
  actually gets big.
- **Network determinism**: enhanced-determinism covers instruction-level
  reproducibility, but replay also requires identical command *order* across
  peers — the dc-api command stream is the natural ordering authority.
- **Buoyancy/fluids**: nothing here models fluid volumes; rapier has no
  built-in fluid coupling. Likely a per-body force generator sampling the
  (future) fluid field inside the bubble.
- **Ragdolls**: multibody joints are compiled in and unused. Bubble logic is
  body-count-agnostic, but a ragdoll's many small colliders will want a
  slightly larger tile size or a per-body-group bubble.
- **World edits vs live bubbles**: tiles cache voxel scans; a block edit
  inside an active bubble must invalidate the covering tile (one `BTreeMap`
  remove). Wire this to the S5 edit path when the two meet.
- **Item demotion thresholds**: the handoff design needs numbers (sleep time,
  distance) once real gameplay exists; the bench suggests even hundreds of
  live items are cheap, so demotion is about memory and save format, not CPU.
