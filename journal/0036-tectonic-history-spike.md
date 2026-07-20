# 0036 — the tectonic-history spike: uplift becomes uplift(t)

*Draft — spike branch `spike/tectonic-history`, not merged. Measured numbers in
docs/spikes/S12-results.md.*

For its whole life the deep-time engine has added the **same** uplift plane to
bedrock every one of its 200 iterations. `Erosion::new` even precomputed
`uplift_sum` on the flat assumption that uplift is constant. That is the
"one-shot upheaval" the field reports kept naming: one orogeny per world per
place, all of the same age, no superimposed belts, no unconformity *machinery*,
flat-lying beds everywhere (the layer cake). And the forcing was painted onto
14.7 km pregen cells with a 2-ring falloff and bilineared into the 460 m grid,
so nothing narrower than ~15 km could exist and every belt flank smeared to
~50 km.

The ratified design (tectonics.md, all of U1–U8) inverts the machine: **surface
uplift stops being the input.** The input is plate kinematics — seeds and
velocities, advected through chapters — and each chapter's boundary geometry is
evaluated *analytically at deep-grid resolution*. What that forcing drives is not
elevation, it is **crustal thickening** of per-cell columns; **isostasy** then
derives elevation from the columns each step. Erosion argues with the result for
five hundred million years. Everything sits behind one `DeepConfig` flag,
`tectonic_history`, off by default and byte-identical when off.

This entry records what the spike built and, in the § 14 spirit the design asked
for, where implementation reality contradicted the spec.

## The shape of the build

Two new modules and a set of phases threaded into the existing loop:

- **`deeptime/tectonics.rs`** — plate kinematics (seeds + velocities in km,
  advected `pos += vel·chapter`), the chapter table, and the analytic forcing
  `amplitude(type, v_conv) × exp(−(d/W)²)` where `d` is the *exact* signed
  distance from a deep cell to the plate-pair bisector and `W` is a design
  parameter in kilometres. There is **no grid term anywhere in the expression**,
  so the only wavelengths in the forcing are the designed ones — method rule 5
  satisfied at the source, not dressed after.
- **`deeptime/isostasy.rs`** — Airy compensation of the *smoothed* load. Per-cell
  Airy at 460 m would be physically wrong (real lithosphere supports loads below
  the flexural wavelength) and a stability hazard aimed straight at the
  erodibility clamp — an erosion↔rebound feedback at exactly the grid scale. So
  the load is smoothed over the flexural wavelength (a separable prefix-sum box
  blur, scalar in both drivers), the Airy equilibrium surface is derived, and
  bedrock relaxes toward it at `iso_rate`. Smoothing the compensation *is* what
  plate rigidity does.
- New crustal-column planes on `DeepGrid` (`t_crust`, `crust_kind`, `exhum`), a
  `DepUnit.chapter` stamp, and a drainage export on `DeepField` (`recv`, `area`,
  `lake`, plus `exhum`/`t_crust`/the chapter table).

The chapter loop lives in `run_cells`: it precomputes one analytic forcing plane
per chapter geometry (repaint-per-chapter, milliseconds) and blends two adjacent
planes per iteration across the chapter ramp — the forcing is always in transit,
which is what lets a transverse river saw a gorge through a rising axis instead of
being dammed (a step function is the negative control).

## The load-bearing decisions, and the deviations from spec

**1. The mass ledger did not migrate all the way to thickness-space — and it
should not.** The design says the conserved stock becomes column thickness and
`R` is reconstructed from `(T, H)` by isostasy. Taken literally that is a rewrite
of `transport`/`weather`/`diffuse`, all of which operate on `R` directly, and it
would forfeit the byte-identical-off composition the whole flag rests on. What
actually keeps conservation exact and portable is subtler and, I think, more
honest: the engine keeps `ΣR+ΣH` as its stock and declares **isostasy's bedrock
injection as an external input**, exactly the way the biotic layer declares
`biotic_total`. The ported mass test then reads `Δ(ΣR+ΣH) == uplift_total +
biotic_total` unchanged, where `uplift_total` is now the summed isostatic ΔR. The
thickness stock gets its *own* exact invariant — `Δ(Σt_crust) == thickening −
exhumation` — because `t_crust` changes by exactly two things: the thickening
forcing adds, and every metre of bedrock the erosion phases remove decrements it
(and grows `exhum`). Both ledgers balance to fp slack in the tests. This is the
§ 14 pattern applied to the design's own § 3.4: the letter (one migrated stock)
gave way to two exact stocks with isostasy as a declared input.

