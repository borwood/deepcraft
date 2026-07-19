# S9b results — does parallelism flip the deep-time verdict?

Status: spike complete, 2026-07-19. A sharply-scoped follow-up to S9
(`docs/spikes/S9-results.md`), stress-testing that spike's one open flip
condition:

> **Recommendation: A always-on + C refinement; do NOT build B — UNLESS the
> erosion engine parallelizes** (a parallel priority-flood could pull B to
> ~2 min, where its simplicity wins; that mini-spike decides).

This is that mini-spike. Code additive-only in
`crates/dc-worldgen/src/deeptime/erosion.rs` (rayon per-cell phases + a measured
tiled-flood probe), `mod.rs` (`run_with`), the measurement harness
`examples/deeptime_par.rs`, and one new invariant in `tests/deeptime.rs`. Numbers
from the same Windows dev box, release profile, seed `0x0D5E_ED57_2026`, Medium
world (17×17 pregen). **Machine: 6 physical cores / 12 logical threads.** That
core count is load-bearing for the verdict — see below.

**Headline: parallelism does NOT flip the verdict. A+C stands, decisively.**

## What was parallelized (and what stayed scalar, and why)

The deep-time step is nine phases. They split cleanly by data dependency:

- **Embarrassingly parallel (per-cell independent)** — uplift apply, surface
  snapshot, D8 routing, bedrock weathering, hillslope diffusion, strata
  recorder. Each is now a *pure per-cell kernel* driven by either a sequential
  loop or a rayon `par_iter`. Because the per-cell arithmetic and its summation
  order are identical in both drivers and every write is to a disjoint cell, the
  parallel output is **byte-identical** to the scalar path *by construction* —
  proven by the new `parallel_equals_scalar_byte_identical` test (same planes AND
  same strata records, same seed).
  - Hillslope diffusion needed a reformulation to parallelize safely: the
    original **scatter** (`h[i] -= f; h[j] += f`) has cross-cell writes. It is
    now an equivalent **gather** — each cell sums its own four in/out edge fluxes
    from a frozen surface and a frozen limiter. Each edge's flux is referenced
    identically from both endpoints, so mass is conserved exactly; scalar and
    parallel both use the gather and agree to the bit. (The gather differs from
    the pre-S9b scatter only in fp round-off, well inside the mass-conservation
    slack — the mass test still passes.)

- **Serial by nature (kept scalar)** — priority-flood depression fill (a global
  elevation-ordered min-heap frontier), drainage-area accumulation, and
  stream-power transport (both are downstream-ordered flux chains: a cell needs
  its full upstream contribution before it runs). Parallelizing these means a
  level-ordered gather whose summation order differs from the serial scan, which
  **breaks byte-identity**. Per the determinism mandate, they stay scalar. They
  are the measured serial floor.

## Phase profile (record OFF — the B datapoint)

ms/iteration, scalar vs. the byte-identical parallel path, all 12 threads:

### 1.7 M cells (192 m), the B/16 scale

| phase | scalar ms/it | parallel ms/it | speedup | % of scalar step |
|---|---:|---:|---:|---:|
| uplift | 1.28 | 1.45 | 0.88× | 0.3% |
| surface | 1.78 | 1.68 | 1.06× | 0.4% |
| **flood** | **327.5** | **323.8** | **1.01×** | **65.1%** |
| route | 51.4 | 5.54 | **9.29×** | 10.2% |
| accumulate | 8.90 | 7.40 | 1.20× | 1.8% |
| transport | 45.3 | 46.0 | 0.98× | 9.0% |
| weather | 8.85 | 1.81 | **4.88×** | 1.8% |
| diffuse | 58.5 | 10.4 | **5.61×** | 11.6% |
| **TOTAL step** | **503.5** | **398.1** | **1.26×** | |

Serial floor (flood+accumulate+transport) = 381.7 ms/it = **75.8%** → Amdahl
max ≈ **1.3×**.

### 6.8 M cells (96 m), the B/4 scale

| phase | scalar ms/it | parallel ms/it | speedup | % of scalar step |
|---|---:|---:|---:|---:|
| uplift | 5.12 | 4.91 | 1.04× | 0.2% |
| surface | 7.22 | 6.72 | 1.07× | 0.2% |
| **flood** | **2115** | **2061** | **1.03×** | **71.5%** |
| route | 202.7 | 22.5 | **9.02×** | 6.9% |
| accumulate | 53.9 | 52.8 | 1.02× | 1.8% |
| transport | 309.3 | 308.2 | 1.00× | 10.5% |
| weather | 36.8 | 6.48 | **5.68×** | 1.2% |
| diffuse | 228.0 | 39.1 | **5.84×** | 7.7% |
| **TOTAL step** | **2958** | **2501** | **1.18×** | |

Serial floor = 2478 ms/it = **83.8%** → Amdahl max ≈ **1.2×**.

**The per-cell phases parallelize well** (route 9×, diffuse/weather ~5.7×), **but
they are only ~20% of the step.** The priority-flood alone is 65–72% and it is
serial. The whole-step byte-identical speedup is therefore only **1.18–1.26×**,
and the serial share *grows* with scale (see full-B below).

