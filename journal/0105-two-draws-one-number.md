# 0105 — Two draws, one number

*2026-07-25. Background agent, isolated worktree. User-ratified twice: first
"this needs fixed either way. **Decorrelate.**", then — after seeing what the
defect actually was — "in fact we could have some utility system that is a
**randomness provider** for a function, ensuring that **every caller gets
allocated their own band of the hash** … it would be optimal to fix it with a
**construction guarantee**."*

> blogworthy — lens 2 (procgen against the backdrop of priors) and lens 3
> (reflexions in a deepsim codebase): **a defect that was 100 % real at the
> level it was written about and 0 % consequential at the level anyone could
> see, standing next to a second defect in the same four lines that nobody had
> named and that was worth 1.49× the variance of every contact voxel at a
> weathering front — and the realisation that the fix for both was not a better
> comment but a mechanism that makes the comment unnecessary.**

## The claim

A voxel at a weathering front makes two stochastic decisions. `allocate_partial`
splits its eight eighths between the recorded bands that overlap its 0.9 m span.
Then, for each band carrying a **loose pore rider** — a weathering product
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
offset, counted from the bottom) they are bits **8–10**. Same three bits, two
namings; worth stating both, because the two documents otherwise look like they
contradict each other.

So the file said "disjoint" and the arithmetic said "a sub-range". Filed to
ROADMAP Observed, not fixed, because re-addressing the rider moves every contact
voxel in the world. The user's call came back: decorrelate, and *tell me whether
the banding the comment feared was ever there*.

## "They share bits" is a code reading. What is the number?

`examples/pore_decorrelation_probe.rs` reconstructs the whole decision off the
public generation path — `column_record` → `ColumnFill` → `allocate_partial` →
`pore_rider_share` — mirroring `mixed_at` arm for arm, and reproduces the
**retired** two-line offset formula beside the new draw. So before and after are
measured on the same voxels of the same world in one run, rather than across two
checkouts where any other difference could creep in. Production world, seed 1337,
`Extent::Medium`, `--weather-inventory`, 48 chunk-columns at the strongest
weathering cells: **464,521 rider decisions in 276,820 voxels**, out of 2,700,288
recorded voxels.

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
have a residual with mean zero: `cnt − entitlement` for the allocation,
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
uniform unless the interval is *narrower* than 2,048 of 1,048,576 — a fractional
remainder under 0.2 % of an eighth. Real contacts are nowhere near that thin. The
coupling was total at the bit level and invisible one level up, because the bits
that were shared were the wrong bits to matter.

That is an uncomfortable result to sit with: the comment was **wrong about the
mechanism it described** and **right about the outcome it promised**, for a
reason it did not know.

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
product carried **1.49× the second moment** independent roundings give — 1.22×
the spread — while every argument in the corpus for why the front's mass is
trustworthy (journal/0055, journal/0103) is an argument about *independent*
unbiased roundings whose errors cancel over a neighbourhood. The mean was never
wrong. The variance was, by half again, in exactly the voxels the front is
expressed through.

Nobody wrote a comment about that one, so nothing had to expire for it to be
wrong. It is spines.md § A-2 with the premise removed: not a justification that
outlived its premise, but a justification that was **never true and could not be
checked**, because a claim about bit ranges has no gate.

## The user's escalation: stop fixing call sites

The narrow fix was a distinct salt for the pore rider, which is right and small.
The user's response reframed it, and the reframing is the actual content of this
entry:

> *"we could have some utility system that is a **randomness provider** for a
> function, ensuring that every caller gets allocated their own band of the hash
> … it would be optimal to fix it with a **construction guarantee**."*

The point is that the old convention was: *pick a salt constant by hand, or slice
some bits out of a neighbouring draw, and write a comment claiming disjointness.*
Every part of that depends on the next person reading the comment correctly — and
we have the receipt showing they don't, because the comment was wrong from the
day it was written and survived a full diagnostic pass (journal/0103 read it,
believed the phrasing enough to file it as a *loose end* rather than a bug, and
only the arithmetic caught it). **A convention that relies on care is the same
defect one layer up.**

So the deliverable became a mechanism. `dc_sim::statistical::rng`:

```rust
pub trait Domain { const SALT: u64; const NAME: &'static str; }

pub struct Draws { seed: u64, salt: u64 }
impl Draws {
    pub fn of<D: Domain>(seed: u64) -> Self { … }
    pub fn unit(self, addr: &[u64]) -> f64 { … }
    pub fn bits(self, addr: &[u64]) -> u64 { … }
}
```

and a macro that declares a crate's whole set at once:

```rust
dc_sim::draw_domains! {
    /// The eighth allocation in a mixed voxel.
    GeoFill = 0x5700_000F;
    /// The pore rider's rounding offset.
    GeoPore = 0x5700_0011;
    …
}
```

