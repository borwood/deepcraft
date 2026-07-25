# 0105 — Two draws, one number

*2026-07-25. Background agent, isolated worktree. User-ratified: "this needs
fixed either way. **Decorrelate.**" Plus, and this turned out to be the whole
entry: "I would like to know whether banding is visible and what that means for
the before-after."*

> blogworthy — lens 2 (procgen against the backdrop of priors) and lens 3
> (reflexions in a deepsim codebase): **a defect that was 100 % real at the
> level it was written about and 0 % consequential at the level anyone could
> see — standing next to a second defect, in the same four lines, that nobody
> had named and that was worth 1.49× the variance of every contact voxel at a
> weathering front.** The instrument that proved the first one harmless is the
> instrument that found the second one.

## The claim

A voxel at a weathering front makes two stochastic decisions. `allocate_partial`
splits its eight eighths between the recorded bands that overlap its 0.9 m span.
Then, for each band that carries a **loose pore rider** — a weathering product
sitting in its parent rock's pores — `pore_rider_share` splits that band's
winnings between parent and product, `cnt · k8 / 8` stochastically rounded so the
product's share stays *proportional* to its host's (journal/0099).

Two decisions, so two draws. The code had one, and said so in a comment:

> The offset is a **low digit** of the voxel's own fill draw, not its high bits:
> `allocate_partial` consumes the high end, and reusing it here would correlate
> "this band won an extra eighth" with "the product won an extra eighth of it"
> into a visible pattern.

That comment is a justification with a testable claim inside it, which is a
pleasant thing to find. journal/0103 read the arithmetic and found the claim
false: `(u * 4096.0) as u64 & 7` keeps fractional binary digits 10–12 of the
draw, and `allocate_to`'s `uq = (u * ONE) as u64` is the **top twenty** — which
contains them. In the numbering the probe uses (bits of the 20-bit allocation
offset, from the bottom) they are bits **8–10**. Same three bits, two namings;
worth stating both, because reading the two documents side by side otherwise
looks like a contradiction.

So the file said "disjoint" and the arithmetic said "a sub-range". Filed to
ROADMAP Observed, not fixed, because re-addressing the rider moves every contact
voxel in the world — an appearance change, and the user's call. The user's call
came back: decorrelate, and *tell me whether the banding the comment feared was
ever there*.

## "They share bits" is a code reading. What is the number?

`examples/pore_decorrelation_probe.rs` reconstructs the whole decision off the
public generation path — `column_record` → `ColumnFill` → `allocate_partial` →
`pore_rider_share` — mirroring `mixed_at` arm for arm, and reproduces the
**retired** two-line offset formula beside the new draw. So before and after are
measured on the same voxels of the same world in one run, rather than across two
checkouts where any other difference could creep in. Production world, seed 1337,
`Extent::Medium`, 48 chunk-columns at the strongest weathering cells:
**464,521 rider decisions in 276,820 voxels**, out of 2,700,288 recorded voxels.

**Measure 1 — predictability.** How often can the pore offset be read straight
out of the offset the allocation consumed? Not "do the declared bit ranges
overlap" — that is a claim about today's `FRAC_BITS` and rots the moment someone
widens a field. Slice the allocation offset every way it can be sliced, all
eighteen 3-bit windows, and ask how often each window *equals* the pore offset:

```
      shift   retired      decorrelated
          0     12.49 %        12.48 %
          …
          8    100.00 %        12.52 %   <== the coupling
          …
         17     12.53 %        12.54 %
```

**100.00 %.** The pore offset was not merely correlated with the allocation's
offset, it was a **deterministic function** of it: three bits of the allocation's
twenty, zero conditional entropy, one number doing two jobs. That is the
quantification, and it is as bad as a coupling can be.

**Measure 2 — the thing the comment actually feared.** Which was not "the two
offsets share bits", it was *"this band won an extra eighth" correlating with
"the product won an extra eighth of it"*. Both decisions are roundings, so both
have a residual with mean zero: `cnt − entitlement` for the allocation, and
`g − cnt·k8/8` for the rider. Correlate them.

