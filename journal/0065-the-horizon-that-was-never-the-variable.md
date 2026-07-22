# 0065 — The horizon that was never the variable

journal/0051 bounded `HostWorld.chunks` and measured the RAM march flat —
+27.4 MB/jump before, 0.00 MB/jump after, over 147 jumps. It measured that at
`--horizon 3`. Every DeviceLost death in this project's history happened at
`--horizon 6`, so the fix has been carried in ROADMAP for two sessions as
"proven at 3, unproven at 6", and the re-run has been sitting there as *a
cheap, high-value first act*. This is that act.

The short answer: **a long `--horizon 6` session survives, and the horizon is
not the variable.** Every number that matters is the same at 6 as at 3. The
one thing horizon 6 actually costs is a **constant** ~155 MB of steady-state
footprint, paid once when the far field fills, and never paid again.

## Two regimes, because the storm is blind to the horizon

The obvious method was 0051's: `DC_MEM_PROBE=1`, a fresh ±15 km teleport every
1.8 s, RSS sampled externally once per jump. Running it at horizon 6 for the
first time exposed why it alone could not have answered the question.

Under the storm the far field **never fills**. The viewer moves every 1.8 s and
mesh building saturates ~100 % of wall time, so the probe shows `far_tiles`
oscillating between 2 and 40 for the whole run — at *either* horizon. The
storm's working set is the near field plus whatever the streamer managed in
1.8 s, and that is capped by `LOAD_BUDGET_PER_FRAME`, not by `--horizon`. A
storm at horizon 6 is, to a first approximation, a storm at horizon 3 with a
larger camera far plane.

That is also the mechanical reason for a fact journal/0050 recorded and did not
explain: "h3 and h6 jump slopes are near-identical". They are near-identical
because the storm is nearly *blind* to the horizon. (Instrument-must-see-the-
question, one level up: the *stimulus* has to be able to reach the thing you
are asking about, not just the instrument.)

So the wide-horizon question needs the other regime — the one the historical
crashes actually died in. Two of the three recorded DeviceLost deaths were
**idle**: boot at horizon 6, let the field stream, sit still, die at ~4.5 min.
That regime is where the far field is *fully materialised and resident*, and it
is the only regime in which `--horizon` is a real variable. So: two harnesses.

- **storm** — 200 jumps at 1.8 s, ~7 min, RSS per jump.
- **settle** — boot, let the field fill, sit still 8 min, RSS every 5 s.

Both alternated 6 / 3 / 6 / 3 rather than run as two blocks, because machine
drift between sets has exceeded a real effect in this project before.

## The storm

| run | horizon | jumps | RSS start → end | slope (from jump 50) |
|-----|--------:|------:|-----------------|---------------------:|
| c6a | **6** | 200 | 664 → 1 095 MB | **+0.542 MB/jump** |
| c3a | 3 | 200 | 659 → 1 102 MB | +0.531 MB/jump |
| c6b | **6** | 200 | 660 → 1 101 MB | **+0.529 MB/jump** |
| c3b | 3 | 200 | 662 → 1 096 MB | +0.542 MB/jump |

Four runs, two horizons, and the horizon signal is **zero** — the spread within
a horizon (0.529–0.542) is the same size as the spread between them. Every run
exited **0**, and not one logged a `WARN`, an `ERROR`, a `DeviceLost` or a
panic. RSS high-water was 1.10 GB.

`host_chunks` sat at 2 108–2 332 against a budget of **2 360** in all four runs,
with 18 000–19 800 chunks evicted per run. The store 0051 bounded is doing
exactly what it was built to do, at both horizons — and note that the budget is
**2 360 at horizon 6 and 2 360 at horizon 3**, which is the first falsified
assumption of this slice (below).

The one apparent wrinkle: **+0.54 MB/jump is not 0.00 MB/jump.** 0051 reported
a flat 1 072 → 1 072 MB over its steady window; the same storm today looked
like it rose half a megabyte per jump, at *both* horizons. Fifty times smaller
than the pre-eviction +27.4, and certainly not the chunk store (capped, visibly
evicting) — but not nothing either, and a slope you cannot explain is a slope
you have to chase.

## The 200-jump window was the wrong window

So the last run was a single **700-jump, 23-minute** session at horizon 6, and
it dissolved the wrinkle:

| jump | 50 | 150 | 250 | 350 | 450 | 550 | 650 | 700 |
|------|---:|----:|----:|----:|----:|----:|----:|----:|
| RSS (MB) | 1 017 | 1 063 | **984** | 1 075 | 1 127 | **1 046** | 1 136 | 1 166 |

It is a **sawtooth**, not a march. RSS climbs for roughly 250 jumps, drops
120–180 MB in one step, and climbs again, oscillating in a ~980–1 170 MB band
for the whole session. Regressed over the full run the slope is **+0.113
MB/jump** (jumps 50–700) and **+0.074 MB/jump** over the last 300 — an order of
magnitude below what the 200-jump windows reported, because every one of those
windows happened to sit **inside a single tooth**. Four runs agreeing on +0.54
was four runs making the same framing error, not four confirmations. 0051's
0.00 MB/jump and today's +0.54 are the same phenomenon read at two phases of
the same oscillation.

Final state of that run: 700 acknowledged teleports, **71 928 chunks evicted**,
`host_chunks=2224` against `host_budget=2360`, RSS high-water **1 166 MB**,
exit code **0**, and not one `WARN`, `ERROR`, `DeviceLost` or panic in
twenty-three minutes.

