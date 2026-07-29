# 0123 — The clock the engine never had

*2026-07-29. The RATE axis, built. `docs/dependency-graph.md` E3.*

---

## The thing that was already there, and wasn't

If you had opened `runner.rs` yesterday and grepped for `dt`, you would have found
it. `DeepStepCtx::dt`, set once per firing, handed to every pass. You would have
found a `period` on each `DeepPass`, and a scheduler that respected it. You would
have concluded, reasonably, that the cadence axis existed.

The field's own doc comment said otherwise, in the flattest possible language:

> *The phase length handed to the firing pass (`= period`). Pinned/inert this
> movement — carried as the rate axis's time-base, **scaled by nothing yet**.*

That sentence had been true for a month. `dt` was a variable that got assigned and
never read — except by `weather_inventory`, one pass, whose period was 1, so it
multiplied by exactly 1.0 forever. And `period` was a number written in Rust source
beside the pass body, which is a constant, not data. The axis was **shaped**, and
it was **empty**.

This is the third time this project has found that shape: machinery built to the
point where it looks finished from the call site and does nothing at the far end.
`reads_prev` was documentation until journal/0104 handed it to the graph kernel.
`sample_dithered` was built, named for a user law, and uncalled for seven days.
The tell is the same each time — you can trace the call chain and it *arrives*, so
nothing in a code reading catches it. What catches it is asking what would change
if you deleted the thing. Delete `dt` yesterday and nothing moved.

## What RATE nearly became, and the ruling that stopped it

The slice was sequenced because of an erosion defect. journal/0122 found the
hillslope creep operator running 43× past its own monotonicity bound and fixed it
by sub-cycling *inside the pass*: `n = ceil(max_cell eff_diff / (1/8))`, derived
from the von Neumann analysis. It worked. And it left behind a stub entry
(`stubs.md` § 30) whose title says the whole problem — *a pass that hand-rolled its
own timestep*. The pass had grown a clock because the engine did not have one.

The obvious reading is that RATE should own the sub-cycle: it is a timestep, RATE is
the time axis, put them together. That reading survived a few hours on 2026-07-29
before the user rejected it:

> *"I really hate to make RATE more complex now. Couldn't substepping be solved
> within the field instead, where it takes `dt` from outside and calcs its own
> internal multiplier in addition to that to stay within bounds?"*

This is the ruling that made the slice small, and it is worth stating why it is
right rather than merely accepted. **The two divisions look identical and have
different owners.** `dt` is *authored*: how much world time this turn covers, a
modelling choice a world makes. The sub-step count is *derived*: how finely this
particular operator must integrate that time to stay monotone — and only the
operator can know it, because the threshold is a function of its **stencil** (a
4-neighbour Laplacian and an 8-neighbour one have different constants), of `dx` and
`dt`, and of the coefficient field. Four inputs, and only one of them belongs to
the scheduler.

Put the sub-cycle in RATE and every plugin author needs a von Neumann analysis
before writing a diffusion pass. Put it in the kernel and **the unsafe call becomes
inexpressible** — the same move `CoarseField` made when it stopped the raw per-cell
read from being sayable. So `stubs.md` § 30's heir is the S-10 field-solver
primitive, not this slice; RATE supplies the `dt` the kernel divides, and stops.

In the code the two divisions now sit four lines apart in `Erosion::diffuse`, in the
order they have to be in:

```rust
let rate = cfg.diffusion * dt;     // authored: how much time this turn covers
...
let diff_sub = rate / f64::from(n_sub);   // derived: how finely to integrate it
```

That is the whole of what "the field takes `dt` from outside and calcs its own
internal multiplier" means, and it took one line to say once the ownership question
was settled. The half-day was the ownership question.

## What got built

Three things, in `crates/dc-worldgen/src/deeptime/cadence.rs` and the runner.

