# 0013 (draft) — the parallelism that wasn't: measuring the deep-time flip

*Draft for integration. S9b, the follow-up S9 asked for by name.*

S9 ended with one honest loophole. We'd measured B — the full-resolution global
deep-time tier, 27 M cells at 48 m — at ~63 minutes and ~3 GiB, and recommended
against it in favour of A+C. But the recommendation was explicitly *conditional*:
the engine was a single-threaded scalar priority-flood, and we'd written down
that a parallel flood "could plausibly be 10–50× faster; at 30× the full run
drops from ~1 hour to ~2 minutes," at which point B's decisive advantage — it is
by far the *simplest* code, no refinement builder, no halo sizing, no coarse→fine
drainage handoff — might outweigh its cost. The user's skepticism of C was on the
record. So we owed a real number, not a hand-wave. This is that number.

## Profile first, so you parallelize what actually costs

The temptation was to reach straight for rayon and sprinkle `par_iter` over the
whole step. We profiled instead, and the profile is the whole story. The
deep-time step is nine phases; timed individually at 6.8 M cells they split into
two populations. Six phases — routing, weathering, hillslope diffusion, and the
bookkeeping ones — are per-cell independent and together about 20% of the step.
The other ~80% is three phases with a hard data dependency: the priority-flood
depression fill (a global elevation-ordered heap), drainage-area accumulation,
and stream-power transport (both are flux chains — a cell can't run until all its
upstream has). And **the flood alone is 65–72%.** One phase. A serial min-heap.

That reframes the task completely. Parallelizing the easy 20% is a rounding
error unless you also crack the flood.

## The determinism tax is the whole plot

We did parallelize the easy 20%, and did it well — routing went 9.3×, diffusion
and weathering ~5.7×. The trick worth remembering: to keep the parallel result
**byte-identical** to scalar (non-negotiable — worldgen determinism is
load-bearing everywhere downstream), every phase became a *pure per-cell kernel*
driven by either a sequential loop or a `par_iter`, so the per-cell arithmetic
and its summation order never change — only who runs which cell. Hillslope
diffusion needed reformulating from a neighbour-**scatter** (`h[i]-=f; h[j]+=f`,
cross-cell writes, unsafe to parallelize) into a **gather** where each cell sums
its own four edge-fluxes; it conserves mass identically and now agrees to the
bit. A new test asserts it: same seed, scalar vs. parallel, identical planes and
identical strata records. Green.

But byte-identity is exactly what kills the flip. The whole-step speedup came out
to **1.18–1.26×** — because the flood, the 70%, has *no* byte-identical parallel
form. And it gets worse with scale, not better: at the full 27 M cells the
flood's heap and arrays (~700 MiB) overflow every cache, each heap pop becomes a
DRAM miss, cost goes super-linear, and the flood swells to **98.5%** of the step.
The full-B parallel run clocked **70 minutes** — statistically the same as the
63-minute scalar baseline. We parallelized everything we were allowed to, and B
did not move.

## The flood tiles beautifully — and lies at the seams

So we measured the only lever that matters: tile the flood into row strips, fill
each in parallel. It scales *gorgeously* — up to **~11×** on 12 strips, blowing
past the ~1.16× ceiling the streaming phases hit. The reason is a nice one: the
streaming phases are memory-*bandwidth* bound (6 cores, one memory controller,
instant plateau), but the flood is memory-*latency* bound, and a strip's working
set fits in cache, so tiling is a locality win stacked on the core-count win.

The catch is the seams. Our tiled flood treats strip boundaries as open outlets —
no reconciliation — so any depression straddling a seam mis-fills, by up to
**110 m**, at ~0.4% of cells. That's the optimistic ceiling with the correctness
switched off. A *correct* parallel flood (Barnes' spill-graph join) has to pay
that back with reconciliation sweeps, and is still not byte-identical to the
serial fill. So the flood can't ride the deterministic path at all — a parallel
flood is a different algorithm you'd have to re-verify, not a drop-in.

## The arithmetic of the flip

Grant the best case anyway — a *free* flood — and the profile says transport and
accumulate are *also* serial flux chains, leaving a ~6–7 minute floor at B on
this machine. Realistic parallel flood plus serial transport: ~15–20 minutes.
The flip target was ~2 minutes — about **35× off scalar**, and six physical cores
top out near ~6–12× no matter how much engine work you throw at it. The flip
doesn't happen on consumer hardware. It would need a ~32-core box *and* a correct
parallel flood *and* a parallel transport — three speculative, determinism-
breaking rewrites — versus C, which S9 already proved sound. That's the wrong
trade.

**Verdict: A always-on + C refinement, unchanged. The parallelism escape hatch
S9 left open is now closed by measurement.** B stays fine for Small worlds, and a
known many-core server could bring Medium B into a ~5–8 min coffee break if you
weight code-simplicity very heavily — but for player machines and Medium/Large
worlds, A+C wins, and now we can say so with a table instead of a maybe.

> blogworthy: "The determinism tax on parallelism." The intuitive move —
> parallelize the hot loop — was foreclosed not by difficulty but by a
> correctness contract: the one phase that dominates (a priority-flood min-heap)
> has no byte-identical parallel form, while the phases that *do* parallelize
> byte-identically are a bandwidth-bound minority. The counter-intuitive punch:
> the tiled flood scales to 11× (latency-bound, cache-local) but only by lying at
> the seams — fast *because* it's wrong. A clean parable for when "just add
> threads" meets "results must be reproducible to the bit."

*Falsified: S9's "a parallel priority-flood could pull B to ~2 min" (→
corrections). Also: the recorder empty-header cost is 833 MiB at B, not the
648 MiB S9 estimated (full 32-byte struct, not just the Vec header).*
