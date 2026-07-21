# 0044 — The flip is pomp

*2026-07-21. The user ratified turning `tectonic_history` on in production and,
unusually, cut the appearance walk out of the loop before it happened: "I'm
going to tell you to flip tectonics regardless of what it looks like because I
want us to make progress. We can correct mistakes later. The flip is pomp."
So this is U8, the last of the eight tectonics decisions — and it is the first
flip we have shipped that the walk documents rather than gates. This entry is
the flip, the re-baseline ledger, and the measured cost.*

## The decision, and why it reads the way it does

Every prior world-shape flip on this project earned its ratification through a
photograph. The erodibility flip (0030) was "flip it, i want to see." The
amplitude A/B (0040) was two screenshots at the same summit. U8 breaks that
pattern on purpose. The tectonics design (tectonics.md) posed U8 as "production
flip after spike screenshots, 0030-style" — and the user overrode the
sequencing: ship it now, judge it later. The reason is stated in the memory as
the standing "no bandaid, make progress" posture — an interim mechanism rides
as-built, and a spike screenshot is not a prerequisite for making the sealed
path real. The knob that would actually change the mountains (`thickening_scale`,
U7) is *not* part of this flip: corrections #23 already measured that it buys no
sub-km relief — it lifts the whole continent by ~700–1000 m and leaves the
walking-scale relief byte-identical — so its value is deferrable and rides at
the `DeepConfig` default of 80. `full_agents` stays off; it was never ratified.

## What the flip is

One line in `production_config` (`deeptime/field.rs`), the same class of change
as the `biotic` and `erodibility` flips sitting right above it:

```rust
tectonic_history: true,
```

Off, the deep-time run was a one-shot painting of uplift onto coarse cells with
a constant plane added for 200 iterations. On, it is the chaptered kinematic
history the design describes: plates advect through K=8 chapters, boundaries
re-classify and repaint an analytic forcing, convergence thickens crustal
columns, smoothed-Airy isostasy derives elevation from column state, and
drainage re-marches against a *moving* landscape. Concretely, in every world
made from here on:

- **`surf` and `strata` change** — terrain shape, and the depositional record
  that shape carries, are different everywhere.
- **The tectonic-only exports stop being empty.** `recv`/`area`/`lake` (final
  drainage), `exhum`/`t_crust` (the metamorphic-grade axes), and the `chapters`
  table are populated in production for the first time. They are *exported and
  consumed by nothing yet* — the collapse-tier slice that reads them
  (dip/fold/fault expression, metamorphic classes) is Sequenced, not built. I
  corrected two doc-comments in `field.rs` that claimed the collapse tier
  "reads" these axes; it does not, and stubs.md § 4 already flagged the lie.

As with 0029/0030, the honest cost is stated plainly: **every world created
after this flip is unreproducible under any earlier build.** That is exactly
why it needed ratifying, and it is the whole reason the byte-identity override
channel (0039) exists — an all-`None` `DeepOverrides` still reproduces the
sealed path to the bit, so the flip changes the *default*, not the *mechanism*
of reproducibility.

## The re-baseline ledger

The flip is a world-fingerprint event, so tests that froze production content
broke. Three did. None was a bug; each is the flip doing its job, and in each
case the test's actual subject survived intact.

**1. `deep_config_plumbing::tectonic_history_override_populates_the_tectonic_exports`**
(renamed `…_toggles_…`). This is the test tectonics.md warned would *invert*.
It proved the override channel bites by building the production field (tectonics
off, exports empty) and an override field (`Some(true)`, exports populated).
With production now tectonics-*on*, the "off" baseline is no longer off — its
exports are non-empty and the `assert!(off.chapters.is_empty())` fails. The
subject is "the `tectonic_history` override reaches the run," and it is
unchanged; I reworked the falsifier to prove it through the seam that now turns
the bundle *off*: production carries the exports, and `Some(false)` empties every
tectonic-only plane. The byte-identity tests (empty overrides == production) were
untouched and still pass — they compare two now-tectonic fields and find them
identical, which is the point.