```
allocation residual vs pore residual   r = +0.0006 (retired)   +0.0012 (decorrelated)
  mutual information                     0.0609 bits            0.0610 bits
                                         (estimator floor, measured: 0.0610 bits)
dither entropy given (band, allocation)  2.999 bits             2.999 bits   (max 3.000)
  worst single group of 64               2.995 bits             2.997 bits
```

**Nothing.** `r` is six ten-thousandths. The mutual information sits *on* the
finite-sample floor — measured by running the same estimator over the same
samples with an offset drawn from an address that cannot be related to the fill
draw at all. And the dither's entropy conditioned on everything the allocation
decided about that band is 2.999 bits of a possible 3.000: knowing what the
allocation did tells you essentially nothing about what the rider will do.

The reason is aliasing, and it is worth spelling out because the intuition points
the other way. The allocation's decision is a **contiguous interval** in its
20-bit offset — a material takes an extra eighth exactly when the offset falls in
a window whose width is that material's fractional remainder. Bits 8–10 are a
**fast sawtooth** across that space, cycling through all eight values every 2,048
counts. So conditioning on the allocation's interval leaves the pore offset
uniform, unless the interval is *narrower* than 2,048 of 1,048,576 — a fractional
remainder under 0.2 % of an eighth. Real contacts are nowhere near that thin. The
coupling was total at the bit level and invisible one level up, because the bits
that were shared were the wrong bits to matter.

That is a genuinely uncomfortable result to sit with: the comment was **wrong
about the mechanism it described** and **right about the outcome it promised**,
for a reason it did not know.

## The defect that was actually there

Looking for the harm the comment named is how the harm it did not name turned up.
`mixed_at` draws `u` **once per voxel** and then loops over the events the
allocation returned. Every one of them got the same three bits.

A weathering front is not one band. It is many thin bands of one parent member
differing only in their pore share — that difference *is* the gradient, which is
why journal/0099 widened `ColumnFill`'s merge key to keep them separate instead
of collapsing them into one event. So a contact voxel there routinely carries
several rider decisions at once. On a shared offset each one is the same monotone
step function of that offset (`residual = 1{off ≥ 8 − s} − s/8`), so they all
round the **same direction**, every time.

```
sibling riders IN ONE VOXEL       r = +0.4878 (retired)   −0.0012 (decorrelated)
                                  over 187,701 sibling pairs
Var(voxel total) / Σ Var(rider)     1.488×                 0.999×
```

Their errors **added** instead of cancelling. A multi-band contact voxel's total
product carried **1.49× the second moment** independent roundings give — 1.22× the
spread — while every argument in the corpus for why the front's mass is trustworthy
(journal/0055, journal/0103) is an argument about *independent* unbiased roundings
whose errors cancel over a neighbourhood. The mean was never wrong. The variance
was, by half again, in exactly the voxels the front is expressed through.

Nobody wrote a comment about that one, so nothing had to expire for it to be
wrong. It is the shape spines.md § A-2 is about with the premise removed: not a
justification that outlived its premise, but a justification that was **never
true and could not be checked**, because a claim about bit ranges has no gate.

## The fix, and why it is hard to un-do

Three things, and only the first is the fix; the other two are what stop it
coming back.

1. **Its own salt.** `SALT_GEO_PORE`, and `pore_draw(seed, vx, vy, vz, event)`.
   Domain separation by salt rather than by bit range is the form that cannot
   rot: two salts stay independent whatever widths either offset grows into,
   where a carve-out ("take the low digits, they leave the high ones alone") is
   only disjoint for the widths it was written against.
2. **The event index in the address.** This is what fixes the defect that
   mattered — one band's decision is now independent of its neighbours' inside
   the same voxel. `event` is the index into the record's own canonical
   `StrataRec::events` order, not an iteration order, so it is an address like
   any other.
3. **A type that makes the old shape unrepresentable.** `pore_rider_share` no
   longer takes an `f64`; it takes a `PoreDraw`, whose field is private to
   `fill.rs` and whose only constructor is `pore_draw`, which is the only place
   `SALT_GEO_PORE` is spelled. Handing it the fill draw is now a type error.
   Re-correlating the two draws takes a deliberate edit to that file rather than
   an innocent one at a call site.

