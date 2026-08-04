# 0122 — The refinement that was an order of magnitude short

*2026-07-29 · fixing the hillslope-transport operator, and finding that the hypothesis three
entries had circled was right all along — including the one that was formally falsified*

> blogworthy: **lens 1 (AI-native development)** and **lens 3 (reflexions in a deepsim
> codebase)**. Four entries and two corrections converged on a defect. The third of them ran a
> clean, well-controlled experiment, got a null, and wrote the null into the corpus as a
> falsification. The experiment was sound. The null was real. And the conclusion was wrong,
> because nobody computed how far the thing being refined had to travel — the answer was two
> lines of arithmetic on a constant that had been sitting in the config the whole time.

## The state of the board

journal/0114 calibrated the erosion rates and shipped the calibration **off**, because turning
it on opened deep closed depressions. journal/0115 walked the calibrated world, found the
sizing of that defect was a **severe undercount** measured by an instrument that structurally
saturates, and re-scoped it: the pits are the tail, the defect is that adjacent cells are
oscillating against each other. journal/0116 ran three discriminators, confirmed the
checkerboard decisively, killed two proposed mechanisms — and named the register: **the flux
limiter / donor-cell partition in `erosion.rs::diffuse`.**

It also left a hypothesis, in the same deliberate shape journal/0115 had left its own:

> *A donor-cell scheme that moves everything downslope has a **period-2 mode by construction**
> — A gives all its cover to B, B is now higher and gives it back — damped only by isostasy
> downstream.* Every number above is consistent with it and none of them isolates it.

The brief for this slice said: **test that before fixing anything.** If it is wrong, that
result is worth more than a fix built on it.

## The isolation, which took twelve minutes and no world

The reason nothing had isolated it is that every measurement so far came from a world in which
eight other passes were running. So: flat bedrock, a checkerboard of regolith cover, the creep
pass and nothing else.

At the calibrated rate the arithmetic can be read off by hand. A high cell's requested outflux
is `4 · D · cover`; at `D = 5.4` that is `21.6×` its own inventory, so the limiter binds and
the cell ships **all** of it, `cover/4` per edge. Each low cell has four high neighbours and
therefore receives exactly `cover`. **The two populations swap. To the bit. Forever.**

Measured on a 32×32 fixture, in a window eight cells in from the border so the boundary's
different amplification cannot contaminate it: strict sign alternation for six steps with the
amplitude conserved to within `1e-9` of its initial value. Not a decaying oscillation — a
*mode*.

> **The hypothesis survives, exactly as written, and it is now a fixed point rather than a
> reading.**

The first draft of that test measured the whole interior and reported a sign flip on the
**shipped** arm too, which would have been a much more exciting result and was entirely an
artefact of the border cells' three-neighbour stencil leaking into the window. The instrument
was manufacturing the signature it was built to detect. That is recorded in the helper's doc
comment rather than quietly fixed, because it is the same failure this whole arc keeps
finding.

## And then the constant nobody had divided by

Writing the fix meant asking what the *correct* per-edge coefficient is, which meant doing the
von Neumann analysis that had been implicit in three entries and written down in none. For an
explicit four-neighbour Laplacian, `g(k) = 1 − 2a(2 − cos k_x − cos k_y)`, so:

- `a ≤ 1/4` — **stable**, but the grid-scale mode is reflected with its amplitude intact.
  `g(π,π) = −1` is a period-2 flip-flop that never decays.
- `a ≤ 1/8` — **monotone**. No mode may change sign at all.

Then you look at what the world actually runs:

| | `diffusion` | peak `eff_diff` | multiple of the 1/8 bound |
|---|---|---|---|
| shipped | 0.12 | **0.261** | 2.1× |
| calibrated (45×) | 5.4 | **12.60** | **100.8×** |

*(Both peaks are the post-run derivation over the whole grid. The number the last epoch's
`diffuse` actually divided down is a few per cent lower — the biotic pass rewrites `bio_resist`
**after** erosion, the lagged coupling, so a re-derivation is one epoch out of phase. The gate
asserts against the epoch-matched value, `Erosion::creep_peak_coeff`, because a check that
compares a count to a re-derivation is comparing across that lag; it cost one red gate to
notice.)*

