# 0016 — the ladder that was already tall enough, and the map that wasn't

*2026-07-19 · surface-machinery hardening (background agent; integrated by the main session, merge `81a87b8`). The walk-12 report it was dispatched to fix turned out to be my own misdiagnosis — corrections #10 — and the investigation found the real defects anyway. Seed 1337, N=2 worldgen authority.*

Walk 12 came back with a blocking receipt: `pose_set { surface: true }` at
(x=40, z=6) returned feet at y ≈ 1004.45, a cross-check said the column was
solid — "granite at y=1050, stone at 1006/1010" — and `eye_in_solid` said
`false` while the walker believed it was standing inside a mountain. The
diagnosis written down was tidy and plausible: `true_surface_m` scans a column
downward from a per-column *ceiling*, that ceiling came from a pre-deep-time
elevation estimate plus an S1-sized 8 m headroom, and 3e-1 had just moved real
ground by ~100 m. The window would start below the ground, find nothing, and
return a point inside the rock. *The ground moved and the ladder didn't.*

## Reproducing it, and the first surprise

The reproduction came back with the walk's own number and the opposite verdict:

```
(40, 6): analytic=1003.50  true=1004.40  vy=1116  here=Air  below=Dirt
```

Feet at 1004.45 m is `true + 0.05`, exactly what the walk saw — and the voxel
the feet occupy is **air**, with **dirt** immediately beneath. That placement is
correct. The teleport was not burying anybody.

The cross-check was the thing that lied, and it lied by units. The MCP block
queries take **world voxel coordinates**; the pose reports feet in **meters**.
The surface voxel in that column is 1115. So "granite at 1050" is 65 voxels
*below* the walker's feet — basement, exactly where granite belongs — and
"stone at 1006/1010" likewise. Read as meters, those three numbers all sit above
1004.45 and paint a walker entombed under 45 m of rock. Read as what they are,
they are a perfectly ordinary column: soil on stone on granite.

> blogworthy: an agent walking its own world cross-checked a meters answer
> against a voxels answer and wrote a blocking bug report about a mountain that
> wasn't there. The fix for that class isn't a code change; it's making the two
> coordinate systems impossible to confuse in a reply.

## The second surprise: the ceiling was already honest — by inheritance

Why *didn't* the diagnosed mechanism fire? Because the worldgen authority's
ceiling had stopped being an independent estimate before deep time ever landed.
It reads `ColumnRec.heights`, and those heights come from
`lattice(L_VOXEL, …)` — the same midpoint-displacement pyramid that 3e-1 taught
to inject the eroded deep-time surface at `L_DEEP`. The ceiling inherited deep
time for free, on the day the terrain did, because it was derived from the
terrain's own provenance instead of re-deriving it alongside.

That is the whole lesson, and it is worth more than the bug: **the ceiling was
safe precisely because it was not a second opinion about the terrain.** The S1
authority's ceiling is an estimate (a half-voxel-exact analytic bound) and it is
still fine at S1 scale; the worldgen authority's is a *read of the generator's
own answer*, and it cannot drift from the terrain because there is nothing to
drift from.

## Proving the danger was real anyway

A near-miss you cannot demonstrate is indistinguishable from a story. So the
regression test was written first and the fault injected second: force the
worldgen ceiling 200 m below the deep-time surface, one line, and run it.

```
seated body embedded at (-281.7, -317.3), ts = 798.30, analytic = 993.60
```

A body seated **195 m inside the mountain** — the walk-12 report, exactly as
written, just from a cause that wasn't live. And the old code returned that
point *silently*, because when the downward scan found nothing it fell back to
`analytic - SURFACE_SCAN_DEPTH_M` and handed back a plausible-looking float. A
miss and a surface were the same type. That is the defect that was genuinely
shipped, whether or not anything was triggering it.

## The bug that was actually live: `eye_in_solid` answered from another world

`eye_in_solid` did not consult the authority. It consulted the client
`ChunkMap`, whose miss path is `terrain.block_at(…)` — the legacy **S1
`TerrainGen`**, whose surface sits at ~8 m. Under the worldgen authority the
walkable world is ~1000 m up, so for any chunk that had not streamed yet, the
answer to "is my eye in rock?" was computed against a different planet and came
back `false` for every altitude the player can actually occupy. Right after a
teleport — precisely when nothing has streamed and precisely when the walker
most needs the check — `eye_in_solid: false` was structurally guaranteed and
carried no information at all.

