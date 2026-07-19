# 0017 — one world-answer surface: closing the S1-fallback class

*2026-07-19 · the S1-fallback sweep (background agent; worktree branch for the
main session to integrate). Closes the defect class journal/0016 § "The class"
named but left standing — only `eye_in_solid` was fixed there. Seed 1337, N=2
worldgen authority.*

## The class, restated

journal/0016 found `eye_in_solid` answering from a different planet: it read the
client `ChunkMap`, whose miss path fell back to the legacy S1 `TerrainGen`
(surface ~8 m) while the worldgen authority put the walkable world ~1000 m up.
Right after a teleport — nothing streamed, every chunk a miss — the query
answered against a world the player was not in. That entry fixed the one
instrument and enumerated its siblings, all consuming the same root:

```
map.is_solid(&terrain.0, scale, x, y, z)   // ChunkMap miss → terrain.block_at → S1
```

Six of them, all silently wrong under the worldgen authority: player collision
(`player.rs`), character ground-finding (`character.rs`), the crosshair edit
raycast (`edit.rs`), physics collider tiles (`physdemo.rs`), and mesh-border
face culling in two places (`streaming.rs`, `remesh_dirty`). The far rings
(`farmesh.rs`) were worse still — they generate *entirely* from S1, painting a
phantom old world a kilometre below the real terrain.

The through-line is a single sentence worth keeping: **a cache had a fallback to
a generator it did not own.** The `ChunkMap` is a render/collision cache of the
authoritative hosted world. When it missed, it did not say "I don't have that";
it invented an answer from a generator that, since the worldgen authority
landed, describes a different world. A miss and an answer shared a code path, so
the wrong-world answer shipped into gameplay wherever a chunk hadn't streamed —
which is exactly the streaming edge, exactly where collision matters most.

> blogworthy: "a cache with a fallback to a generator it doesn't own." The bug
> was invisible for as long as the fallback generator *was* the world (the S1
> era). The day the authority changed underneath it, every consumer of the cache
> became a confident liar at its own edges — and nothing in the type said the
> answer might be from somewhere else.

## The fix: one solidity surface, and it's the authority

The shape was decided going in, and it held: give the authority a voxel-level
solidity query reading the hosted world (lazily generating, edits included — the
same path `is_solid_m` already used for `eye_in_solid`), and route every site
through it.

```rust
pub fn is_solid_voxel(&mut self, x: i64, y: i64, z: i64) -> bool {
    self.world.block_at(Vec3i::new(x, y, z)).is_solid()
}
```

`is_solid_m` (the meters-typed `eye_in_solid` shim) collapses to a scale
conversion over it. The six sites now build a closure over the active authority
and hand it to the same dc-core / dc-physics functions as before —
`move_aabb`, `raycast_voxels`, `column_top_solid_y`, the physics `advance`.

### The `&mut` inside an `Fn`

The one real friction: `block_at` needs `&mut self` (it lazily generates the
containing chunk), but every dc-core query takes `&impl VoxelQuery`, whose
blanket impl is for `Fn(i64,i64,i64) -> bool` — an *immutable* `Fn`. The
established idiom in this file (already used by `true_surface_m` and the attach
embed-guard) is interior mutability: wrap `&mut authority` in a `RefCell`, and
the closure `|x,y,z| cell.borrow_mut().is_solid_voxel(x,y,z)` is `Fn` while the
work behind it is `&mut`. Every routed site uses exactly that three-line pattern.

Bevy scheduling fell out for free: the Update systems are already a single
`.chain()`, so eight of them now holding `ResMut<Authority>` serialize without a
conflict — they already ran in order.

### The character-loop borrow

`sync_characters` iterated `authority.world.characters()` (an immutable borrow
held across the whole loop) and needed `&mut authority` inside for the foot-IK
ground read — a direct conflict. The wrong turn I didn't take was threading a
second resource or a snapshot struct; the right one was cheap: collect the
characters into an owned `Vec` first (a handful of bodies), releasing the borrow,
then build the solidity closure over the freed authority. Foot placement now
reads the world the body stands in, edits included — which also retires a
walk-11 loose end (the foot-IK could momentarily see the S1 far-mesh phantom at
the extreme load-radius edge).

## The surprise in the numbers

The mission flagged mesh-border culling as the hot path to watch: lazily
generating a neighbour chunk through worldgen sounds far more expensive than an
S1 analytic sample. Measured (headless A/B, same worldgen chunks, only the
border-neighbour query differing), it is the opposite:

```
mesh-border culling mean per chunk over 75 chunks:
  S1-analytic (old)              1.168 ms
  authority lazy-neighbour (new) 0.931 ms   (cold; worldgen cold-chunk 0.281 ms)
```

The old path *looked* cheap but recomputed FBm hills + chasm + 3D cave noise on
every border voxel. The new path generates each neighbour chunk once (~0.28 ms
cold) and every subsequent border query into it is a cache hit — and that
neighbour is one the streamer was about to generate anyway, so the work isn't
even new, just pulled a few frames forward. No neighbour cache was needed; the
authority's own column/chunk memoization is the cache.