journal/0116's D2 refined the time step **4×** and found the oscillation did not collapse,
and concluded — reasonably, and it is now `corrections.md` #63 (ii) — that this is *not* an
explicit scheme past its stability limit.

**A 4× refinement takes 100.8× past the bound to 25.2× past the bound.** Of course nothing
collapsed. The experiment was sound, the control held to three digits, the measurements are
all still good. It was simply an order of magnitude short of its own register, and **nothing
inside it could have said so**, because the distance to the bound is not a property of the sim
at any step size — it is a closed-form property of the stencil.

> *A refinement experiment measures nothing until you put its factor beside the distance the
> thing has to travel.* "We refined 4× and it did not move" is a statement about the number 4.

That is `corrections.md` #72, and it is [[measure-against-the-literature]] one level in: the
sim was checked against **itself at a different step**, when the answer was arithmetic on a
textbook amplification factor.

### The two diagnoses that were called opposites

#63's sharpest line was that *"the operator has saturated" and "the operator is unstable in
time" are opposite diagnoses, and saturation is the evidence against the second one.* It is a
good line and it is backwards. They are two halves of one behaviour, and each explains what
the other cannot:

- the **coefficient** decides that the grid-scale mode changes sign every step — that is the
  instability, and it is why the field is a checkerboard rather than merely rough;
- the **limiter** decides how big the flip is — capping export at the cell's whole inventory
  turns what would be an exponential blow-up into a **finite-amplitude period-2 limit cycle**
  whose amplitude is set by the cover.

So the limiter is exactly why the defect is deaf to `dt` — #63's real finding, and it stands
untouched — **and** exactly why it never blew up to infinity and got caught years ago.
Saturation was not evidence against instability. It was the reason the instability was
survivable enough to ship.

It also retires, without ceremony, the qualification journal/0116 insisted on: *saturation
alone is not sufficient — the shipped world's limiter binds on 88.7 % of cells and has no
checkerboard.* That was an honest observation with no mechanism. It now has one: the shipped
world sits **2.1×** past the bound with 4.6 m of cover, the calibrated world **100.8×** past
it with 41 m. Same operator, same saturation, two orders of magnitude of difference in how
much of the mode is excited and how much inventory it has to swing.

## The fix, and the two things it is deliberately not

The pass takes the epoch it is given and **splits it internally**:

```text
n = ceil( max_cell eff_diff / CREEP_MAX_EDGE_COEFF )
```

and runs `n` gather steps at `diffusion / n`. Nothing else about the inner step changes: it is
the same frozen surface, the same frozen per-cell limiter, the same antisymmetric edge flux
that made it mass-exact and order-independent to begin with.

Against the five properties the brief asked for:

1. **No overshoot** — `a ≤ 1/8` gives `g ≥ 0` for every mode, so no mode can change sign. That
   is a stronger statement than "bounded by the amount that would level the pair", and it is
   the right one: with *simultaneous* updates the pair-levelling bound is not sufficient,
   because both endpoints move. Levelling a checkerboard cell-by-cell **is** the period-2 mode.
2. **Stability independent of the rate the caller states** — yes, and by not taking the
   caller's `dt` as the integration step at all.
3. **Exact mass conservation** — unchanged and re-asserted through the new driver, including
   the species itemisation, which now has to close against the *sum* of the sub-steps.
4. **Order independence** — unchanged, and asserted bit-for-bit scalar↔parallel across
   sub-steps.
5. **Simultaneous over the neighbourhood** — unchanged; it always was.

**It is not implicit**, and that was considered. Backward Euler is unconditionally stable and
its matrix is an M-matrix, so it would satisfy 1 and 2 outright — but `h ≥ 0` makes it an
obstacle problem rather than a linear solve, and exact mass conservation through an *iterative*
solve means recovering the fluxes and re-applying them antisymmetrically anyway. At this
world's coefficients the diffusion length is ~5 cells per epoch, so an iterative solve needs
tens of sweeps: the same cost, for a much larger surface of things to get wrong. Sub-cycling
reuses a kernel that is already proved.

**And it is not a cap.** Clamping `eff_diff` at 1/8 would also remove the oscillation, in one
line, and it would reinstate exactly the one-cell-per-epoch conveyor that `stubs.md` #27 is
about. Gen time is not this project's constraint; simulation is never cheapened to buy it.