> blogworthy: "the ledger that split in two" — the design wanted one conserved
> stock; the code wanted two, each exact, with buoyancy as an external input like
> photosynthesis. Naming the coupling instead of hiding it is what kept the mass
> test a one-line falsifier.

**2. `Erosion::new` caching `uplift_sum` (design § 14.3) is real and now
documented in place.** On the legacy path it is still valid and still returned.
On the tectonic path it is simply not the ledger — the step returns the isostatic
injection instead. The cache is left, with a comment pointing at why it does not
apply, so a future reader does not rediscover it as a mysterious conservation
failure.

**3. The chapter stamp threads through `deposit`/`overprint_top` as an explicit
parameter, not hidden state.** Every call site passes a chapter; off the flag it
is always `0`, so the extra `top.chapter == chapter` merge guard is always
satisfied and the record is byte-identical. `sizeof(DepUnit)` is unchanged — the
`u8` fits the existing 8-byte padding, as the design predicted (verified in the
suite). Units no longer merge across a chapter boundary, which is deliberate (a
chapter boundary is a real time surface) and whose cost is bounded to ≤ K−1 extra
breaks per continuously-depositing column — measured, not assumed (§ SPIKE 2).

**4. Addressed-draw family re-addressed.** The design named `0x5B00_*` for the
per-chapter draws, but the biotic layer already owns that high byte
(`SALT_BIO_FIRE`/`FLOOD`). The tectonic layer takes a fresh `0x5D00_*` so the
address spaces still never collide.

**5. Advection is Eulerian, the stated casualty.** Columns do not carry their
record sideways, so there is no terrane docking and no lateral strike-slip offset
of landscapes. This is the permanent simplification § 13 accepted; the fault
*record* (a `StructEvent` species) is where transform motion will live when the
punctuation designs land.

## What the measurements say

See docs/spikes/S12-results.md for the tables. The headlines: byte-identity off
holds against every tectonic knob varied; determinism and scalar↔parallel
byte-identity hold on; the forcing's gradation-to-peak collapses from the ~50 km
smear to 20.9 km at the default `W=25` (and tracks `W` linearly); both mass
ledgers balance; the recorder grows ~3.2× at K=8 (bounded, stated); and the
ritual's only new per-iteration cost of note is the isostasy smoothing (8 ms/iter
→ ~1.6 s), so the whole thing is a **~1.13× ritual** (16.2 s vs 14.3 s at
Medium 200), inside the design's estimate.

The one genuinely surprising measurement is **exhumation**: even under 2.9 km
belts over 300 iterations the deep sim strips only ~4–11 m of bedrock, so the
exhumed-metamorphic-core signature (§ 6.4) does **not** yet reach the surface.
That is not a bug — it is the same "thicknesses are thin because supply and time
are compressed" loose end S9 already flagged, now proven to gate a *landform*, not
just a bed thickness. The tell-tale is that mountains are cheap to raise
(amplitude) but their roots are expensive to expose (denudation rate × time).
Legibility of the landform reads is therefore gated on **two** user-owned calls —
the amplitude (U7) *and* the erosion/supply calibration — not one. That is the
most useful thing this spike learned for the integrating session.

## What was cut, and stays honest about it

- **Rebound-persistence half-life (§ SPIKE 4f)** and the **isostasy-off foreland
  control (§ SPIKE 4c)** are reported as proxies rather than the full
  controlled experiments — there is no "thickening without isostasy" mode (the
  two do not flip separately by design), so the clean mechanism-attribution
  control would need a bespoke harness mode. Flagged in the results doc.
- **The full biharmonic flexure** (forebulge, Te fields) is not built — the v1
  smoothed-load Airy is the ratified minimum, promoted only if the foreland read
  fails.
- **Punctuation hooks, fault events, angular-unconformity rendering** are
  interface-only in the design and out of this spike's scope; the chapter table
  and `exhum` are exported so those passes have their axes.

The production flip of `tectonic_history` is the user's appearance call, from the
§ SPIKE 7 relief renders — exactly like the 0030 erodibility flip, and it changes
terrain shape for every world made afterward.
