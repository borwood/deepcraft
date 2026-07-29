# 0124 — The seeds were repair for the skip

*2026-07-29. The `Schedule` sum type, and an audit that came back unanimous when it was
written expecting a split decision.*

> blogworthy: **lens 3 (reflexions in a deepsim codebase)** and **lens 1 (AI-native
> development)**. A ratified type had three variants and the audit it shipped with retired
> every candidate for two of them — and that is the *good* outcome, not an embarrassment.
> Also the sharper story: the one artifact this slice moved had no golden, so a 834-test
> gate watched it move and said nothing.

## The rule nobody wrote down

`runner.rs` carried a line that had been true since the loop was first re-housed as declared
passes:

```rust
epoch.is_multiple_of(period) && (period == 1 || epoch > 0)
```

*A coarse-rate pass does not fire at epoch 0.* It was written for three passes — the climate
march, the geotherm, the head field — each of which was run once before the loop opened, in
`run_cells_with_cadence`'s pre-loop block. Firing them at epoch 0 as well would redo the seed,
so the runner skipped them.

RATE (journal/0123) made that rule visible without settling it, and flagged it. The moment a
**world** could author a cadence onto any pass through a `CadenceTable`, the same clause
silently acquired a second meaning: *and your re-rated erosion pass does not run in epoch 0
either.* Nobody decided that. It was a property of the runner masquerading as a property of a
declaration.

The ROADMAP's guess at the fix was *"a declared `seeded` flag beside the cadence."* The user's
ruling (`ARCHITECTURE.md` § *Schedule*, 2026-07-29) rejected it by elimination and reached a
sum type instead: a `bool` cannot express *run-once-only*, and a `rate: 0` sentinel cannot
express *seed-then-step* — and it would re-open the zero `Cadence` that `NonZeroU32` was
chosen to close.

```rust
pub enum Schedule {
    Seed,                    // pre-loop once, never in-loop. Integrates ZERO time.
    Step(Cadence),           // in-loop only.
    SeedAndStep(Cadence),    // both.
}
```

with two rulings attached: **a seed is an initial condition** — it establishes t=0 state and
integrates zero time — and **epoch 0 fires for everyone, the skip rule is deleted.**

## The audit, and the discriminator that actually decided it

The brief said to audit each of the three pre-loop incumbents and decide *from what the body
does*: honest initial condition (`Seed` / `SeedAndStep`) or first step in disguise (`Step`,
and its pre-loop run dies). I expected a split — `climate` looked like a first step, `head`
looked like a genuine initial condition, and `geotherm` looked ambiguous.

Reading the bodies settled two of them immediately and left the third genuinely contested.

**`dc:deep/climate` — `Step`.** The pre-loop line is `climate::march(&mut grid,
sea_level_at(cfg, 0))`. The pass body is `climate::march(&mut ctx.grid, ctx.sea_level)`. It is
*the same call*. `march` is a pure function of `(surface, sea level)` writing `grid.precip`,
and climate is first in the topo order, so the epoch-0 firing recomputes bit-for-bit what the
seed wrote. No argument needed.

**`dc:deep/geotherm` — `Step`.** Same shape, one indirection out: `geotherm::march` is a pure
recompute of `grid.geotherm` from the current crustal state, and nothing in the epoch reads
that plane at all — only coal rank does, post-loop. So the value that survives the run is the
last firing's, either way.

**`dc:deep/head` — this one was a real question.** Its pre-loop call is *not* its body. The
body relaxes the potential against the drainage solve's `filled` / `routed` / `area`; the seed
passes `&[]` for all three, because no drainage has run yet. The code even says so, and says
it well:

> *No routing has happened yet at epoch 0, so the seed reads the bare surface as its own
> free-water level and no stream anchors — the epoch-0 potential is therefore anchored by the
> sea and the border alone, which is the honest initial condition rather than a guess.*

That is a good defence of an initial condition. It is also, once the skip rule is gone,
answering a question nobody is asking any more.

**The discriminator that resolved it — and it generalises, which is why it is the part worth
carrying forward:**