**`n = 1` is the old operator bit for bit** (`x / 1.0 == x`), which is asserted rather than
argued.

## What it did to the world

Production-Medium, seed 1337, all four arms:

| arm | sub | peak `eff_diff` | `conc(h)` rms | `conc(h)/h̄` | ACF(1) x / y | surf conc rms | surf ACF(1) x / y | hollows >1 m | >10 m | deepest | mean h | gen |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| shipped · unbounded | 1 | 0.261 | 3.42 | 0.749 | −0.102 / −0.182 | 0.21 | +0.380 / +0.269 | 0 | 0 | 0.0 m | 4.57 | 33.9 s |
| shipped · sub-cycled | 2 | 0.208 | **2.87** | 0.733 | −0.098 / −0.142 | 0.21 | +0.414 / +0.317 | **0** | 0 | 0.0 m | 3.91 | 37.6 s |
| calibrated · unbounded | 1 | 12.600 | 62.42 | 1.428 | **−0.821 / −0.853** | **40.42** | **−0.867 / −0.912** | 1380 | **818** | **112.8 m** | 43.72 | 34.7 s |
| calibrated · sub-cycled | 100 | 12.600 | **2.66** | 1.905 | **+0.001 / +0.887** | **0.30** | **+0.185 / +0.105** | 16347 | **12** | **15.8 m** | 1.40 | 238.9 s |

The checkerboard is gone. The surface's grid-scale concavity falls **40.42 → 0.30 m** and its
autocorrelation crosses from −0.87 / −0.91 (a near-pure Nyquist mode) to +0.19 / +0.11 (a
landscape). The regolith's own concavity falls **62.42 → 2.66 m**. Hollows deeper than 10 m
fall **818 → 12** and the deepest goes **112.8 → 15.8 m**, so the amphitheatre the walk stood
in is not there any more.

Relief moves −1.9 %, mean surface −2.0 % — reported because `corrections.md` #61 requires the
aggregate to ride *beside* the neighbour-relative measure, never instead of it. On its own it
would have licensed nothing, in either direction.

**The shipped world moves a little, and the reason is worth stating.** `diffusion = 0.12` sits
*inside* the 1/8 bound — so the config rate was never the problem — but `eff_diff` folds in the
lithology's creep susceptibility, and peat is the softest thing in the world. The shipped
peak is **0.261**, so the shipped world takes **two** sub-steps and its goldens moved: mean
regolith 4.57 → 3.91 m, relief 5521.9 → 5521.0 m, mean surface 515.2 → 515.0 m, closed hollows
0 → 0. Had the peak sat inside the bound this slice would have been byte-identical, and the
test that says so is written so the day it becomes true is not a silent one.

The unbounded operator is still reachable and still hashed (`DeepConfig::creep_substep`,
`GOLDEN_SURFACE_UNBOUNDED_CREEP`), which is the discipline that makes a moved golden an
*authorized* move rather than a lost fixed point.

## The thing the fix exposed, which is not mine to decide

Look at the calibrated · sub-cycled row again: **mean regolith 1.40 m**, against 43.72 m
before — and against 4.57 m on the *shipped* world. The calibrated world now has less soil than
the uncalibrated one, and 16,347 cells carry more than a metre of closed-hollow fill on a
surface whose concavity rms is 0.30 m. Those are not pits; they are broad shallow basins on
ground that has been scraped to bedrock.

