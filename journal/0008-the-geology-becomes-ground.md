# 0008 — the geology becomes ground you can stand on

*2026-07-19 · 3c-1 (background agent; integrated + walk-verified by the main
session, merge `ee639f9`, 35 suites green on merged main). Carries the first
geology photographs, as journal/0007 promised.*

Geology shipped headless in 0007: strata records, passes-as-graph, a placer
that finds gold-dust by density alone. All of it invisible — the client still
streamed the S1 spike terrain (`TerrainGen`: rolling hills, one chasm, some
caves), and the hosted world behind the edit authority generated that same
hill-field. 3c-1 is the smallest honest step to *walk* the geology: swap the
real worldgen into the client's chunk seam, give the four v1 strata classes
each a visible block, restore the ratified N=2 scale, and freeze a companion
that loses its session.

## The seam was shaped for a pure function; the generator is a stateful borrow

The client embeds a `HostWorld` as its edit authority (journal/0002). Its one
extension point is the `ChunkGenerator`:

```rust
pub type ChunkGenerator = Box<dyn Fn(ChunkPos) -> Chunk + Send + Sync>;
```

A closure. Pure. `Send + Sync`. The S1 `TerrainGen` slid in without friction —
it samples noise in meter space and owns nothing. `WorldGenerator` is the
opposite shape in three ways, and each one is a real constraint the seam is
right to impose:

1. **It borrows.** `WorldGenerator<'a>` holds `&'a Pregen` — the pregenerated
   coarse world (tectonics, climate, drainage, history) it collapses from. A
   `Box<dyn Fn ... + 'static>` cannot hold a borrow.
2. **It is `!Send`.** Its collapse caches were `Rc<RegionRec>` / `Rc<LocaleRec>`
   / `Rc<ColumnRec>`. `Rc` is the correct choice for a single-threaded lazy
   pyramid and the wrong one for anything that must cross a `Send` bound.
3. **It mutates on read.** `generate_chunk(&mut self, ...)` — the caches
   memoize, the trace records the lookahead assertion, the mixture table
   interns. A `Fn` (not `FnMut`) closure cannot call it without interior
   mutability.

The resolutions, smallest-first:

- **Borrow → `Arc`, without disturbing the borrowing API.** A new
  `PregenSource<'a>` enum — `Borrowed(&'a Pregen)` or `Owned(Arc<Pregen>)` —
  derefs to `Pregen`, so every `self.pregen.…` inside collapse reads
  identically. Every existing caller (the S7 tests, the geology determinism
  suite) keeps its stack-owned `Pregen` and the borrowing `new`/`with_geology`.
  The client alone uses the new `new_owned(Arc<Pregen>)`, which yields a
  `WorldGenerator<'static>` that carries its world by reference count. No test
  churn, byte-identity untouched.
- **`Rc` → `Arc`.** Mechanical. The atomics cost nothing measurable and buy
  `Send`; determinism is a property of *what* is cached (pure derivations of
  seed + pregen), never of the pointer's refcount discipline. `column_record`'s
  public return type went from `Rc<ColumnRec>` to `Arc<ColumnRec>` — the only
  visible ripple, and the surface machinery wanted that handle anyway.
- **`&mut` on read → a `Mutex`.** The closure locks one `WorldGenerator` and
  calls `generate_chunk`. `Mutex<T>: Send` needs only `T: Send`, which the
  `Arc` change delivered.

**What the `Mutex` hides, and what it must never hide.** It hides *sequencing*:
two callers can't be mid-`generate_chunk` at once, so the caches never tear.
It must never hide *order-dependence* — if the chunk you get depended on which
chunks were generated before it, the lock would be papering over a determinism
bug, not solving a threading one. The dc-worldgen suite already proves
byte-identical regeneration and registration-order independence; 3c-1 adds the
proof at the seam itself (`worldgen_seam_is_order_independent`): two authorities
from one seed, the same chunk set generated front-to-back and back-to-front,
byte-identical every chunk. The lock is allowed to serialize; it is not allowed
to matter.

One quiet subtlety kept the block tier honest: the worldgen `MixtureTable`
interns in generation order, so its *ids* are order-dependent (0007's noted
loose end). 3c-1 pushes only **blocks** through the seam (`generate_chunk`, not
`generate_chunk_with_materials`), and block bytes are order-independent — the
material sidecar, and its order-sensitive table, wait for 3c-2.

## Two worlds share one lock

The generator behind the `Mutex` isn't only the chunk source — it is also the
*surface authority*. The `Arc<Mutex<WorldGenerator>>` is cloned once: the
`HostWorld` closure holds one handle, the `Authority` holds the other. Both
point at the same generator and the same warm caches, so a `surface:true`
teleport and the chunk the player stands on are answering from one world.

This forced the surface machinery (journal/0006) to grow up. `true_surface_m`
scans the real voxels downward from a per-column *ceiling*, and that ceiling
came from `TerrainGen::surface_height_m` — an analytic field the worldgen does
not have. But it does not need one: `ColumnRec.heights` already carries the
surface voxel top per column, which is exactly the upper bound the scan wants.
So the surface query became authority-aware — solidity always from the hosted
world (edits included), the ceiling from whichever authority is live:
`surface_height_m` for terrain, the `ColumnRec` height for worldgen. Spawn,
`surface:true` teleport, and `surface:true` attach all route through the one
`Authority::true_surface_m`, and the embed-guard/attach tests pass over both.