`pore_rider_share` also moved out of `collapse.rs` and into `fill.rs`, next to
`allocate_partial`. They are the two quantizers of one voxel; the claim that they
are independent is checkable at a glance only when they are on the same screen.

And the claim is now **gated**, three ways. In `fill.rs`, over 40,000 real voxel
addresses: no 3-bit window anywhere in the fill offset predicts the pore offset
better than chance, and sibling bands in one voxel collide at 1-in-8. In the
probe, the same two assertions against the world's actual records — plus, as the
control that keeps the test honest, an assertion that the **retired** formula
still scores 100 % on the same data. A before/after inside one run: if the probe
ever stops reproducing the code it is the control for, it says so.

## Is the banding visible? No.

Banding is spatial structure, so the instrument has to measure spatial structure.
The right frame is one chunk-column: all 1,024 of its voxel columns share **one
strata record and one fill plan**, so across that 32×32 field the *only* thing
that varies is the draw. If two coupled draws print a pattern anywhere, they
print it there.

```
      lag    retired (x, z)        decorrelated (x, z)
        1    +0.0005, -0.0261     -0.0099, +0.0113
        2    +0.0137, +0.0054     -0.0283, -0.0131
        3    -0.0357, +0.0322     -0.0537, -0.0320
        4    +0.0152, +0.0216     +0.0184, -0.0656

  mean same-sign run along x:  1.889 (retired)   1.947 (decorrelated)   [2.000 = none]
```

Every autocorrelation is inside ±0.07 of zero, before and after; the run lengths
bracket the no-structure value of 2.0 from below on both sides. The ASCII maps
the probe prints are two fields of salt and pepper that no eye could tell apart.
**The artifact was imperceptible.** Not "too subtle to bother with" —
*structurally absent*: both offsets are functions of a position hash, so both
fields are white noise, and a per-voxel dependency between two decisions **at the
same voxel** cannot produce structure **between** voxels. It could only have
shown up as banding if it had killed the dither at a given depth, which measure 2
above rules out at 2.999 bits of 3.

So: the fix was correct, the comment's fear was unfounded, and the thing that was
actually broken was invisible for a different reason — a variance is not a
pattern. All three of those deserve to be written down, because each of them
alone would invite someone to re-litigate this.

## What moved

Every contact voxel with a loose pore rider re-rolls, which was ratified up front:
*"we do not care about the current arbitrary state of the world… it is a scratch
sheet while we build a world-building engine."* Goldens are a regression detector,
not a specification — so they are re-baselined here, deliberately, with the size
of the move stated rather than the hashes quietly swapped.

- **32.0 % of rider decisions changed** (148,590 of 464,521). Two thirds are
  unchanged because a re-rolled offset lands on the same side of the threshold
  five times in eight on average.
- **45.5 % of rider voxels changed** (125,899 of 276,820) — **4.66 % of the
  2,700,288 recorded voxels** in the sample.
- **Net +332 product eighths over 464,521 decisions**, `+0.0007` eighths each.
  A redistribution, not a gain: the estimator's mean is `cnt · k8 / 8` under both
  offsets, and `fill.rs` now asserts that by enumerating all eight.
- **Visible character: none.** Where a contact voxel changed, it changed by one
  eighth of product between parent and product *within the voxel* — a mixture
  the renderer already draws, at a contact it already drew, in a field that was
  white noise before and is white noise after. The world is not different to look
  at; it is differently right.

Golden hashes, `tests/contents_contract.rs` — the block, material and mixture-table
fingerprints over the sampled world:

GOLDEN_TABLE_PLACEHOLDER

## What this cost to find

One probe, 28.6 s on the production world, and the answer to a question that had
been sitting in Observed as "cheap next step: a fullbright walk along a strong
front looking for banding". That walk would have found nothing and proved nothing
— a null from an eye, on a signature that turns out to be structurally incapable
of existing. The autocorrelation of a 32×32 contact plane is the instrument that
can see the question (CLAUDE.md's rule, applied to a statistic instead of a
render pass), and it is also the only one that could have distinguished "no
banding" from "banding I did not notice".
