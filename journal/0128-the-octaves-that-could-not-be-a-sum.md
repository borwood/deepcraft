# 0128 — The octaves that could not be a sum

*2026-07-29. E5 member #0, continuation slot (a): the octaves `DitherSource`.
`docs/dependency-graph.md` E5 / P4 / P6. The kernel half of a pair — its consumer
lands in journal/0129, same worktree, same evening.*

---

## The instruction was five days old and it was right

On 2026-07-24 this project diagnosed the near-field material squares twice. The
first diagnosis was wrong and became corrections #45: *the member dither is
anchored to a chunk with no awareness of its neighbours*. Reading
`interp_select_draw` killed it — the field hashes **absolute** chunk corners and
interpolates bilinearly, so it is world-anchored and C0-continuous across every
seam. The second diagnosis, from the same afternoon, was right and has been sitting
in the corpus ever since: it is **a single octave of value noise at chunk
wavelength**. *"Fix = octaves, not resolution."*

That sentence then went missing twice. It was settled in a close block, never
stamped onto the audit that asked the question, re-derived by a design agent five
days later, and caught only by the user's memory (corrections #73, and the reason
read-first item 5 now covers close blocks). This entry is the fix arriving.

It is also, independently, what a user field report asked for on the morning it
was written: *"the bilinear noise does not actually approximate what loaded
chunks look like well, it sticks out poorly. So there's a more foundational issue
to that."* Two sites, one missing property — **structure at more than one scale**.

## What a single octave actually does, as a number

The mechanism is worth stating precisely, because the precise version is what
makes it testable.

A bilinear corner field is *linear along x inside a cell*. Not smooth — linear.
So its second difference along x is **exactly zero** everywhere except where the
three-point window straddles a cell line, and there it is the slope change. All
of the field's curvature — every kink in every contact it draws — lives on one
lattice, and for the member dither that lattice was the 32-voxel chunk.

Measured on the shipped source (`draws.rs`, mean `|u(x−1) − 2u(x) + u(x+1)|`
over a 1000×64 voxel window):

| | on the 32-lattice | off it | ratio |
|---|---|---|---|
| `Coherent`, stride 32 (shipped) | **8.440e-3** | **8.441e-17** | **1.0e14** |
| `Octaves` (this slice) | 4.472e-4 | 3.287e-4 | **1.36** |

`8.4e-17` is float rounding residue on a function that is affine in the interval.
A ratio of 10¹⁴ is not a bias or a tendency; it is the field having exactly one
grid in it. That is the 28.8 m square-edge signature, and it is now a gate test.

*(The first version of that test bucketed by `x ≡ 0 (mod 32)` and measured
2.7e-4 off-lattice, which had me briefly believing a bilinear field has interior
curvature. It does not: **phase 31 straddles a line too**, because `x + 1` is in
the next cell. The predicate is "does this window cross a lattice line", not
"is this point on one" — a two-phase mistake in a three-point stencil.)*

## The sum you would write first is unusable, and arithmetic says so

"Octaves" in graphics means fractional Brownian motion: add L copies of the noise
at halving wavelength and halving amplitude, normalise. I was two lines from
writing that when the distribution question surfaced, and the answer is that the
obvious construction is not slightly wrong here — it is **catastrophically** wrong,
in a way no frame would have diagnosed cleanly.

A `DitherSource` does not feed a displacement. It feeds an **inverse-CDF draw**:
`ShareVec::draw(u)` picks class `i` when `u` lands in that class's band of the
unit interval. So the *marginal distribution* of `u` is not a cosmetic property —
it **is** the mapping from recorded shares to rendered fractions. A uniform `u`
renders share 0.6 as 0.6; anything else renders it as something else.

Six octaves of bilinear uniforms is a weighted sum of **24 independent uniforms**.
Normalised to the unit interval its standard deviation is

```
σ = sqrt(Σ aₖ² · Σⱼ wⱼ² / 12) / Σ aₖ  ≈  0.085
```