> blogworthy: the "expensive" correct path was cheaper than the "cheap" wrong
> one, because per-voxel analytic noise lost to generate-once-and-memoize. A
> reminder to measure the hot path instead of pricing it by intuition (this is
> corrections #8's lesson wearing a client hat: cost is a measurement).

## The far mesh: left honestly wrong, on purpose

Part 2 was "source far rings from worldgen if cheap; otherwise leave S1 with a
loud comment and write up the shape." It is not cheap. The far mesh samples a
*coarse voxel scale* (2^L base voxels — 14.4 m voxels at L4), and the worldgen
generator is N=2-baked: it produces 0.9 m chunks off the collapse pyramid, with
no coarse-scale sampling path. Two ways to feed the existing far-mesh path from
worldgen, both real work:

1. **Full-res generate then downsample.** Deriving 1 km of LOD pyramid means
   full-res-generating 1 km of world (~5 min, per farmesh.rs' own module note) —
   a non-starter at boot.
2. **A persisted coarse summary pyramid.** The right answer, and the one the
   ROADMAP already wants under "far field should become summary-shaped (adaptive
   volume)": worldgen emits per-column height/material summaries at coarse LOD
   levels on save; the far mesh reads cached summaries. This is renderer-scale +
   storage-layer work (couples to S3 region-file grouping and the LOD-derive-on-
   save path exercised in `--bench-storage`).

So the far mesh stays on S1, now isolated in a single loudly-named resource
(`farmesh::FarFieldTerrain`) whose doc names the defect. It is *honestly* wrong
(a visible phantom that dissolves on approach) rather than *silently* wrong
inside a shared cache. The shape of the remaining work is filed to ROADMAP
Observed.

## The guard: making the class unrepeatable

Three layers, so the compiler catches the next wrong-world consumer before review
does:

- **The root is gone.** `ChunkMap::is_solid` — the method with the generator
  fallback — is deleted. The cache is now doc'd as answering only for chunks it
  holds; there is no method on it that invents an answer. A caller that wants an
  unloaded-chunk answer must go to the authority; there is nowhere else to go.
- **The ambient wrong-world resource is gone.** The `Terrain` resource that held
  the fallback `TerrainGen` no longer exists in the ECS. A new system cannot
  `Res<Terrain>` a wrong world by accident — the resource an author would reach
  for isn't there. The only surviving S1 generator lives inside
  `farmesh::FarFieldTerrain`, the sanctioned far-field residue.
- **A tripwire test that fails on old main.** `empty_cache_solidity_paths_read_
  the_worldgen_authority` boots the worldgen authority with no streamed chunks
  and asserts a known-solid voxel 900 m down answers *solid* through every path
  a gameplay system reaches: the point query, the `Fn` closure, a downward
  `raycast_voxels`, and a `move_aabb` drop. It first asserts the precondition
  that the old empty-cache fallback (S1) calls that voxel **air** — so on
  pre-sweep main, where those paths ran through `ChunkMap::is_solid`, every
  assertion would fail. It catches all six routed sites at once.

The doctrine — the client has exactly one world-answer surface, and caches never
fall back to a generator they don't own — is drafted into ARCHITECTURE.md, dated,
marked NEEDS RATIFICATION.

### What the guard does *not* yet do

`TerrainGen` the *type* is still `pub(crate)` — nameable crate-wide — because
the authority's 3/4-key legacy path, the far-mesh residue, and the headless
benches all legitimately construct one. Fully sealing it behind
`pub(in crate::authority)` is blocked only by the far mesh's sanctioned S1 use;
it becomes possible the day the far field moves off S1. Filed. What *is*
compiler-enforced today: no cache fallback method exists, no ambient wrong-world
resource exists, and no near-field gameplay system names `TerrainGen` at all
(verified — none import it).

## What changed, by site

| Site | Was | Now |
|---|---|---|
| `player.rs` collision | `map.is_solid(&terrain, …)` | `move_aabb` over authority closure |
| `character.rs` foot IK | `ground_top_m(&map, &terrain, …)` | `ground_top_m(&solid, …)`, solid = authority |
| `edit.rs` raycast | `map.is_solid(&terrain, …)` | `raycast_voxels` over authority closure |
| `physdemo.rs` colliders | `map.is_solid(&terrain, …)` | `advance` over authority closure |
| `streaming.rs` border cull | `map.is_solid(&terrain, …)` | authority closure (fetch chunk/contents first) |
| `authority.rs` `remesh_dirty` border cull | `map.is_solid(&terrain, …)` | authority closure |

Keys 3/4 still boot the legacy S1 authority; at those scales the authority *is*
the S1 terrain, so routing through it preserves behaviour — all existing tests
(including the S1-terrain spawn/attach/parity/replay suite) stay green.

## Coda

journal/0016 ended "only `eye_in_solid` is fixed here." This is the rest of the
sentence. The instrument that lied and the collision that fell through a
streaming edge were the same bug seen from two windows; both are closed by the
same move — deleting the fallback and pointing everyone at the one authority.
The part I'll remember: the fix made the code *faster*, because the correct
world was already memoized and the wrong one was recomputing noise. Correctness
and cost pointed the same way, which is not the usual shape of these.