The lock discipline that keeps this safe is un-nested-ness: the analytic
closure locks the generator, reads a column height, and *releases* before the
solidity scan runs (which may lock the generator again to lazily birth a
chunk). Sequential locks on one thread, never a lock held across a call that
re-enters. A `RefCell` splits the borrow so the analytic reads `self.surface`
while the scan reads `&mut self.world` — disjoint fields, no aliasing.

## The blocks, and the scale that was a stale default

The four v1 strata classes each earned a `Block` — `Mudstone`, `Sandstone`,
`Granite`, `Basalt`, appended (never inserted: `Block` is a positional postcard
id, corrections #4) after `Wood`. The collapse block-tier reader, which mapped
every clastic stratum to `Dirt` and everything else to `Stone`, now speaks the
four: clastic-fine → mudstone, clastic-coarse → sandstone, intrusive → granite,
extrusive → basalt. Unrecorded basement stays `Stone`; the soil band above the
record stays `Dirt`/`Grass`; the placer ore never forms a band of its own (it
rides *inside* a sandstone event), so it needs no block. The determinism tests
never asserted specific block ids — they compare same-seed runs — so the new
mapping passed them untouched.

Boot moved from player = 3 voxels to **2** — the ratified S1 scale that a
spike-era bench default had quietly overridden (ROADMAP Observed). The scale
keys became authority-aware: **key 2 = the worldgen authority** (N=2-baked, the
real world), **keys 3/4 = the legacy S1 `TerrainGen`** at that resolution. The
window title now names the active authority so a walker always knows which
world they are in. *(This key mapping is flagged for ratification.)*

## Freeze on disconnect

A companion whose session drops kept walking under its last move intent until
it hit something (0005's cliff was the feature; the walking-corpse was not).
DECIDED 2026-07-19: **freeze** — a disconnect zeroes the move intent, the body
stands where it was left. The mechanism is a session-teardown `Drop`: the
per-session MCP handler holds an `Arc<SessionEnd>` shared across every clone
rmcp makes to serve a request, so its `Drop` runs *exactly once* — when the
last handle to the session goes away, never on a transient per-call clone. On
that drop, if the session had attached, it sends one `CharacterFreeze` down the
bridge, and the authority submits a zero move-intent command on the ordinary
receipted rail. It rides the same door as any controller verb, so the replay
bit-identity is unchanged — freeze is not a body-side behavior, it is the last
thing the controller says before it goes quiet.

## Perf

Cold-chunk generation through the seam measured **~0.24 ms mean** over a
contiguous 5×5×3 block of chunks around spawn — faster than the S7 headless
0.711 ms surface mean, but not a fair fight: a contiguous local block shares
region- and locale-collapse across its columns, whereas the S7 number sampled
scattered surface chunks. Read it as "the seam adds nothing measurable, and
locality helps" rather than as a speedup. The Medium pregen at boot is the
one-time "generating world history…" cost (~15 ms, 0007), paid once.

## Walk 7: the quarry

Main session, merged main, `--fullbright`, freshly rebuilt exe (timestamp
checked against the merge — the 0006 lesson holds). Spawn landed in air at
**y = 529** — the worldgen world has real S7 elevation, and the N=2 spawn
seated correctly on the new authority's surface.

The vista (`assets/0008-first-geology-vista.png`) is all grass and soil —
correctly: the soil band caps everything, and geology does not show itself on
gentle slopes. So the first geology photograph took the honest route: **dig**.
A scan column at spawn read, bottom to top: 53 voxels of granite → 3 of
basalt → mudstone → dirt → air. A dev-command quarry (21×38×13 voxels of
`world_fill dc:air` — the receipt came back `from: "dc:granite"`, the edit
pipeline speaking the new blocks) opened the cut, and the wall shows the
column exactly (`assets/0008-quarry-strata-bands.png`): pale granite mass,
near-black basalt flow, red-brown mudstone drape, tan soil cap. From the rim
(`assets/0008-quarry-from-rim.png`) the pit reads as a proper stratigraphic
section. Every shot from verified open air (`eye_in_solid: false`).

An honest observation for the visuals pass: a single-material wall under
fullbright flat shading is a *featureless color field* — the first quarry
shot was a full frame of undifferentiated granite pink. Banding carries all
the information; within-material structure carries none yet. That is exactly
the gap the interim dither (3c-2, decided this session) and later the
LabPBR/splat pipeline exist to close.

**Freeze, live.** The `Drop`-fires-once mechanism got its end-to-end proof
the unit tests couldn't give it: character `strider` attached via 7778,
driven at full walk (measured 4.5 m/s), session killed with an HTTP DELETE
mid-stride — reattach found it standing within a tick of the drop, velocity
zero, position stable across polls. The walking-corpse era (0005) is over.

Ratified 2026-07-19 during integration: the authority-aware scale keys
(2 = worldgen, 3/4 = legacy S1 terrain as a dev affordance — with the note
that scale 3 may yet win later on feel).

> blogworthy: "one lock, two worlds" — a pure-function chunk seam meets a
> stateful, borrowing, single-threaded lazy generator, and the adaptation
> (open the ownership without opening the API; Arc the caches; let the Mutex
> serialize but forbid it from mattering; make the surface query ask whichever
> authority is live) is a small clinic in fitting real machinery through an
> interface designed for a toy.