— a narrow bell around ½ that essentially never leaves `[0.25, 0.75]`. **Any
class whose CDF band lies outside the middle half of the interval would never be
drawn at all.** The dice-sum distortion that corrections #39 had to correct the
*sign* of (`Coherent` amplifies the majority; it does not flatten it) does not
merely get worse as octaves are added — it stops being a bias and becomes a
truncation. And it gets worse *monotonically*: independent summands multiply the
density's Fourier coefficients toward a Gaussian, and a Gaussian on a bounded
interval is the opposite of uniform.

This is the same lesson as the 9/16 integral in journal/0125, arriving at a
different joint: **when a mechanism's effect is a distribution, reasoning about
its typical value tells you nothing.** There the doc comment described the
behaviour at an extreme and hid a 44 % effect. Here the word "octaves" describes
a spectrum and hides a distribution collapse. Both were caught by doing the
integral rather than by re-reading the sentence.

## Use the Gaussianity instead of fighting it

The fix is not to correct the sum. It is to notice that the sum *wants* to be
Gaussian and let it: **draw the corner values from a unit normal instead of a
uniform.**

A bilinear blend of independent normals is normal — exactly, not asymptotically —
and so is a weighted sum of those blends, with a variance that is a closed form of
the interpolation weights:

```
S(p)  = Σₖ aₖ · Σⱼ wₖⱼ(p)·gₖⱼ ,   gₖⱼ ~ N(0,1) iid
σ²(p) = Σₖ aₖ² · Σⱼ wₖⱼ(p)²  =  Σₖ aₖ² · sx·sz ,   sx = (1−fx)² + fx²
u(p)  = Φ( S(p) / σ(p) )
```

`Σⱼ wⱼ²` factorises into two one-dimensional terms, so the position-dependent
variance costs two multiplies per octave. `S/σ` is standard normal at **every**
position, so `u` is **uniform at every position** — and the unbiasedness a
`ShareVec` consumer needs becomes a property of the construction rather than a
tolerance somebody measured once.

Then the literature walked in and said this was already a method. Thresholding a
Gaussian random field at the quantiles of target proportions is
**truncated-Gaussian facies simulation**, which is how geologists actually
simulate facies distributions from measured proportions. The record supplies the
proportions, `ShareVec::draw` supplies the thresholds, and this supplies the
field. We arrived at it from the wrong end — from a distribution bug in a dither
— and that is a fair description of most of this project's geology.

Measured, same instrument as `coarse.rs`'s own law test, 10,000 independent
positions:

| source | renders a 0.6/0.4 share vector as | marginal KS deviation |
|---|---|---|
| `Coherent` (shipped) | **0.6829** | — |
| `Octaves` (this slice) | **0.6053** | **< 0.02** |

So `Octaves` is the **first unbiased coherent source in the tree**, and
corrections #39's majority amplification turns out to be a property of
`Coherent`, not of coherence. That matters more than a bookkeeping fix: a source
that pushes splits toward the majority erases exactly the minority members the
near-path slice exists to let through. The bias was pointing *against* the
slice's purpose.

Two approximations remain and both are bounded and measured rather than asserted:
corner normals come from a 1024-entry midpoint-quantile table (renormalised so its
variance is exactly 1, because the variance formula *assumes* that and a midpoint
rule under-weights the tails it truncates), and `Φ` is Abramowitz & Stegun 7.1.26
at `|ε| ≤ 1.5e-7`. The KS bound is derived, not fitted: 1.63/√10000 = 0.0163 at
99 %, plus ~1e-3 of approximation, so the line is 0.02.

## Primes, because powers of two would have kept the lattice

The strides are `509, 257, 127, 61, 31, 13, 7, 3` voxels — **distinct primes**,
ratios ≈ 2.

A dyadic ladder starting at 512 would put every coarse octave's kinks *on top of*
the fine ones, so a 32-voxel lattice would still be in the field, just with less
amplitude. Distinct primes share no common multiple below their product, so no
lattice survives at any scale a player can see. The measured on/off ratio at the
chunk scale is **1.36** against `Coherent`'s **1.0e14**. Each octave also carries a
fixed offset, because coprime strides still all pass through the origin.

The head of the ladder is deliberate: **509 voxels is 458.1 m at the N=2 player
scale — the ~460 m deep cell.** The selection field varies at the scale of the
thing it is selecting *within*, and below. The foot is deliberate too: 13 voxels
is 11.7 m, and a "member" patch thinner than about ten voxels is not a rock body,
it is per-voxel speckle — a scale that already belongs to the eighths-allocation
draw.

