# 0088 — the day the rendering stopped being haunted

*2026-07-24. Two builds landed — the deep-cell keystone and the contents inspector
— and between them a long walk through the "haunted" rendering the user kept
finding on the east coast. By the end, three or four separate-looking bugs turned
out to be one, and a dev tool we built to measure it turned out not to be needed
for the measurement.*

> blogworthy (lenses: procgen-against-priors; deepsim-reflexions): the moment
> where "the far LOD is haunted," "the chunks are a checkerboard," and "member
> stepping looks loud" all collapsed into a single sentence — *a coarse ~460 m
> facies field, point-sampled instead of interpolated.* Four screenshots, one
> root cause, and a fix (octaves, not resolution) that the user reached by
> refusing to accept the flattering version.

## The keystone landed

The deep-cell material inventory (Crux 1) merged: a mutable working span-list per
deep cell, and the **transformation-fact ledger** the commit-semantics design
called for — in-place transformation appends a fact, depositional arrival appends
a unit, facts persist while the working inventory is transient, `commit_chapter`
is diff-and-append. Byte-identical under the identity default over 25,600 real
cells; a non-identity agreement test proves a known edge round-trips through the
provenance read. The single-edge shape shipped; the design refined mid-flight to
**apply-time edge logging** (a multi-edge chapter's net delta has no unique
factorisation, so the log — not a post-hoc diff — is the authority) and to a
**`cause`** field (the responsible agent/actor), with the pass/epoch/driver
**derived-and-displayed, not stored**. Those ride in with the first real behaviors.

## The instrument, and what it revealed

The contents inspector merged too: `world_get_contents` (the full `VoxelContents`
beside `classify`'s one-name summary — S-3 in reverse) and an F3 look-at HUD. It
surfaced a load-bearing fact: **the runtime `Chunk` stores only the 1-byte
`Block`; the full mixture does not survive at the sim/edit tier** — it is
re-derived from worldgen (pure-of-pos, edit-blind) for unedited voxels. That is
S-2 working (don't store what you can derive), and it is also the runtime face of
"block is a summary": giving the player the real mixture on break needs the
runtime twin of the keystone — contents as *derivable base + edit facts*, a break
as a *move-fact*. Designed, not built; the north-star's first runtime-process
milestone.

## The haunting, dissolved

Three field reports, diagnosed read-only (planning held at the user's direction):

- **Haunted far LOD** — first-gen dither near, faithful texture farther, the
  finest band inverted. One path, one gate: `REDUCTION_STANDOFF_M = 176 m` sits
  *inside* the L1 ring (112–256 m), so L1's inner shell synthesises cold while
  every band ≥ 256 m warm-reduces. Static, not an eviction race.
- **Chunk checkerboard (palette-quant)** — a chunk's whole composition is a
  *single point-sample of the deep record at the chunk centre*, NEAREST at the
  ~460 m deep-cell grid, shared across all 1024 columns. Continuous pattern,
  jumping composition.
- **Same root cause.** Both are the coarse ~460 m NEAREST deep-facies field
  **point-sampled instead of interpolated/area-summarised** — near and far, two
  sites, one disease. The S-4 rule unmet.

And the fine 16×16 squares the user counted turned out to be a fourth face of the
same family: **within-class member stepping**. A class (clastic, igneous) carries
a *distribution* over its members; a per-voxel dither materialises it. But
`interp_select_draw` is **one octave of value noise at chunk wavelength** — already
world-anchored and C0-continuous (we were both wrong that it "ignored neighbours"),
but single-scale, so every patch is chunk-sized and reads as a grid. The user
pushed past the flattering answer: a finer grid just makes *smaller* squares; the
fix is **octaves** (fractal, a physical facies driver for the large scale, the
finest octave bounding mesh cost) — structure at every scale, so no scale reads as
a grid. That is the mixture-representation arc, now opened.

## What the day taught about the loop

Two process notes. The palette-quant "measurement" we scoped an inspector for was
answerable **in the code** — the mesher and dither are per-voxel world-anchored, so
the discontinuity had to be upstream, and it was; the inspector confirms but was
never the critical path. And an agent lost a wake-up exactly as the user predicted
(its test finished, it never reported) — resumed cleanly from its transcript with
the machine state, and it had done the work all along. Absence of a report is not
absence of work.