**Where the guarantee actually lives.** Two ways to collide, both closed at
compile time:

- **Reusing a salt** — every salt is spelled exactly once, inside the list, and
  the macro emits `const _: () = assert!(all_salts_distinct(ALL_DOMAINS));`. A
  copy-pasted number is a build failure, not a subtly correlated world. (Const
  evaluation runs the pairwise loop; the diagnostic names the duplicate.)
- **Reusing a name** — each entry generates a type, so a repeated name is a
  duplicate definition.

And a call site cannot reach a stream by writing a number at all: `Draws::of::<D>`
takes a *type*. Reuse remains available — two call sites of one conceptual draw
*should* share a domain — but only by writing that domain's name, which is the
deliberate act rather than the accident.

**Explicit salt values, not ordinals.** The tempting version derives each salt
from its position in the list, which makes duplicates structurally impossible
rather than merely rejected. It is the wrong trade here: these numbers are baked
into every world ever generated, so a list whose values move when you sort it
would silently re-roll the planet. Values are written out; the compiler, not the
reader, checks they are distinct.

### Converted, all of it

Twenty-six hand-rolled salts existed across three files and three unrelated
numbering prefixes (`0x5700_*` in `pregen/mod.rs`, `0x5900_*` in `deeptime/grid.rs`,
`0x5B00_*` in `deeptime/biotic.rs`) — three people each inventing a prefix and
hoping. They are now one list per crate:

- **dc-worldgen** (`src/draws.rs`, 20 domains): every call site converted —
  `pregen/{mod,tectonics,history}.rs`, `geology.rs`, `collapse.rs`, `fill.rs`.
  `interp_select_draw` became `draws::interp_corner_field(Draws, …)`.
- **dc-sim** (`statistical/world.rs`, 6 domains): `Home`, `Danger`,
  `FrontierPrior` and `Collapse` converted at their call sites.

**Every world is byte-identical across the conversion.** `Draws::bits` folds
`(seed, salt, addr…)` through exactly the same splitmix chain `mix(&[…])` did, so
`Draws::of::<Plate>(seed).unit(&[p, 0])` *is* `draw_f64(&[seed, SALT_PLATE, p, 0])`
— and the S7 byte-identity goldens are the proof, unmoved.

### The three holes, all named rather than papered

1. **Tag space inside a domain is still hand-laid.** `GeoSelect` addresses four
   veneer passes as tags 0–3; `GeoAccessory` uses `tag` and `tag + 1024` for its
   presence gate and its selection. Those are hand-rolled *sub*-domains with the
   same failure mode at smaller scale. A `Domain` per decision would fix it and
   the cost is a longer list. Named in `draws.rs`, not fixed.
2. **Two `dc-sim/engine.rs` draws are not on the provider.** Their address is
   `[seed, k, SALT, r, t]` — the sample index sits *before* the domain, and
   `Draws::of` fixes the domain at slot 2, so converting them changes the key and
   re-rolls every world's history layer. The salt still has exactly one spelling
   (`<domains::RegionStep as Domain>::SALT`); the move is deliberate and
   world-changing and is sequenced on its own, not smuggled into a slice about
   the pore rider. Marked loudly in code.
3. **A recorded salt is data, not a choice.** `StrataEvent::sel_salt` persists
   which stream an event's member was selected from, so the per-voxel member
   dither can re-run *that* selection later. `Draws::from_recorded_salt` exists
   for exactly that, with a name long enough to be friction, and a doc comment
   saying replaying a recorded address is reading rather than issuing.
4. **`deeptime/`'s three domains are registered but their call sites still spell
   the constant locally.** Registering them is what makes the uniqueness check
   *total* — nothing can now re-issue `0x5900_0001` — and
   `the_deeptime_constants_agree_with_their_registered_domains` keeps the two
   spellings in agreement (the "a summary is not an authority" pattern: the list
   is the authority, the local `const` is the copy, and a test asserts they
   agree). Converting three call sites is a follow-on, left to whoever holds
   those files today.

## The pore rider, riding on it

`GeoPore` is the provider's first customer, and it takes two separations, not one:

1. **Its own domain**, which makes it independent of `GeoFill` whatever bit
   widths either offset grows into.
2. **The event index in the address** — this is what fixes the defect that
   mattered, making one band's decision independent of its neighbours' inside the
   same voxel. `event` indexes the record's own canonical `StrataRec::events`
   order, not an iteration order.

Plus a type that makes the old shape unrepresentable: `pore_rider_share` no
longer takes an `f64`, it takes a `PoreDraw`, whose field is private to `fill.rs`
and whose only constructor is `pore_draw`. Handing it the fill draw is a type
error. And it **moved out of `collapse.rs` into `fill.rs`**, beside
`allocate_partial` — the two quantizers of one voxel are checkable at a glance
only when they are on one screen.