This is why the walker's instruments could not settle the question it was
asking. It was told "your eye is clear" by a map of somewhere else.

The fix is small and honest: `Authority::is_solid_m` queries the hosted world,
which lazily generates the one chunk and always answers for the world the player
is standing in — edits included.

## What changed

- `true_surface_m` returns `Option<f64>`. A scan that finds no top-solid voxel
  is a **miss**, not a buried y. `surface:true` teleport leaves the position as
  requested and says `surface_snapped: false`; `character_attach` refuses with a
  `no_surface` receipt and spawns nothing; scale-switch keeps the current
  altitude; spawn falls back to the analytic height.
- A `debug_assert` fires if a column's top solid voxel reaches the scan ceiling
  — i.e. the window started inside the terrain. The class of bug journal/0015
  described now trips a wire instead of returning a number.
- The S1-sized constants are gone. The 8 m headroom was slop against an
  *estimate*; with the ceiling derived from the authority's exact column height
  it is only an **edit** allowance, now `SURFACE_SCAN_EDIT_HEADROOM_M = 4.0`.
  Scan depth 220 m → 96 m for the same reason.
- `eye_in_solid` reads the authority, not the ChunkMap's wrong-world fallback.

Journal/0006's guarantees are untouched: footprint-max over per-column scans,
edits included, both authorities supported, legacy S1 keys 3/4 still green.

## The class

Two silent-failure shapes, and the walk hit the seam between them:

1. **A query whose upper bound is an estimate of the thing it is measuring** is
   correct exactly until that thing's provenance changes — and fails *downward*,
   into terrain, where a returned float still looks like an answer. The
   antidote is not a bigger headroom; it is deriving the bound from the same
   source as the data, so the two cannot move apart.
2. **A fallback that answers from a different world.** The ChunkMap miss path
   was a reasonable default when the S1 terrain *was* the world. When the
   authority changed underneath it, it silently became a confident liar — and
   the far mesh, mesh-border culling, player collision, character grounding,
   edit targeting, and collider tiles all still consult it (see ROADMAP
   Observed; only `eye_in_solid` is fixed here).

And a third, which cost more than either: an instrument that reports one
quantity in meters and its neighbour in voxels will eventually be cross-checked
against itself.

## Coda from the main session: whose error this was

The walk-12 report was mine, and it was wrong in the most ordinary way
available: I read a pose in **meters** and cross-checked it against block
queries in **voxels**. At N=2 those scales differ by 0.9, and 111 voxels of
difference turned a walker standing on dirt into a walker entombed in
granite. Every subsequent inference — the stale ceiling, the lying
`eye_in_solid`, the BLOCKING label, the dispatched brief — descended from
that one unchecked unit.

This is corrections #3 wearing new clothes. Then it was "don't diagnose a
renderer from a viewpoint you haven't verified is in air"; now it's "don't
diagnose a placement from an instrument whose units you haven't verified."
The fix shipped for the first lesson was `eye_in_solid`. The fix owed for
this one is smaller and just as mechanical: **pose replies should echo the
voxel coordinate alongside the meters**, so the two languages a walker speaks
can be compared without a mental conversion nobody performs reliably at 3am.
Filed.

What redeems the episode is that the false alarm still paid for itself. The
investigation found that `eye_in_solid` was answering from the legacy S1
world for any unstreamed chunk — structurally `false` right after a teleport,
which is precisely when a walker asks — and that a surface scan finding
nothing returned `analytic − 220 m` as though it were an answer. Both were
live. Neither was what I reported. The walker's instrument was broken in a
way that made my misreading *unfalsifiable from inside the game*, which is
the part worth remembering: when the tool that would catch your error is the
tool that's broken, you get a confident wrong story.

> blogworthy: "a miss and a surface shared a type" — the `Option` that
> wasn't. Plus the units lesson: an agent walking a world needs its
> instruments to speak one language, or it will invent geology that isn't
> there.