**A cadence is two numbers and one derived quantity.** `period` — epochs between
firings, the coarse direction. `sub_turns` — turns taken per firing, the fine
direction, and this is the half that did not exist at all. Then
`dt = period / sub_turns`, epochs of world time per turn, which is the number the
runner hands the pass. The user's 2026-07-23 sketch had both directions in it from
the start (*"tectonics ×1 … weathering ×3 each"*) and only the coarse one had ever
been implementable.

Both numbers are `NonZeroU32`. A zero cadence is an infinite loop or a division by
zero depending on which one you got wrong, and this codebase's stated preference —
from the same argument as the paragraph above — is to make the bad call
**unsayable** rather than to validate it at run time. There is no
`CadenceError`; there is no error path; `Cadence::new` returns `Option` and the
convenience constructors read a zero as a one, exactly as the runner's existing
`remarch_interval.max(1)` did.

**The runner takes the turns.** Two lines:

```rust
ctx.dt = p.cadence.dt();
for _ in 0..p.cadence.sub_turns() {
    (p.body)(ctx);
}
```

Each turn sees the cell state the previous one left, which is the sketch's *"a phase
is handed the cell state at its start (as other passes' phases left it)"*. With one
sub-turn — every pass in the shipped roster — this is the pre-RATE loop, character
for character in effect.

**The numbers become data.** A pass declares a default cadence; a `CadenceTable`
authored by the world overrides it by pass id; `deep_passes_with` applies the table
over the declaration and the runner executes what it is handed. *The engine derives
nothing.* An empty table is every pass keeping what it declared, which is the
shipped schedule.

The keys are owned `String`s and not `&'static str`, deliberately, and this is the
one place the slice looks over-built for what it does today. There is no manifest
format yet — order-as-data brings that, and it is the next slice in this arc. What
exists here is the *seam* a loader will fill, shaped so a loader can fill it, and
nothing more; inventing the format now would be building the general mechanism ahead
of its caller, which `session-workflow` § Seam-first has an explicit rule against and
which this project has paid for before.

## Making `dt` real without moving a world

Threading a clock into a simulation that has never had one is the part where you
break everything. The saving property is arithmetic: **`x * 1.0 == x` exactly for
f64**, and every pass in the shipped roster has `dt == 1.0` or ignores `dt`. So a
transformation can be converted from *"a magnitude per epoch"* to *"a rate × the
phase length"* one pass at a time, and each conversion is provably a no-op on every
world already created.

Converted this slice: hillslope creep (`cfg.diffusion * dt`), the uplift plane and
its ledger total (`u * dt`, and the returned sum scaled with it — a ledger accounting
for a different amount of time than the grid got is a mass-conservation failure
waiting to be blamed on something else), crustal thickening (`f * dt`). Already
live: inventory weathering.

Not converted, and the reason matters more than the list: **a pass that ignores
`dt` is making a claim, and for three passes the claim is true.** The climate march,
the geotherm and the head field are *relaxations toward an equilibrium set by the
current state* — they solve for where the field would sit given today's topography,
not for how far it moved in an interval. Their coarse period says *when to resample*,
and there is nothing for a duration to scale. That is a real distinction between two
kinds of pass and the RATE axis makes you notice it, which was not true before,
because before there was nothing to ignore.

Left owed: stream transport, bedrock weathering, the wind and wave agents. Each is
rate-shaped and each needs a modelling decision rather than a mechanical one —
weathering is an exponential approach, so its honest `dt` form is `1 - exp(-k·dt)`
and not `k·dt`, and picking between them at the wrong moment silently re-tunes a
constant that `EROSION_CALIBRATION` is about to be re-picked against anyway
(dependency-graph P2). Those conversions belong with that re-pick, and they are
recorded rather than done.

## The test that was a hash comparison

The best thing about this slice is that it did not require judgement. journal/0122
had just moved the goldens and left the world at a **known-good fixed point**, so
the acceptance criterion could be stated before any code was written: *build RATE,
express today's schedule as the default cadence data, and the world must be
byte-identical.*

It is:

```
RATE, empty table: surface = 0x15A6B7567A8429FB  record = 0x820BA19849DD234A
```

against `GOLDEN_SURFACE = 0x15A6_B756_7A84_29FB` and
`GOLDEN_RECORD = 0x820B_A198_49DD_234A`. Both, and through the *same* distillation
path the goldens were captured through — `build_field_cfg_cadence` with an empty
table is `build_field_cfg`, so the comparison needs no equivalence argument of its
own. Both hashes rather than one because they see different things: the surface
catches a mis-scaled uplift or creep, the record catches an epoch that ran a
different number of times, and a `dt` slice could plausibly move either alone.

**And then the mirror half, which is the one that is easy to skip.** A cadence table
that changed no bit anywhere would satisfy the acceptance test *perfectly* and be a
decoration. This is the failure mode this project keeps re-finding — machinery that
arrives at the call site and does nothing — so the same file asserts that an
authored cadence *moves the world*: creep sub-turned ×2 and the eolian agent at
period 3 both produce a different surface, both still close
`Δ(ΣR + ΣH) = uplift + biotic`, and both leave the forcing pass's integrated time
**bit-identical**, because a clock leaking from one pass to another is exactly the
bug this axis could plausibly introduce.

Sub-turning creep changes the world because the operator is **non-linear** — its
flux limiter binds on the cells with the most cover — and it is worth being precise
that this is the *justification* for the knob rather than a caveat about the test.
If creep were linear, sub-turning would be a no-op and there would be nothing to
assert. The reason a per-pass temporal-resolution knob is worth having at all is
that the processes it governs are not linear, which is `material-behavior.md` § 5's
own argument for separate declared passes over a fused monolith.

One assertion in that test failed on the first run, instructively. It asserted
`uplift_total` was unchanged when creep was re-rated. It is not — on the tectonic
path `uplift_total` is the **isostatic injection** ΣΔR, a *response* to how much
bedrock erosion removed, so it moves when creep does. The ledger is named for uplift
and holds a feedback. `thickening_total` — the analytic forcing plane summed per
firing, with no erosion term in it — is the quantity that actually says *"the forcing
pass integrated the same amount of time"*, and it is what the test asserts now. A
small thing, but it is the sort of misreading a green test would have hidden forever
if the assertion had happened to be about the right variable for the wrong reason.

## Where this leaves the arc

RATE was the first of three (`ARCHITECTURE.md` § *The engine is plugin-agnostic, and
pass ORDER is authored*, DECIDED 2026-07-26). **The arc does not close here.** Still
owed: authored ORDER and its validator, with the revision-token chain retired; the
open resource vocabulary, with `DeepAxis` deleted; epochs declared with pass members
and a chapter count *or* a terminating condition; and the vocabulary-split question.

The convergence worth naming: RATE is engine-side and the cadence *numbers* are
content-side, and the slice found that boundary without being told where it was. The
runner executes a declared cadence at a real clock — that is a primitive. Which
cadence tectonics runs at is the default pack's opinion about tectonics, and a third
party's pack will have a different one. *Every pass is content, including tectonics
and erosion*; the scheduler is not.

The seam the next slice needs is now visible from here, too. A `CadenceTable` and an
authored pass order are the same kind of object — per-world data the engine executes
and does not derive — and they will arrive in the same manifest. Building the
cadence half first, against a fixed point, was the cheap way to find out what that
object has to look like.

> **blogworthy:** *the axis that was shaped and empty* — a scheduler with a `dt` it
> assigned and never read, a `period` that was a constant wearing data's clothes, and
> the third instance in this codebase of machinery that arrives at the call site and
> does nothing. Plus the ownership ruling that kept it small: two divisions four lines
> apart, one authored and one derived, and why housing the derived one anywhere but
> the kernel makes a von Neumann analysis a prerequisite for writing a plugin.
> *Lenses: 3 (reflexions in a deepsim codebase — who owns which timestep), 1
> (AI-native development — a user rejecting a defensible widening, and the slice
> getting better for it).*