> A seed is an initial condition only if something **observes** it before the pass itself
> first steps.

Under "epoch 0 fires for everyone", every period-`p` pass first steps *inside epoch 0*, at its
own position in the order. So a seed survives to be read only if some consumer sits earlier in
epoch 0's order, or runs pre-loop. Walk the three:

- `climate` is **first** in the order. Nothing pre-loop reads `grid.precip` — `Erosion::new`,
  `BioticSim::new` and `TectonicSchedule::new` do not touch it.
- `geotherm` has **no in-epoch reader at all**, by declaration.
- `head`'s only in-epoch reader is `dc:deep/flow_record`, and the graph pins `head` **before**
  it.

All three seeds were writes that nothing observed. They were not initial conditions. **They
were repair for the skip** — they existed to fill the twenty or forty epochs of hole the skip
opened at the start of the run — and when the skip died they died with it. All three pre-loop
blocks are deleted.

That is a unanimous verdict on a question I opened expecting three different answers, and the
reason to record the *method* rather than the outcome is that the method is what a future
pass-author will need. "Is this an initial condition?" is not answerable by reading the body
alone; it is answerable by asking who reads it, and when.

## What the skip rule actually cost

The ratification names the invariant it buys: *every pass's integrated `dt` over a run equals
the world's elapsed time.* Assertable only since RATE made `dt` a real number.

Building the test made the claim sharper. A firing at epoch `e` opens a phase covering
`[e, e + period)` — that is what `Cadence::phase_total` *is* — and firings land on the
multiples of `period` **from zero**. So the phases **tile** the epoch axis, and the run's
window `[0, iterations)` is covered exactly once, clipped only where the world ends inside the
final phase. `Schedule::integrated_dt(N) == N`, for every period, every sub-turn count, every
run length. Σ`dt` = N exactly is a corollary in the divisible case, which the shipped roster is
(20 and 40 into 200).

Read that against the deleted clause and the cost is not a rounding error at a boundary. The
skip did not *shift* the tiling — it **deleted the first tile**. A period-20 pass in a
200-epoch world integrated 180 epochs of world time while its period-1 neighbours integrated
200. The passes were on different clocks, which is the one thing a scheduler exists to prevent.
`schedule.rs::the_deleted_skip_rule_lost_a_whole_period_of_world_time` keeps that arithmetic as
a test rather than a paragraph.

`tests/schedule_axis.rs` asserts the invariant over three real rosters at four run lengths, and
builds **no world at all** — when a pass runs is a property of the roster and the config, not
of the terrain. Gate cost: milliseconds.

## One artifact moved, and the interesting part is that nothing could see it

The scratch-pad rule applies (CLAUDE.md § Conventions): a hash move produced by ratified
semantics is re-captured with the *why*. So the question was which hashes moved.

`GOLDEN_SURFACE` and `GOLDEN_RECORD` held — `providers_golden.rs` and `rate_axis.rs` both
green. So did `grid.precip`, `grid.geotherm` and the final `grid.head`, bit for bit, exactly as
the audit predicted (`climate` and `geotherm` are idempotent recomputes; `head`'s exported
plane is re-relaxed post-loop anyway).

The **flow record** moved. On the golden fixture, 315,320 entries → **314,070**, and the whole
of the −1,250 sat in the **vertical** face family (24,668 → 23,418). Lateral 98,747,
boundary-ocean 190,399, boundary-base 1,506, divergent 26,209, convergent 26,091 — all
bit-identical. That localisation *is* the evidence for the mechanism: vertical faces are the
only family fed from `grid.head_exchange`.

Total magnitude went **up** while entry count went down (7,396,442.96 → 7,397,344.27), which is
the shape you would predict from the audit. The bare-surface seed had no lakes and no perennial
streams to anchor the potential, so it spread a weak exchange across cells the real solve does
not recharge at all. Chapter 0's vertical faces used to be integrated, for twenty epochs, from
a head field solved on a landscape with **no drainage**. Now they come from the real epoch-0
solve. This is a move in the direction of correct.