**Persistence is 0.6, and it is derived rather than tuned.** Amplitude `pᵏ` on a
lacunarity-2 ladder is an fBm of Hurst exponent `H = −log₂ p`, and the level sets
of a 2D fBm — which is exactly what this field's contacts are — have fractal
dimension `2 − H`. Published fractal dimensions for traced geological boundaries
cluster at **D ≈ 1.2–1.3**; `p = 0.6` gives `H = 0.737` and `D ≈ 1.26`, inside the
band. It is a plausibility anchor from the literature, not a fit to a deepcraft
measurement, and the code says so — but it is the difference between a constant
with a derivation and a constant chosen because a frame looked right (CLAUDE.md
§ *a closed system cannot detect its own scale error*).

## The variogram is the picture

`V(L) = E[(u(p+L) − u(p))²]`, both sources, same seed and domain:

| lag (voxels) | `Coherent` stride 32 | `Octaves` |
|---:|---:|---:|
| 1 | 0.00011 | 0.00002 |
| 4 | 0.00159 | 0.00026 |
| 16 | 0.02086 | 0.00280 |
| 32 | 0.05709 | 0.00720 |
| 64 | 0.07443 | 0.01737 |
| 128 | 0.07644 | 0.03474 |
| 512 | 0.07498 | **0.09704** |

The single octave **saturates at two strides** (not one — at lag 32 the two points
still share the cell corner between them) and is then flat forever: 0.0744,
0.0764, 0.0750. It has exactly one characteristic length, that length is the
chunk, and *that* is what the eye reads as a patch size. The ladder is still
climbing at 512 and is *smoother* than the single octave at every short lag —
which is the shape a facies map has: large domains, gradational interiors,
detail at the contacts.

## What this slice does NOT do

It has **no visible outcome**, and labelling that honestly is part of shipping it.
`Octaves` is a kernel with no production caller: nothing in the world reads it
until journal/0129 wires it into the near path's member dither. No world moved, no
golden moved, and no frame changed. The tests are its only consumers, which is
exactly the state journal/0125 spent seven days regretting on `CoarseField` — the
difference is that the consumer is in the same worktree and the same evening, and
the pair is named as a pair on the board.

It also does not touch the far site. `surface_class` keeps `Coherent`, because the
far register's blend semantics were **rejected by the user on 2026-07-29** and its
heir is a separate design conversation (a far register derived from
refinement-operator budgets). Swapping the far source under a rejected semantics
would be changing the answer to a question that is being re-asked.

## The shape worth keeping

**A "well-known" construction can be well-known for a different problem.** fBm by
summation is correct for displacement and wrong for a CDF argument, and nothing in
the name tells you which. The tell was asking *what consumes this number* —
`ShareVec::draw` consumes it as a probability, and a probability's distribution is
its whole contract. **Ask what the number is FOR before importing the recipe.**

**The distribution of a mechanism is a first-class part of its interface.** This
is now three times: corrections #39 (the coherent source's bias sign),
journal/0125 (the 9/16 expectation hidden behind a pointwise doc comment), and
this (a sum whose spread collapses). The general form: *a mechanism whose output
is consumed as a probability owes its marginal, and a mechanism whose cost is a
distribution owes an expectation — a limit or a typical case will mislead every
reader including its author.*

> blogworthy: **lens 2 (procgen against the backdrop of priors)** — "add octaves"
> is the most standard advice in procedural generation and it is *wrong* when the
> noise feeds an inverse-CDF draw rather than a height; the fix is the
> geostatisticians' truncated-Gaussian facies simulation, which we re-derived from
> a distribution bug. Also **lens 3 (reflexions in a deepsim codebase)** — a
> single-octave field has all its curvature on one lattice by 10¹⁴, which turns a
> five-day-old prose diagnosis ("the squares are one octave at chunk wavelength")
> into a number and then into a gate test. And **lens 1 (AI-native development)** —
> a correct diagnosis that went missing twice inside its own corpus, and the
> read-first rule that now exists because of it.