## Thread scaling (6.8 M, whole parallel step, ms/it)

| threads | ms/it | speedup vs. scalar |
|---:|---:|---:|
| scalar (ref) | 2940 | 1.00× |
| 1 | 2918 | 1.01× |
| 2 | 2704 | 1.09× |
| 4 | 2574 | 1.14× |
| 8 | 2554 | 1.15× |
| 12 | 2547 | 1.15× |
| 16 | 2529 | 1.16× |

**The byte-identical path saturates at ~1.16× by 4–8 threads and does not
improve past it.** The parallelizable phases stream large arrays (route/diffuse/
weather touch the whole grid once each) — they are **memory-bandwidth bound**, so
6 physical cores sharing one memory controller plateau almost immediately. More
threads (12, 16 — oversubscribing 12 logical / 6 physical) buy nothing.

## Full B — the headline number (27.3 M cells, 48 m, parallel)

| | value |
|---|---:|
| grid | 5222² = 27 269 284 cells |
| parallel ms/iter (4 iters) | **21 155 ms/it** |
| **projected 200-iter parallel wall** | **4231 s ≈ 70.5 min** |
| serial floor (flood+accum+transport) | 20 836 ms/it = **98.5%** of the step |
| memory | grid 728 MiB + scratch 1898 MiB ≈ **2.6 GiB** (recorder off) |

This is the whole point, and it is brutal: **at B scale the byte-identical
parallel path is no faster than scalar** (S9 measured ~63 min / 18.9 s/it
scalar; parallel here is ~70 min / 21.2 s/it — within noise/overhead of the
same thing). The flood so dominates at 27 M cells — its heap + `filled` array
(~700 MiB) blows every cache, so each heap op is a DRAM miss and cost grows
super-linearly — that the 20% of the step we *can* parallelize shrinks to ~1.5%
of the total. Parallelizing everything that is byte-identically parallelizable
buys **nothing** for B.

## The flood is the only lever — and it can't be pulled deterministically

Since the flood *is* the cost, the flip lives or dies on parallelizing the
priority-flood itself. We measured the **optimistic ceiling**: a tiled flood that
splits the grid into row strips, fills each in parallel, and treats internal
seams as *open* outlets (no reconciliation passes — the best case for speed, and
the worst case for correctness).

| scale | serial flood | 4 strips | 8 strips | 12 strips | max divergence vs. serial |
|---|---:|---:|---:|---:|---:|
| 1.7 M | 331 ms | 4.05× | 6.92× | **9.21×** | 70–85 m at ≤0.41% of cells |
| 6.8 M | 2042 ms | 4.53× | 7.91× | **10.9×** | 110.9 m at ≤0.38% of cells |

Two findings fall out:

1. **The flood tiles far better than the streaming phases** — up to **~11×** on
   12 strips, well past the 1.16× bandwidth ceiling. It is *cache-latency*
   bound, not bandwidth bound: each strip's working set is 1/12 the size and
   fits cache, so tiling is a locality win on top of the core-count win. So a
   parallel flood is genuinely fast — *if you accept the seams*.

2. **The seams are not free.** The open-seam fill mis-fills any depression that
   straddles a strip boundary — up to **110 m** wrong, at ~0.4% of cells. A
   *correct* parallel priority-flood (Barnes 2016) must pay that back with
   border-relaxation / spill-graph reconciliation sweeps, which erode the raw
   ~11×, and which are **not byte-identical to the serial fill** regardless.
   So the flood cannot go on the determinism-preserving path at all; a parallel
   flood is a different (approximate or separately-verified) algorithm, not a
   drop-in speedup.

### What even an ideal parallel flood buys at B scale

Grant the best case — a *free* flood — and read the remaining floor off the
profile. At 27 M cells, transport and accumulate (both serial flux chains, ~11%
+ ~2% of the step and super-linear in n) remain: roughly **1.5–1.7 s/it × 200 ≈
5–6 min** of irreducible serial transport, plus the parallel phases (~0.3 s/it).
So:

| B on this 6-core box | 200-iter wall |
|---|---:|
| scalar (S9) | ~63 min |
| byte-identical parallel (measured) | ~70 min — **no gain** |
| + ideal *free* flood, transport still serial | **~6–7 min floor** |
| + realistic ~6× parallel flood (post-reconciliation) | **~15–20 min** |
| the flip target | **~2 min** |

**~2 minutes is ~35× off scalar. Six physical cores cannot deliver 35×** —
the hard ceiling from core count alone is ~6–12×, and the two levers that reach
even that (a parallel flood, a parallel transport) are both non-deterministic
rewrites. The flip does not happen on consumer hardware.

## Recorder memory (the 648 MiB S9 flagged)

`size_of::<DeepStrata>()` = **32 B/cell** (a 24-byte `Vec` header + `bool` +
`u32`, padded), *before a single unit is stored*:

| scale | empty struct array |
|---|---:|
| A 460 m (297 k) | 9 MiB (negligible) |
| B 48 m (27.3 M) | **833 MiB** |