**And here is the thing that mattered more than the move.** I only know any of the above
because I wrote a throwaway test harness, ran it, stashed the slice, ran it again on `main`,
and diffed by hand. The flow record — *the largest thing the ritual keeps* — **had no golden**.
It is a pure sidecar to both existing goldens by construction, which is exactly why the terrain
and strata fingerprints held; and that same property means neither of them can ever see it
move. A full 834-test workspace gate watched a shipped artifact change and said nothing.

`GOLDEN_FLUX` now exists (`tests/flux_record.rs`), with the move and its mechanism recorded on
the constant in the house style. The generalisation is worth stating plainly, because it is not
specific to flux:

> An artifact the ritual ships with no tripwire on it cannot have an *authorized* move,
> because nobody can see it move.

The project's whole golden discipline is built on the distinction between an explained move and
an unexplained one. That distinction is unavailable for anything ungoverned. We had three
shipped artifacts and two goldens, and nobody had noticed.

## Two variants with no declarer, on purpose

The audit retired every candidate for `Seed` and `SeedAndStep`, so both variants ship with **no
production declarer**. That is a `spines.md` § 3 row and it is written there in state 3 — *held
as a candidate* — not state 1.

The temptation to resist is reading a zero-declarer count as a gap to fill. It is the audit's
**finding**. The axis is a sum type because the *ratification* eliminated the alternatives, not
because three passes were waiting to use all three variants. And the honest reading of "nothing
seeds today" is not "seeding was unnecessary" but "the deep-time roster's genuine setup work is
not pass-shaped yet" — `BioticSim::new` planting the biotic layer's identity planes,
`TectonicSchedule::new` precomputing the chapter table. Those construct *owned state the ctx
carries*, not grid planes a pass will overwrite; converting them is declared epochs' job
(ROADMAP slot (d)), named as the heir on `Schedule::Seed` itself.

They are not decorative. `DeepSchedule::run` opens with a setup pass over `plan(None)`, and two
runner tests drive a synthetic three-pass roster through it — one pure seed, one seed-and-step,
one ordinary per-epoch pass — and read back what the scheduler decided.

## A small mechanism worth keeping: `dt = 0.0` is enforcement, not convention

A seeding body is handed `dt = 0.0`.

That is not documentation of "a seed integrates no time" — it is the thing itself. Every
rate-shaped transformation in a seed's body multiplies by zero. A pass whose "seed" is really
its first step cannot masquerade as one, because running its step body at `dt = 0` produces
nothing. The discrimination the audit above had to make by hand, three times, is a property the
runner now enforces for the fourth pass and every one after it.

Same family as `NonZeroU32` on `Cadence` and `CoarseField` making the raw per-cell read
unsayable: *make the unsafe call impossible to write, rather than rejecting it at run time.*

## One refactor fell out, and it was the right one

`DeepSchedule::run` used to inline its firing decision. Testing the seed phase without building
a world meant separating the decision from the execution, so the runner now drives itself
through `DeepSchedule::plan(Option<u32>)` — `None` for the setup epoch, `Some(e)` for epoch
`e` — returning `(pass id, dt, turns)` in topo order.

The reason to route `run` through it rather than add a parallel query is § A-1: a summary that
is not *derived from* the authority becomes a second authority. A test that reads `plan` is
reading the decision the world is generated from. It costs one small `Vec` per epoch, in gen
time, which is not a constraint here by doctrine.

`DeepPass::fires` went with it — a one-line delegation to `Schedule::fires` with no caller left
once `run` used `plan`. Deleted rather than `allow(dead_code)`d.

## What this leaves

- `Schedule` is built; the arc is not closed. Authored ORDER, the open vocabulary and declared
  epochs are still owed (ROADMAP slots (b), (c), (d), (e)).
- `Seed` / `SeedAndStep` have no declarer and want none until (d).
- `GOLDEN_FLUX` closes one tripwire hole. **The corpus was not swept for others** — the
  question *"which shipped artifacts have no golden?"* was asked once, about one artifact,
  because that artifact happened to move today.
