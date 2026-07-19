# 0012 — deep time in a box: two-plane erosion, and where the halo holds

*2026-07-19 · S9 spike (background agent; integrated by the main session,
merge `3fd00bb`, 37 suites green on merged main after one real fight —
below). Numbers and full tables in docs/spikes/S9-results.md.*

## Integration note: the test that remembered a deleted worktree

The merge's first full-workspace gate run went red — and the failure was
nowhere near the spike. The dc-host parity suite spawns a nested cargo to
build the demo plugin wasm, locating the workspace via **compile-time**
`env!("CARGO_MANIFEST_DIR")` — a path baked into the cached test binary.
Agent worktrees share one `CARGO_TARGET_DIR`, and the workspace-feature
flavor of that binary had last been compiled *inside a worktree that no
longer exists*: `current_dir` → OS error 267, NotADirectory. Solo reruns
passed (different feature flavor, compiled from main), which made it look
flaky; it was deterministic per cached flavor. Fix: resolve the manifest
dir at **runtime** (the test runner sets it in the environment), with the
compile-time value as fallback. Lesson for the shared-cache-plus-worktrees
practice: **compile-time absolute paths and shared build caches are
enemies**; any `env!`-baked path in test plumbing is a stale-worktree bomb
(corrections #7).

We had a pregen world — 17×17 cells at 14.7 km, tectonics and climate and
drainage and a thin settlement history, all committed once at world-create
(S7). And we had the collapse pyramid below it, refining relief with addressed
midpoint jitter down to the voxel. Between them sat a hole: **deep time**. The
strata a player cuts through should be the *record of a landscape that ran* —
mountains rising and shedding sediment, seas coming and going, rivers carving
and filling — not a noise function dressed as rock. Orogeny had shown us the
shape of the answer (a stateful erosion sim baked to a record the sampler
reads). The open question was how big that sim needs to be, and the user had a
specific doubt to honor: **is a coarse tier plus lazy regional refinement really
the golden path, in an engine we 100% own — or does one honest brute-force pass
beat the whole refinement architecture on simplicity?**

So the spike had a real adversary. Not "prove C good" but "make B win if it
can."

## Building the sim

The engine is two planes — bedrock `R`, alluvium `H` — and one loop: uplift the
bedrock from the pregen provenance field, priority-flood the depressions so
every cell drains to the sea, route D8, accumulate drainage area, then move
sediment by stream power `A^m·S^n` with the alluvial cover shielding the bedrock
underneath. Weather bedrock into regolith; creep the regolith downhill. The one
discipline I held to without compromise was **explicit mass conservation**: every
metre that leaves a cell becomes suspended flux, routed down the receiver chain;
every metre the flow can't carry is deposited downstream; whatever reaches the
sea settles there. The only thing that enters the system is uplift. That gives a
falsifier you can actually assert — over N iterations, `Δ(ΣR+ΣH)` must equal
`N·Σuplift` to floating-point slack — and the test holds to <1 metre-cell out of
a ~10⁶ ledger. When a landscape sim conserves mass exactly, the strata it lays
down are *real* sediment that came from *somewhere upstream*, and the record
means something.

The recorder is the S7 `StrataRec` shape bent to deep time. The lesson I took
most seriously from orogeny's corrections was #9/#11: **tag by what you measured,
never by what you interpret.** So a unit carries subaerial-or-subsea (against the
*current* sea level, not the final one), arid-or-humid (from the precip the
orographic march produced *that* iteration), and an energy band (from the stream
capacity that actually moved it). The trick that made it cheap and exact at once:
record the **net** thickness change per cell per iteration. Deposition merges
into the top unit if the tag matches; erosion pops history off the top; strip the
column to bedrock and the next deposit is flagged an **unconformity** — a time
gap as a first-class object. Because the record only ever mirrors net ΔH, the
`sum(units)==H` invariant is exact by construction, not by hope.

## The thing the recorder wouldn't show me until I fixed the sea

My first runs read flat. Mean 1.1 units per column, most columns a single thin
smear. The recorder was working — it just had nothing to record, because **the
sea never moved.** With a static shoreline, every coastal column is permanently
subaerial or permanently subsea; the one axis designed to alternate never
alternated. Adding a deterministic sea-level sinusoid — transgression and
regression over the run — was the difference between a dead field and a legible
one: unconformity-bearing columns went from 0.1 % to ~2.8 %, and coastal columns
started recording the classic couplet — marine mud drowned under prograding
alluvium as the shoreline fell. The doctrine (earth-processes § 6) had told me
sea-level cycles stack alternating facies; I didn't believe how *much* of the
read-quality lived there until the tags lit up. **The record is only as alive as
the processes you let vary through time.**

The stories the columns tell now are ones you can interrogate. A fan on an arid
range flank: alternating medium/low energy bands, bone dry all the way up
because the high interior sits in its own rain shadow (the march earning its
keep) — pulsed debris flows, and you can *read* the pulses. A coastal column
with marine mud at the base and a thick arid cap above a flooding surface — sea
came, sea went. A sediment-starved margin with seven paper-thin marine/subaerial
couplets — a condensed section you can count transgressions in. Every one
answers "how did this get here?" in process terms, which is the whole bar.

## Where the halo holds, and where it doesn't

Then the real question: does regional refinement even work? I built a small fine
window I could afford to run *twice* — a reference, and a copy with its outer
ring perturbed the way a refined region's inherited boundary would be — and
measured how far the boundary error penetrates inward.

Two regimes, same window, same bump. With the rivers switched off (just
weathering and hillslope creep — a pure relaxer), the error decays cleanly and
monotonically: bulk envelope and deepest penetration are the same ~21 cells. The
halo theorem holds exactly — a relaxation process forgets its boundary within a
bounded radius. **Switch the rivers on** and the bulk field still relaxes over
~15 cells — but isolated spikes reappear *deep* inside, 2.9 m of error at 14
cells, another at 28, in an interior that's otherwise quiet. Those spikes are
**drainage reroutes**: the boundary perturbation captured or beheaded a channel,
and the signal teleported tens of cells inland along the receiver chain. A basin
has no halo.

That is orogeny's bounded/unbounded theorem, but *measured in our engine*, and
it settles the architecture question in a way I didn't expect going in. The
answer to the user's skepticism isn't "C is clever." It's: **drainage is the one
thing you can't refine, and it's the one thing that's cheap at coarse
resolution.** So you decide drainage once, globally, at the coarse tier — and
refine *only* the relaxation processes (hillslope, weathering, deposition,
facies, folding) locally, with a halo the measurement sizes at ~16–24 cells. C
isn't a holy grail; it's the *only* split the theorem permits once you've decided
you want landform detail without paying for it everywhere.

## And B?

I gave B its fair shot: the full 27-million-cell, 48-metre global run. It is the
**simplest code of the three** — literally the same `run()` as the coarse tier at
a finer number, no refinement builder, no halo, no coarse→fine handoff. If it had
come in at "minutes, paid once," it might have won on simplicity alone, and I'd
have written that.

It came in at **about an hour and ~3 gigabytes** on this machine — and ~99 % of
those 27 million cells are ocean floor and featureless craton no player will
walk. "Minutes" was scalar-engine-optimistic by ~30×. So B loses — but I want the
future reader to see the *conditional*, because it's the honest part: B loses
**because the engine is single-threaded**. A parallel priority-flood could
plausibly pull that hour down to ~2 minutes, and at 2 minutes B's simplicity
starts to look like the right trade. The one measurement that would flip the
whole recommendation isn't about geology at all — it's whether you're willing to
parallelize the flood. The real 3e work should spike that *before* committing to
the A+C machinery. And for Small worlds, B is already fine today.

The recommendation, then, with its own kill switch attached: **A always-on
(seconds, tens of MiB, and the record already reads true at 460 m), C's bounded
refinement layered on for approached regions, B only if the engine goes
parallel.** The user's doubt was the right doubt. It just resolves to "measure
the engine, not the architecture."

> blogworthy: "The halo holds for hills, not for rivers." A landscape-evolution
> sim where you can *watch* a boundary error decay in one regime and teleport
> across the domain in the other — the cleanest possible picture of why drainage
> is the thing you compute globally and everything else you can fake locally.
> Pairs with the mass-conservation falsifier (strata that are real sediment from
> real upstream) and the sea-level-brings-the-record-alive moment.

## Falsified / refined

- "B is minutes, paid once" → ~1 hour + ~3 GiB at Medium on a scalar engine
  (minutes only at Small extent, or parallelized).
- "Erosion is bounded, so a small halo suffices" → *hillslope* erosion is
  bounded (~16–24-cell halo, measured); *fluvial* erosion inherits drainage's
  global reach. C works only because drainage is decided coarse.

(Corrections.md entries are the integrating session's call, not the spike's — the
candidates are listed in S9-results.md § Falsified assumptions.)