Measured at 500 m: record ON = 25.1 MiB resident (251 k units); record OFF =
6.7 MiB. A flat-array recorder (one shared `Vec<DepUnit>` + per-cell
`(start:u32, len:u32)`) would cut the per-cell overhead from 32 B to 8 B → the
B-with-record header cost from 833 MiB to ~208 MiB. **But this only bites
B-with-record**, which we already recommend against; for A (9 MiB) and for
B-record-off (the datapoint that matters) it is moot. Recorded here as a measured
proposal, not built — it does not move the verdict. (Note: S9's "648 MiB" counted
only the 24-byte `Vec` header; the full struct is 32 B → **833 MiB**. Minor
correction, filed.)

## Verdict — A+C vs. B on total cost of ownership

**Ship A always-on + C bounded refinement. Do NOT build B. The parallelism flip
condition is falsified for this class of machine.**

The reasoning, now with parallel numbers:

1. **The determinism-preserving speedup is ~1.2× and vanishes at scale.** The
   only phases you can parallelize byte-identically are ~20% of the step at
   sub-scale and ~1.5% at B scale. Byte-identical parallel B ≈ scalar B ≈ 70 min.
   The mandate that parallel equal scalar to the bit — which we *want*, because
   worldgen determinism is load-bearing — is exactly what forecloses the flip:
   the one hot phase (flood) has no byte-identical parallel form.

2. **The flood tiles to ~11× but only by giving up determinism** (110 m seam
   error) and only cache-locally; a correct version costs reconciliation passes.
   And even a *free* flood leaves a ~6–7 min serial-transport floor at B. So the
   realistic parallel B on 6 cores is ~15–20 min — better than 63, still an
   order of magnitude past the ~2 min that would make B's simplicity win.

3. **Core count is the wall, not code effort.** 35× is unreachable on 6 physical
   cores no matter how much engine work we do. B could *approach* the flip only
   on a many-core workstation (~32+ cores) AND with a correct parallel flood AND
   a parallel transport — three non-trivial, determinism-breaking pieces — versus
   C, which S9 already proved sound (drainage decided coarse, relaxation
   processes haloed at 16–24 cells). Trading a proven architecture for three
   speculative rewrites gated on unknown target hardware is the wrong bet.

4. **The S9 nuance survives intact.** B stays fine for **Small** worlds (~2.4 M
   cells, already <2 min scalar). And on a known-beefy many-core box, a parallel
   flood could bring Medium B into a ~5–8 min "coffee break" — a defensible
   middle path *only* if code-simplicity is weighted very heavily and the
   deployment hardware is guaranteed. For the general case (consumer hardware,
   Medium/Large worlds), **A+C is the answer**, and this spike is the measurement
   that closes the question the user's recorded skepticism kept open.

### What would change this

A single measured datapoint would reopen B: **a correct, deterministic parallel
priority-flood *and* parallel transport, benchmarked at 27 M cells on ≥16
physical cores, landing the full 200-iter run under ~2–3 min.** Until that exists
and the target hardware is known, the flood is a serial wall and A+C wins. If 3e
ever targets a fixed many-core server (not player machines), re-run
`examples/deeptime_par.rs --full-b` there before reconsidering.

## Falsified assumptions (→ corrections.md candidates)

1. **"A parallel priority-flood could pull B to ~2 min" (S9 flip condition /
   earth-processes.md § S9 VERDICT).** Falsified for ≤12-thread machines. On 6
   physical cores: the byte-identical parallel path gives ~1.2× (≈0× at B scale);
   the flood tiles to ~11× only non-deterministically (110 m seam error) and
   cache-locally; and even a free flood leaves a ~6–7 min serial-transport floor.
   Measured realistic parallel B ≈ 15–20 min, not 2. The "~2 min" needed ~35×,
   which 6 cores cannot supply. True only on a many-core box with two further
   non-deterministic rewrites.
2. **Implicit "parallelizing the engine is the lever" (S9).** Half-true: it is
   *a* lever but a weak one under the determinism mandate. The byte-identically
   parallelizable work is a minority of the step and memory-bandwidth-saturates
   at ~1.16× by 8 threads. The dominant cost (priority-flood) is precisely the
   part with no byte-identical parallel form.
3. **Minor: recorder empty-header cost "≈ 648 MiB" at B (S9 § footnote 2).** The
   per-cell struct is 32 B, not the 24 B `Vec` header alone → **833 MiB** empty
   at 27.3 M cells. Undercount by ~28%.

## Loose ends

- **A correct deterministic parallel flood is unbuilt** (only the optimistic
  open-seam probe exists, for measurement). Barnes' spill-graph join is the known
  route; it is ~300–500 LoC and still not byte-identical to the serial fill, so
  it would need its own equivalence bar. Out of scope until B is reconsidered.
- **Transport/accumulate level-parallelism is unbuilt** and would also break
  byte-identity (gather reorders the flux sums). It is the *second* serial wall
  after the flood — any B revival must clear both.
- **The flat recorder is measured, not built** — an 8 B/cell layout saving
  ~625 MiB at B-with-record, but B-with-record is not a recommended config.
