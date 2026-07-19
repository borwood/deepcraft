# 0015 — deep time becomes the world

*DRAFT — 3e-1 (background agent; integrated by the main session). Numbers from
the Windows dev box, release profile, seed `0x0D5E_ED57_2026`. Full context in
docs/spikes/S9-results.md and S9b-results.md; the ratification is
earth-processes.md § S9 VERDICT.*

For three spikes the deep-time tier had been a thing measured, never a thing
lived. S9 built the two-plane erosion engine and proved it told true stories —
pulsed arid fans, transgressive couplets, condensed cyclic margins — at 460 m
and seconds. S9b priced its parallelism and closed the B escape hatch. The user
ratified A-always-on + C-refinement. This milestone is the A half: take the
spike engine, which touched nothing real, and make it **the world's actual
history** — the terrain you walk on and the strata you cut through.

## Promoting a measurement harness to a pass

The spike lived beside the pipeline: `deeptime::run(&pregen, &cfg)` handed back a
`DeepGrid` full of scratch, and the collapse layer never saw it. Making it
always-on meant it had to become a *declared pipeline pass* like everything
else — honest reads and writes, topo-sorted, not a hand-call bolted onto
`Pregen::run`.

The first friction was a chicken-and-egg one. The pass runs *inside* pregen, so
there is no `Pregen` yet when it fires — only the `PregenCtx` with the coarse
`CellGrid`. The spike's builder took a whole `Pregen`. So the engine got a new
front door, `run_cells(&CellGrid, cfg)`, and the old `run(&Pregen, cfg)` became a
one-line wrapper the spike harnesses keep. The pass declares
`reads: [Elevation, Provenance, Climate]`, `writes: [DeepElevation, DeepStrata]`,
and Kahn's algorithm slots it third — right after climate, because it needs
nothing hydrology produces (it computes its own drainage), and its id sorts ahead
of the hydrology/igneous ties. That it *doesn't* read Hydrology is the honest
part: the deep sim floods its own basins; borrowing pregen's coarse drainage
would be a lie about where its rivers are.

What survives the run into world state is deliberately thin: a `DeepField` of the
eroded **surface** (`R+H` per cell) and the per-cell **strata record**. The
uplift/precip planes and the erosion scratch are dropped. ~25 MiB on a Medium
world, kept for the world's lifetime — the price of history you can walk to.

## The cost fork the brief didn't price: extent