The mechanism is not subtle once stated. **`EROSION_CALIBRATION = 45` was fitted against a
capped operator.** journal/0114 measured that raising `diffusion` bought almost nothing —
100× on transport alone bought 1.6× — and concluded the pass was a one-cell-per-epoch conveyor
(`stubs.md` #27). That was true *of the broken operator*. Remove the cap and the same
multiplier is a completely different amount of erosion: at `D = 12.6` per epoch the diffusion
length is ~5 cells, and over 200 epochs material travels far enough to leave.

> **A constant fitted against a broken operator does not survive fixing the operator.**

So this slice fixes the solve and **falsifies the number calibrated on top of it**. The
multiplier is an appearance-class, user-owned call — journal/0114 is emphatic about that and
so is `production_config` — and the ladder is measured and handed back rather than decided:

| multiplier | sub-steps | mean h | `conc(h)` rms | ACF(1) x | hollows >1 m | **>10 m** | deepest | gen |
|---|---|---|---|---|---|---|---|---|
| 1× (shipped) | 2 | 3.91 | 2.87 | −0.098 | **0** | **0** | 0.0 m | 37.2 s |
| 3× | 7 | 4.40 | 5.39 | +0.148 | 146 | **0** | 1.8 m | 46.0 s |
| 5× | 11 | 3.02 | 3.87 | −0.065 | 300 | **0** | 2.9 m | 53.2 s |
| 10× | 22 | 2.18 | 3.40 | −0.116 | 820 | **0** | 4.7 m | 75.7 s |
| 20× | 45 | 1.65 | 3.06 | −0.121 | 3,676 | **0** | 8.3 m | 121.6 s |
| 45× | 100 | 1.40 | 2.66 | +0.001 | 16,347 | **12** | 15.8 m | 233.1 s |

**The pit criterion is met, and it is met at every rung.** journal/0114's headline against the
old operator was *"deep closed depressions at ANY multiplier above 1× — 44 pits at 5×, deepest
45 m; 148 at 45×, deepest 112 m"*, and it concluded there is **no safe multiplier underneath**.
Under the fixed operator the deepest hollow anywhere on the world at **5× is 2.9 m**, at 10×
4.7 m, at 20× 8.3 m, and **nothing anywhere is deeper than 10 m until 45×**, where twelve cells
are. The autocorrelation stays inside ±0.15 the whole way — white noise is −0.167, so the grid
is quieter than noise at every rung. *There is a safe multiplier underneath. There are five of
them.*

**And the ladder says something the fix did not intend, which is why it is here rather than in a
conclusion.** Mean regolith runs 3.91 → 4.40 → 3.02 → 2.18 → 1.65 → 1.40 m: past 3× the cover
**thins monotonically** as the multiplier rises. journal/0114 measured the exact opposite —
4.6 → 8.8 → 43.9 → 118.8 → 359 → 782 m — and drew the conclusion the whole calibration rests on:
*"the landscape buys denudation by burying itself"*, because transport had a ceiling supply did
not. **Fixing the ceiling inverts the asymmetry.** Transport now outruns supply, so a uniform
`scale_erosion_rates` no longer holds the cover anywhere near fixed in *either* direction, and
the joint-calibration argument — which was right, and was the finding that made journal/0114
tractable — now has a different balance point. That is a derivation to redo against a published
band, not a number to pick, and it is user-owned.

`calibrated_rates` still ships **false**. What has changed is that it is no longer blocked on a
defect; it is blocked on a number, and the number now has a ladder under it.

## The temporal discriminator, for the record

The isolation above settles the mechanism on a fixture. The question it does not answer is
whether the world's checkerboard is *that* mode or merely looks like it, and there is a cheap
test with opposite predictions: solve the same world to `N` and to `N + 1` epochs and correlate
the two `conc(h)` fields. A frozen spatial pattern — a stencil the solve carves into every
cell — is the same pattern one epoch later and correlates near **+1**. A period-2 mode is its
own negation and correlates **−1**.

| arm | `corr(conc_h[N], conc_h[N+1])` | reading |
|---|---|---|
| shipped · unbounded | **+0.7572** | a persistent spatial pattern — a landscape |
| calibrated · unbounded | **−0.9009** | **its own negation one epoch later** |
| calibrated · sub-cycled | **+0.9518** | a persistent spatial pattern again |

44,258 / 43,371 / 43,799 interior land cells. **−0.90 is the period-2 mode, measured on the
world rather than on a fixture**, and the shipped control at +0.76 is what makes it a
discriminator rather than a restatement — the same instrument on a world with the same limiter
binding on 88.7 % of its cells reads the opposite sign. And the third row is the fix: the
calibrated world's grid-scale structure is now *more* persistent epoch-to-epoch than the shipped
world's, which is what a heavily eroded landscape should look like.

This is the third instrument in journal/0115's sequence and the first one that could see the
question directly. The concavity autocorrelation says *the field alternates in space*; the fill
depth says *and it bottoms out in closed holes*; only this one says **it alternates in time**,
which is the difference between a rough answer and a mode.

## One more instrument that had to be re-derived rather than tightened

The fill-depth guard's first bar was *zero cells past 1 m*, argued from the clamp's own
statement: a cell that may be cut down to but not below its lowest outlet holds no closed
hollow beyond what priority-flood tolerates without carving. It failed on the shipped fixture
with **3 interior cells between 1.00 and 1.72 m** — and the tempting move, at that point, is
to write `< 5` and move on.

The honest move is to notice that the bar has to separate two *measured* scales, and to say
which. The dimple floor is ~1 m and is structural: priority-flood fills rather than carves, and
four phases run after incision in the same epoch. The failure scale is ~45 m and up —
journal/0114 measured the clamp genuinely breaking at 45 m and 112 m. **10 m** is an order of
magnitude above the first and 4.5× below the second, so dimple accumulation cannot reach it and
a real clamp failure cannot hide under it. The shallow count is printed and not asserted,
because a shallow closed basin on a flat is a pan, and pinning its population would be pinning
a snapshot.

The ladder is what proves the bar is not vacuous: it holds at 1× / 3× / 5× / 10× / 20× and
**first fails at 45×**, with twelve cells.

## The instruments, which were half the brief

`corrections.md` **#62** is that `mfd_routing::no_interior_cell_is_cut_below_all_of_its_
neighbours` **saturates**: it counts cells below *all eight* neighbours, so as the defect
generalises the neighbours sink too and stop qualifying each other, and the count falls toward
zero exactly when damage becomes universal. It is kept — an isolated deep pit is still real and
this is still the cheapest way to see one — and it is now the first of three:

- **fill depth** (`filled − routed`, the router's own depression fill), which measures the
  hollow at a cell regardless of what its neighbours do and therefore **rises** as the defect
  generalises. Its bar is derived from the clamp's own statement rather than from a run: a cell
  obeying "you may be cut down to, but not below, your lowest outlet" carries no closed hollow
  past what priority-flood tolerates without carving, so the bound is *zero cells past 1 m* —
  the same number, on an instrument that can fail for the right reason.
- **concavity autocorrelation**, which is neighbour-relative and cannot saturate at all,
  because a correlation has no threshold to fall off. Bounded at −0.5, three times past white
  noise's −1/6 and half a checkerboard's −1 — a bound with a derivation, not one chosen until
  green.

And the four census primitives those tests need — `laplacian8`, `pearson`, `acf4`, `flip_rate`
— moved into `deeptime::census`. journal/0116 recorded that extraction as open and out of its
scope, for the honest reason that a Rust example is its own crate root and cannot call a
sibling's census. That left the alternative as *copying* it, which is anti-shape **A-1** on the
exact code path where journal/0115 found three instruments agreeing because they were the same
instrument copied three times. This slice needed the same four functions, so it took the
extraction.

## What this entry is really about

The chain that produced this fix is four entries long, and every link in it was written by
someone who could have shipped a plausible fix instead:

- 0114 measured the pits and did not brief the clamp.
- 0115 falsified its own instrument, on a user's one-sentence report, and refused to promote
  its explanation.
- 0116 ran the discriminators, killed two mechanisms including one it had been handed
  mid-flight, and wrote its own successor hypothesis as a hypothesis.
- 0122 tested that hypothesis before touching the operator, and found that the *falsification*
  in the middle of the chain was the thing that needed correcting.

Nothing here was found by being clever about the code. The mechanism was found by an
experiment that ran the pass on a grid with nothing else on it, and the correction was found by
dividing one number in the config by one number from a textbook. **Both are things you only do
if you have decided in advance that the corpus's confident claims are the ones worth checking.**

> *The most dangerous artefact in a corpus is not an unanswered question. It is a
> well-controlled experiment whose null was recorded as an answer.*

---

*Postscript, stamped 2026-08-04 (corrections #98, by that correction's author).* The
pits bar this entry derived — hollows > 10 m = 0 — was a **clamp-failure detector**,
valid between the ~1 m dimple floor and the ~45 m failure scale it measured, at M≈45,
on the operator as it then stood. It later travelled to M≈375–400 as an acceptance
*doctrine*, carrying a premise this entry never claimed and nobody ever decided ("a
deep closed depression is a defect"), and blocked the P2 flip for a week. The user
rejected the premise 2026-08-04; the replacement criterion is process-ownership
(corrections #98). The detector's derivation stands at its own scale; what expired
was the jurisdiction.