The residual that survives — ~0.07–0.11 MB/jump — is small enough to be
allocator hysteresis and is horizon-independent either way. If someone does
want to chase it, journal/0050 already named the candidate and left it open:
the `WorldGenerator` collapse caches' far-field-only sampling paths
(`coarse_surface` / `column_record`) never trigger `evict()`, which predicts
exactly this shape — position-keyed, indifferent to the horizon. Filed to
Observed, not chased here; this slice's write-set was the harness.

## The settle — where the horizon finally is a variable

| run | horizon | resident far tiles | RSS after fill | drift over the next 7 min |
|-----|--------:|-------------------:|----------------|--------------------------:|
| s6a | **6** | **704** | 980 MB | **+6.9 MB** |
| s3a | 3 | 288 | 821 MB | +12.7 MB |
| s6b | **6** | **704** | 982 MB | **+5.4 MB** |

This is the regime that used to kill the client in four and a half minutes, and
it is **flat**. Not shallower — flat: after the field fills, every count in the
probe freezes (`meshes=790 far_tiles=704 far_tile_ent=704 entities=1191
host_chunks=494 near_built=0.0/s far_built=0.0/s`) and stays frozen for the
whole run, and RSS drifts by less than 7 MB in seven minutes. Horizon 6
drifted *less* than horizon 3. Both exited 0 with clean logs.

The horizon's real cost is now a number: **704 far tiles versus 288, and
~155 MB of resident footprint versus horizon 3** — constant, paid once at fill,
never paid again. That is what "wide horizons cost" means after 0051, and it is
a completely different shape of cost from the one that was killing sessions.

## Falsified on the way in

**`Authority::chunk_budget_for` does not scale with the horizon.** The brief for
this work said it did, and it is a reasonable thing to assume — it is the
client-supplied working-set size, so surely a wider view means a bigger set.
It does not: the budget is derived from `streaming::UNLOAD_RADIUS_M`, which is
`FULL_DETAIL_RADIUS_M + 32` — a **constant**, 160 m, the *near* field. It
resolves to 2 360 chunks at the boot scale at every horizon, and the probe
confirms it (`host_budget=2360` in every run). This is correct, not a bug:
`HostWorld.chunks` backs voxel-resolution queries, and the far field is served
from the coarse summary, which never enters the chunk store. But it means the
sentence "the budget scales with horizon, so check it at 6" was asking about a
quantity that does not exist.

## The harness, and an hour lost to a stale port

Two failure modes worth recording, because both produced *plausible* data.

**A deleted worktree, mid-set.** The agent worktree this ran from was pruned by
external cleanup while the first pair of runs was in flight. The horizon-6 run
had already booted and loaded its assets; the horizon-3 control booted after,
found no `assets/packs/default/pack.toml`, logged one `WARN`, silently fell back
to the built-in shader pack and fallback texture layers, and ran to completion
with a perfectly clean-looking RSS curve. It was not a like-for-like control
and nothing in the numbers said so. The measurement binary is now a **copy** in
scratch and the working directory is the main checkout, so neither a sibling's
rebuild nor a worktree prune can move the ground under a run.

**A stale client on port 7777.** A leftover client from the aborted set was
still holding the MCP port when the next run launched. The new client came up
five seconds *later*, so the driver's `initialize` handshake succeeded against
the **wrong process** — and drove it. The logged client sat perfectly still for
seven minutes while its RSS curve went flat at +0.05 MB/jump. That is the
prettiest result of the whole day and it is worth exactly nothing: a flat
memory curve is the *expected* output of a client that is not being asked to do
anything. The harness now waits for *its own child* to log `MCP: dev surface
on` before it will handshake, refuses to start while anything is listening on
7777, and counts acknowledged teleports (`ACKED=200`) so a run that did not
actually move announces itself.

> blogworthy: "the flattest graph I ever measured was of a program doing
> nothing". Both harness bugs failed *toward* the answer we were hoping for —
> a clean control and a flat curve — which is the direction a bug is hardest to
> notice. The fix in both cases was to make the harness assert the thing it had
> been assuming: *this* process, *these* assets, *this many* teleports actually
> applied. A measurement that cannot fail loudly is not a measurement.

## Verdict

**Yes — a long `--horizon 6` session survives.** Eight client launches across
storm and idle regimes at both horizons, ~80 minutes of runtime including one
unbroken 23-minute horizon-6 session at 700 teleports, every one exiting **0**,
no `DeviceLost`, no panic, no `ERROR` line anywhere.

The ceiling, honestly: idle at horizon 6 is **flat**, and continuous
teleport-storming at horizon 6 is a **bounded oscillation** — ~980–1 170 MB,
sawtoothing, +0.07–0.11 MB/jump of residual trend over 700 jumps. Neither
regime has a memory ceiling worth naming on a 32 GB machine, and the residual
is horizon-*independent* — it would read the same at horizon 3. Horizon 8 or
10 should hold on this evidence — what
they buy is more constant footprint (704 tiles at 6 came from 288 at 3), and
that is a footprint question with a known shape, not a leak question. The
number to watch when someone does try 10 is `far_tiles` at fill, not slope.

## Repro

```
DC_MEM_PROBE=1 dc-client --horizon 6     # storm: teleport +-15 km every 1.8 s
DC_MEM_PROBE=1 dc-client --horizon 6     # settle: boot, sit still 8 min
# watch host_chunks/host_budget/far_tiles in the probe; sample RSS externally.
# alternate 6/3/6/3 rather than running two blocks.
# and run ONE long session (700 jumps / 23 min): under ~250 jumps you are
# measuring inside a single tooth of a sawtooth and will read a slope that
# is not there.
```