The claims are now gated, and the gate carries its own control (**+25.2 s of
gate wall-clock**, four chunk-columns of a Medium world). In `draws.rs`:
two domains at the same address correlate at `|r| < 0.03` and agree to three bits
at chance. In `fill.rs`, over 40,000 real voxel addresses: **no** 3-bit window
anywhere in the fill offset predicts the pore offset better than chance, and
sibling bands in one voxel collide at 1-in-8. In the probe, the same assertions
against the world's actual records — plus an assertion that the **retired**
formula still scores 100 % on the same data, so a before/after lives inside one
run and the control cannot silently stop being the control.

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
shown up as banding if it had killed the dither at a given depth, and measure 2
rules that out at 2.999 bits of 3.

So: the fix was correct, the comment's fear was unfounded, and the thing that was
actually broken was invisible for a different reason — a variance is not a
pattern. All three deserve writing down, because each alone would invite someone
to re-litigate this.

## What moved — and the surprise

Nothing in any golden. That was not the expectation going in, and the reason is
worth more than the fix.

`weather_inventory` is **off by default** (`grid.rs`: *"the production flip is
the user's"*), and the weathering front is the only producer of a *loose* pore
rider. So on the world every golden hashes there is **no pore-rider decision at
all** — the arm never executes. Measured, not inferred, by running the same
census on the same chunk-columns of the same seed with the flag off (Part 5 of
the probe):

```
weather_inventory OFF (the shipped flip):        0 rider decisions in  54,272 Mixed voxels of 1,950,720 recorded
weather_inventory ON  (this probe's world): 464,521 rider decisions in 314,368 Mixed voxels of 2,700,288 recorded
```

Fifty-four thousand contact voxels on the default world, and **not one of them**
makes the decision this slice is about.

This is corrections #51's shape again — *"the shipped world has ZERO coal; the
guard runs on a world nobody ships"* — and it is now twice in two days that a
measured claim about the world turned out to be a claim about a world behind a
flag. Worth naming as a habit: **when a slice moves nothing, check whether the
thing it moves exists in the default build before congratulating yourself on
byte identity.**

Where the world *does* move — behind `--weather-inventory`, which is where every
weathering number in the corpus was measured — the size of the move is:

- **32.0 % of rider decisions changed** (148,590 of 464,521). Two thirds are
  unchanged because a re-rolled offset lands on the same side of the threshold
  five times in eight on average.
- **45.5 % of rider voxels changed** (125,899 of 276,820) — **4.66 % of the
  2,700,288 recorded voxels** in the sample.
- **Net +332 product eighths over 464,521 decisions**, `+0.0007` eighths each: a
  redistribution, not a gain. The estimator's mean is `cnt · k8 / 8` under both
  offsets, and `fill.rs` now asserts that by enumerating all eight.
- **Visible character: none.** Where a contact voxel changed, it changed by one
  eighth of product between parent and product *within* the voxel — a mixture the
  renderer already draws, at a contact it already drew, in a field that was white
  noise before and is white noise after. The world is not different to look at;
  it is differently right.

## A note on the shared build cache, because it bit four times

Every one of those runs happened in a worktree sharing one `CARGO_TARGET_DIR`
with three sibling agents, and **four separate times** the example's test target
compiled against a *sibling's* rlib rather than this worktree's — once resolving
`dc_worldgen` against a stale sibling copy (the give-away was a compiler
diagnostic pointing at `fill.rs:331`, a line number from a file where the item in
question is at 421), and once, after a `cargo clean -p dc-worldgen`, resolving
this worktree's `dc-worldgen` against a **sibling's `dc-sim`** — which of course
does not contain `draw_domains!`, so thirty errors said the macro did not exist
while it sat on screen. corrections #34 predicts exactly this; the operational
lesson is sharper than the entry currently states: **`cargo clean -p` must name
every crate in the changed dependency chain, not just the one you edited**, and a
compiler error whose *line numbers do not match your file* is the signature to
look for. Also: the file-mutex is real but not sufficient — a sibling clobbered
the lock file mid-run at 09:22, which is why re-reading it is in CLAUDE.md.

## What this cost to find

One probe, 28.6 s on the production world, and the answer to a question that had
been sitting in Observed as *"cheap next step: a fullbright walk along a strong
front looking for banding"*. That walk would have found nothing and proved
nothing — a null from an eye, on a signature structurally incapable of existing.
The autocorrelation of a 32×32 contact plane is the instrument that can see the
question (CLAUDE.md's rule, applied to a statistic instead of a render pass), and
it is the only one that could have distinguished "no banding" from "banding I did
not notice".