**2. `deeptime_integration::collapse_column_story_comes_from_the_deep_record`.**
Asserted that the land column with the richest deep record renders a multi-band
strata cliff (≥2 distinct tagged block kinds stacked in a voxel column). After
the flip the single max-events column moved to `(-38,-8)` and rendered **0**
distinct kinds — while its same-event-count neighbour `(-30,-8)`, also 11 events,
rendered 2. I did not take the null on faith; I measured the whole field. **4908
of 6400 sampled land columns (77%) render a ≥2-band cliff post-flip** — the
record→blocks mechanism is entirely healthy; the flip simply moved the
richest-*events* cell onto a spot whose tagged sediments don't stack two-deep in
its surface chunk. Event count was never a good proxy for a readable cliff. This
is the 0030 coal-test situation exactly (a frozen single-point pick landing on a
degenerate spot while the mechanism is intact everywhere), and I fixed it the
same way 0030 did: select the *richest column that also surfaces the cliff*, as
0030 took "the thickest seam that also survives collapse to diggable coal." A
selection fix, not a floor loosening.

**3. `s7_walk::walk_10k_chunks_across_the_civilized_boundary`** —
`SEAM_TOLERANCE_VOXELS` 6 → 12. The 10 000-chunk continuity walk asserts adjacent
voxel columns differ by less than a tolerance chosen to sit between natural
slopes (well below) and seams (tens of voxels). The flip steepens natural
relief: the max *interior* adjacent-column step over the whole transect rose from
≤6 to **7** voxels — a legitimate cliff face in tectonic terrain. Crucially, the
panic was on the *interior* slope; every one of the 10 000 chunk-*border*
crossings still stayed ≤6, so the load-bearing seam invariant (a mismatched
border shows as tens of voxels) is untouched. I raised the shared tolerance to
12 — clear of the measured 7 with margin, still an order below the seam scale —
and recorded the old and new numbers in the source.

Everything else in the workspace passed unchanged, including all the erodibility
and biotic byte-identity proofs (they build explicit configs, so the production
flip does not reach them), the same-seed determinism fingerprints (both sides of
each comparison are now tectonic and still bit-identical), and the geology world
regeneration tests.

## The measured ritual cost

Measured at the production entry point (`build_field`), tectonics off vs on, on
the byte-identical parallel path, seed-matched same-build A/B via the override
channel:

| extent | grid | build OFF | build ON | ratio | resident OFF → ON | Δ resident |
|---|---|---|---|---|---|---|
| Small | 160², 460 m | 2147.8 ms | 1913.2 ms | 0.89 | 3.07 → 4.42 MB | +1.3 MiB |
| Medium | 545², 460 m | 14318.5 ms | 15240.3 ms | 1.06 | 34.2 → 55.3 MB | +20.1 MiB |

The Medium numbers confirm S12's prediction on the shipped path: S12 measured
16.2 s / 1.13×, and the production ritual lands at **15.2 s / 1.06×** — inside
the ~15 s ritual class, tectonics adding about a second. (The Small ratio below
1.0 is measurement noise at ~2 s absolute; there is no real speedup — the flip
does not make generation faster.) The **+20 MiB resident at Medium** is the
tectonic exports the world now keeps: four `f64`/`i32` planes at 297 k cells
(~9.5 MB) plus the lake mask, the chapter table, and the strata record's growth
from the chapter-stamp merge breaks — all noise against the run's working set,
exactly as § 11 of the design predicted.

## What comes next, honestly

Nothing about this flip makes the mountains legible on its own. Corrections #23
stands: relief lives at sub-km scale, in the collapse lattice's roughness decay
and in erosion supply, not in deep-time forcing amplitude. What the flip *does*
buy is that the exhumation, crustal-thickness, drainage and chapter data are now
real in every production world — so the Sequenced collapse-tier expression slice
(chapters → dip/fold/fault in cut faces; `exhum`/`t_crust` → metamorphic-grade
classes) has live inputs to consume the day it is built. The flip is pomp; the
substrate it lights up is not.

> blogworthy: **shipping the mechanism before the payoff.** Every other
> world-shape change on this project waited for a photograph that justified it.
> This one shipped on "make progress, correct mistakes later" — and the
> discipline that makes that safe is the same one that makes the *reversible*
> flips safe: a byte-identical override channel means the production default can
> move without the reproducibility guarantee moving, so "we can correct mistakes
> later" is a true statement about the code, not a hope.

## Files

`crates/dc-worldgen/src/deeptime/field.rs` (the flip + two honesty fixes),
`docs/design/tectonics.md` (U8 DECIDED addendum), the three re-baselined tests
named above, and this entry.