S9 measured A at 460 m / Medium = 14 s. But cell count grows with the *square* of
extent, and the deep tier is now on the world-create critical path for **every**
extent. A literal 460 m at Large (~1017 km) is a 2211² ≈ 4.9 M-cell run —
minutes and gigabytes — and it would blow the `pregen_time_vs_extent` <60 s
budget the test suite already asserts. The ritual is supposed to be ~15 s ("it
IS the ritual"), not a coffee break, and the machine that hangs on parallel
builds does not want a five-minute Large world-create in every test run.

The smallest honest option: **cap the deep grid width** (~550 cells). Small and
Medium keep 460 m exactly as specified; Large gets a coarser cell (~1.8 km) that
holds the ritual to ~300 k cells regardless of world size. The read-quality is
coarser at Large — but S9 already measured true stories at the 1 km sweep, and
the C-refinement slice (3e-2) is precisely where landform detail for *approached*
Large regions comes back on demand. Measured after the cap:

| extent | deep cells | world-create |
|---|---|---:|
| Small  | 161²  | ~1.2 s |
| Medium | 545²  | ~12.7 s |
| Large  | 550² (capped from 2211²) | ~12.2 s |

The cap is not a nicety — without it the Large assert is red. FLAGGED for the
integrating session as a deviation from a literal fixed 460 m.

## The elevation composition, and the seam that didn't happen

Scope item 2 was the one I expected to fight: make the collapse elevation lattice
derive from the deep-time surface without breaking the max-6-voxel seam invariant
the walk test guards over 10 000 chunks. The lattice is a midpoint-displacement
pyramid — cell corners at level 0, refined with addressed jitter down to voxels —
and its continuity comes from adjacent columns *sharing lattice ancestors*.

The clean insight is that the deep tier's 460 m cell is almost exactly the
**locale** scale (512 voxels = 460.8 m, lattice level 5). So the composition is
surgical: at level 5, replace the analytic elevation with a bilinear sample of
the deep surface; leave every finer level's jitter untouched. The macro-terrain
becomes the eroded deep-time landscape; the sub-460 m relief stays the same
fractal detail as before. Roughness is never touched — it still schedules the
fine jitter.

I braced for seams — the deep surface and the old analytic terrain are *different
fields*, and where they meet (the wilds boundary, and every level-5 corner) a
mismatch could jump tens of voxels. It didn't, and the reason is the same
mechanism that made the pyramid continuous in the first place: any difference
between two adjacent level-5 corners is spread linearly across the 512 voxels
between them. Even a 500 m corner-to-corner relief change is ~1 voxel per
step. Measured over the full walk: **max interior step 2 voxels, max border step
2** — unchanged from before, comfortably under the tolerance of 6. The wilds
boundary holds because inside and outside both reduce to the shared pregen edge
elevation (ocean, low relief); the deep sample is simply absent past the grid and
the analytic value stands.

> blogworthy: "The seam that didn't happen." Injecting a whole different terrain
> field into a fractal pyramid *sounds* like a guaranteed discontinuity — and the
> intuition is wrong for exactly the reason the pyramid was continuous to begin
> with: shared ancestors interpolate any parent mismatch away. The fix and the
> invariant are the same theorem.

## Class selection against a climate that no longer exists

The point of the whole milestone is scope 3: the deep-time record becomes the
**formation-context source** for depositional geology. Until now the clastic pass
asked "what member forms here?" against the column's *year-zero* temp and
precip — the ratified shim. Now a depositional stratum asks against the
environment measured *when it was deposited*: paleo precipitation from the
recorder's aridity tag, temperature from the column's latitude (a stable axis;
the paleo-temperature curve is a later slice), burial depth from the overlying
record. Marine (subsea) units select fine mud; high-energy subaerial units select
coarse proximal bodies; low-energy distal reaches select fines. The facies→class
mapping is roster-independent — only *which* member fills the class is
selection — so the "adding a member diversifies but never inflates" invariant
still holds to the byte.

The structural decision that made this safe was to keep the year-zero **veneer**
and inject the deep history *below* it. Year-zero climate is ratified-legitimate
for the active, still-forming surface — soil and modern alluvium — and the placer
rides that modern fan (a placer is a *present-day* channel process). So a land
column now reads, bottom to top: igneous basement, then the recorded deep-time
sequence under its at-deposition context, then the recent veneer. That ordering
is the sentence a cut face speaks — marine mud under arid fill under recent
veneer — and it falls straight out of the pass order.

## The sampling choice: nearest, not bilinear

A surface is a scalar; you can bilinear it. A **record** is a variable-length
sequence of tagged units; you cannot interpolate one. So elevation samples the
deep surface bilinearly (continuous), but the *story* comes from the **nearest**
deep cell. That means facies contacts step at the 460 m deep-cell grid rather
than wandering continuously — but 460 m is a geologically legitimate facies scale,
and crucially it is *coarser* than the 28.8 m chunk grid, so corrections #6's
chunk-line cutover does not recur (the per-voxel member dither still smooths
member contacts *within* a facies). FLAGGED: softening the facies-boundary step is
a C-refinement concern, not this slice.

## The first column whose story came from history

`integrated_deep_record_tells_true_stories`, ported from S9's metrics against the
now-integrated field, reads the same alive distribution the spike measured —
proof the promotion didn't flatten anything:

```
record-bearing 184 764, mean units 1.58, multi-unit 34 173,
marine 50 898 (0.275), subaerial 155 205, transgressive 21 339,
tag-varied 34 173, unconformities 6 500
```

Marine fraction 0.275 against S9's 0.265; 21 339 transgressive-regressive
couplets; 6 500 proto-unconformities — the record is the world, and the world
runs. And `collapse_column_story_comes_from_the_deep_record` walks the surface
chunk of the richest sampled column and confirms the recorded sequence reaches
*blocks*: a multi-band cliff of distinct strata a player can read, not a noise
function dressed as rock.

## Walk 12: the terrain that happened — and the ceiling that didn't move

> **CORRECTED 2026-07-19 (corrections #10, and see journal/0016).** The
> regression reported below is **not real**: I compared a pose in meters
> against block queries in voxels, and an ordinary column looked like a
> burial. `true_surface_m` was already deep-time-aware (its ceiling reads
> `ColumnRec`, which 3e-1 taught the deep-time surface). The section stands
> as written because it is what I believed at the time; the investigation it
> triggered found genuine defects elsewhere — an `eye_in_solid` that answered
> from the legacy S1 world, and a miss path that silently returned a buried
> point. Read this section as the wrong turn, and 0016 as the mechanism.

The vista is unmistakable (`assets/0015-deeptime-vista.png`): the S1
hill-field is gone, replaced by **long erosion-graded ridges and rounded
divides** — landscape as the time-integral of uplift minus erosion rather
than as summed octaves. A scanned column reads granite basement → basalt
flows → a thin recorded clastic band → soil veneer: the deep record
reaching blocks, exactly as designed.

**Then the walk found a regression, and it is the walk-6 lesson wearing a
new hat.** `pose_set { surface: true }` at (40, 6) returned feet at
y = 1004.45 — while direct authority queries show that column solid
(granite at y = 1050, stone at 1006/1010). The teleport put the player
**inside rock**, and `eye_in_solid` returned `false` while doing it.

Mechanism (diagnosed, not yet fixed): `true_surface_m` scans a column
downward from a per-column *ceiling*, and for the worldgen authority that
ceiling still comes from the pre-deep-time elevation estimate plus a small
headroom (the S1-sized `SURFACE_SCAN_HEADROOM_M = 8`, already an Observed
item since journal/0006). 3e-1 replaced macro-elevation with the deep-time
surface at the locale level — which moves real ground by **~100 m** in
places. The scan window now starts *below* the true surface, finds no
top-solid voxel in range, and returns a point inside the mountain. The
`eye_in_solid: false` is the same lie from the same stale window (plus
client-side chunk-map solidity that hadn't streamed).

So: the deep-time world generates correctly and reads correctly, but
**every consumer that seats a body — spawn, surface teleport, safe attach
— is broken against it until the ceiling learns about deep time.** Filed
blocking; fix dispatched the same session. The photograph of a deep-time
cut face is owed to walk 13, once a walker can stand up in this world.

> blogworthy: "the ground moved and the ladder didn't" — a surface query
> whose upper bound is an *estimate of the terrain it is measuring* is
> correct exactly until the terrain's provenance changes. Journal/0006
> taught us not to seat bodies on an analytic field; 3e-1 taught us the
> same field can also be a stale *ceiling*, which fails silently in the
> other direction.

## Perf

- **World-create ritual**: Small ~1.2 s, Medium ~12.7 s, Large ~12.2 s (capped).
  Within the ~15 s budget; the "generating world history…" pause is the ritual.
- **Cold chunk**: ~0.94 ms (was 0.711 ms) — +0.23 ms for the per-lattice-point
  deep-surface sample and the nearest-cell record lookup. Warm ~0.39 ms. Still
  ~250× under the latency budget.
- **Memory**: the kept `DeepField` is ~25 MiB at Medium (surface + records).

## Falsified / refined

- "A literal fixed 460 m deep tier is affordable at every extent" → false at
  Large (minutes, GiB, blows the <60 s assert); the width cap holds the ritual by
  coarsening the cell for very large worlds. (→ corrections candidate.)

*(corrections.md entries are the integrating session's call — candidate listed in
the agent report.)*
