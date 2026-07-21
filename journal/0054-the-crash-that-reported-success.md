# 0054 — The crash that reported success

Two entries have now been written about DeviceLost without anyone touching the
DeviceLost. journal/0050 diagnosed the leak that provoked it; journal/0051
removed the leak. Both ended with the same sentence in different words: *the
secondary defect stands — a DeviceLost should degrade loudly, not cascade into
`unwrap`/`PoisonError` panics.* This entry is that slice.

It is worth saying up front why it is worth doing at all now that the cause of
the observed losses is fixed. A lost device is not a symptom of our leak. It is
a permanent property of running on a GPU: a driver bug, a Windows TDR, a GPU
reset, an eGPU cable, another process eating the last of VRAM. Any of those and
the device is gone, and the only question left is whether the game says so.

## The failure, as recorded

From the ROADMAP Observed entries, the same shape twice:

```
DeviceLost ("driver implementation is at fault")
  → swap-chain loss
  → wgpu buffer-map panic
  → bevy_pbr cluster PoisonError cascade
```

Three things are wrong with that tail, in ascending order of how badly they
mislead a reader.

First, the root cause is at the *top*, and everything after it is louder. By
the time the user or an agent looks, the visible end of the log is a cluster of
`PoisonError`s in `bevy_pbr` — code that has nothing to do with what happened.

Second, the poison is self-inflicted amplification. One thread panics while
holding a `Mutex`; the mutex is now poisoned; every later `.lock().expect(…)`
panics too, and each of those panics names *the mutex*. Our own
`Arc<Mutex<WorldGenerator>>` seam had five such `expect`s. A single failure
anywhere in the process could convert all five into panics that report a
worldgen mutex when the actual event was a GPU falling out of the machine.

Third, and worst: **the process reported exit 0.** Both recorded crashes did.
The ROADMAP carries a standing warning about it, and CLAUDE.md's walk protocol
tells agents never to characterize a session end from its exit code — a rule
that exists purely to route around this defect.

That third one is the interesting bug, and it took a minute to see why it
happens. The panics are on *worker* threads. Rust's `101`-on-panic only applies
to a panic that unwinds out of `main`; a panic on a spawned thread kills that
thread and nothing else. Bevy runs rendering and its task pools off the main
thread, so the cascade quietly killed the render side while the main thread's
event loop wound down normally and returned `AppExit::Success`. The process
then *did* exit successfully, in the only sense the OS knows. The exit code was
not lying so much as answering a different question than the one being asked.

## What the API actually allows

The brief said: investigate what wgpu and Bevy offer at this version, and do not
invent a mechanism the API cannot support. Three findings, one of which killed
the obvious plan.

**wgpu 29 has exactly the hooks you would want, and Bevy 0.19 already owns
both.** `Device::set_device_lost_callback` and `Device::on_uncaptured_error`
exist, and `bevy_render::error_handler::DeviceErrorHandler::new` registers both
at device creation. Neither is a chain — registering ours would *replace*
Bevy's, which is how the error reaches Bevy's own state machine at all. So the
first plan (install our own device-lost callback so we are the earliest
possible reporter) was wrong, and would have broken Bevy's error handling to buy
a few milliseconds of latency.

**A `DeviceLost` never arrives through `on_uncaptured_error` anyway.** wgpu's
core backend drops it there explicitly:

```rust
ErrorType::DeviceLost => return, // will be surfaced via callback
```

(`wgpu-29.0.4/src/backend/wgpu_core.rs:297`). The device-lost callback is the
only path, and Bevy holds it. Worth recording because "just add an uncaptured
error handler" is the answer a reasonable person reaches for first, and it
cannot see this class of error at all.

**What Bevy leaves us is the policy seam, and it is a good one.** Bevy 0.19
added `RenderErrorHandler` — a `Resource` wrapping a plain fn pointer, called
with the classified `RenderError` and *both* worlds, returning a
`RenderErrorPolicy`. The default logs one line and writes `AppExit::error()`.
Overriding it is a one-line `insert_resource`, and it is the correct place: by
the time it fires, wgpu's error has been classified into `ErrorType::DeviceLost`
for us, and we hold a `&mut World` to act on.

So the honest summary is that Bevy 0.19 has already done the hard part —
catching the loss without panicking — and the defect that remains is entirely
ours: what the client does with the news, and what the process tells the world
on the way out.

## The mechanism

`crates/dc-client/src/devicelost.rs`, four parts.

**A banner at detection.** Our `RenderErrorHandler` classifies, and on
`ErrorType::DeviceLost` calls `report_device_lost`, which prints a boxed
`error!` naming the loss as the root cause and pre-emptively labelling
everything after it as downstream noise. A `Once` guards it: a device is lost
once, and the *first* reason is the cause. It then writes `AppExit::error()` and
returns `RenderErrorPolicy::StopRendering` — a controlled shutdown, with the
window and the two MCP server threads torn down the ordinary way.

**A terminal summary.** This is the part I would defend hardest, and it was not
in the original plan. A banner at detection is worth little if the thing that
follows it is a thousand lines of cascade — the reader's `tail` shows the noise,
not the banner. So `App::run`'s return value now flows back to `main` through
`devicelost::finish`, which prints a three-line diagnosis as the **last** thing
in the log:

```
[dc-client] SESSION ENDED ABNORMALLY
  root cause : GPU DEVICE LOST — <reason>
  panics     : 4 (all downstream of the device loss)
  exit code  : 70
```

The end of a log is the one location that cannot be buried. Property 1 of the
brief said "stated once, unmistakably, at the top of the failure"; the honest
improvement is to state it at the top *and* at the bottom, because those are two
different readers.

**The exit code stops lying.** `main` now returns `ExitCode`, derived by
`exit_code_for` from *what happened*, not from whichever `AppExit` the event
loop managed to produce:

| session | before | after |
|---|---|---|
| clean quit | 0 | 0 |
| device lost | **0** (recorded, twice) | **70** |
| panic on any thread, device fine | **0** if off-main | **101** |
| other fatal render error | 0 or 1 | 71 |

A device loss or a panic outranks `AppExit::Success`, which is exactly the case
the recorded crashes hit. Counting panics on *any* thread needs a process-wide
panic hook, so there is one — chained in front of the existing hook, so the
normal message and backtrace still print. It also labels a panic that arrives
after a device loss as downstream, and, once a cascade is evident (device lost,
or three panics), requests a shutdown. That last bit closes the zombie case: a
process whose render threads are dead no longer sits there pretending.

**The poison cascade is removed at the source.** `lock_forgiving` —
`m.lock().unwrap_or_else(|p| p.into_inner())` — replaces the `expect` at all six
client mutex seams: the five `Arc<Mutex<WorldGenerator>>` acquisitions in
`authority.rs` and the MCP bridge's receiver.

Poison tolerance needs a justification, not just a call site, because
`PoisonError` exists for a real reason. It is sound here because none of these
locks guard an invariant that spans an acquisition. The `WorldGenerator` behind
the notable one guards pure memoization caches: every entry is a deterministic
function of its key, a `HashMap` insert either happened or it didn't, and a
half-written *entry* is not representable. A panic mid-`generate_chunk` can
leave a cache with fewer entries than it might have had; it cannot leave one
with a wrong entry. The 0051 invariant — evict-then-regenerate is
byte-identical — is the same property viewed from the other side, and it is
already tested. The MCP bridge's lock guards a queue, where the worst a
poisoned recovery costs is a dropped request.

The bridge is a small separate win. It previously read
`let Ok(mut rx) = bridge.rx.lock() else { return };` — poison-tolerant in the
sense that it does not panic, but the effect is that after one unrelated panic
the client stops draining MCP requests **forever**, silently. An agent's tool
calls would simply stop being answered with nothing said anywhere. A poisoned
queue is still a queue.

Last, `renderer_healthy` gates the whole gameplay `Update` chain. Once the
device is gone there is nothing to stream chunks or build meshes *for*, and each
further frame is another chance to manufacture noise on top of the real cause.

## The wrong turn worth recording

I spent a while designing around `RenderErrorPolicy::Recover(RenderCreation)`,
which Bevy 0.19 offers and which sounds like it makes this whole entry
unnecessary — the renderer recreates its device and the game plays on.

It is out of scope and stays out. Recovery recreates every GPU resource in the
render world, and our renderer holds a lot of hand-rolled state that has only
ever been created once: the shader-pack post stage, the `--edges` pass, the
terrain material's assembled placeholder atlases, and some thousands of streamed
chunk and far-tile meshes with their render-world entities. None of that has
been exercised against a re-created device, and none of it *can* be exercised
against one, because we cannot induce a real device loss on demand. Shipping an
untestable recovery path would mean that the day a device is genuinely lost, the
game takes an untrodden branch instead of a known one. An honest shutdown is
strictly better than a recovery nobody has ever watched work.

## What I can prove, and what I cannot

The dishonest version of this entry claims the DeviceLost path is fixed. It
isn't provable that way, and it is worth being precise about the boundary.

**Proved, by test.** `lock_forgiving` returns the real value from a genuinely
poisoned lock (a thread panics mid-mutation; the assertion reads the mutation
back). The exit-code contract: clean → 0, fatal render error → 71, and a
reported device loss forces non-zero *even against `AppExit::Success`*, which is
the precise shape of the recorded crashes. The first reason wins and later ones
do not overwrite it.

**Proved, live.** The whole path downstream of classification, via
`DC_SIMULATE_DEVICE_LOST=<seconds>` — an env-gated diagnostic in the
`DC_MEM_PROBE` family, zero cost when unset. It calls the same
`report_device_lost` seam the real handler calls, so the banner, the flag, the
gameplay gate, the shutdown pump, the terminal summary and the process exit code
are all the real ones.

**Not proved, argued.** The link from a *real* wgpu device loss to our handler:
wgpu's device-lost callback → Bevy's `DeviceErrorHandler::poll` →
`RenderState::Errored` → `RenderErrorHandler`. That is Bevy's code between two
points I have read, and I have not seen it run. I cannot induce a real
`DeviceLost`, so this is source-reading, not measurement. It is the one seam in
the chain I am taking on faith, and I would rather name it than let it hide.

## An impossible green, which is worse than an impossible red

Worth recording because the brief warned about the opposite failure and this is
its mirror. The brief said: on an *impossible red* against source that plainly
has the symbols, suspect a stale artifact from the sibling worktree's shared
build cache. What happened instead was an **impossible green**. The workspace
test gate passed — 489 tests, exit 0 — and the three tests written for this
slice did not appear in it. `cargo` had reused a `dc_client-*.exe` timestamped
10:49, minutes before the new module existed, out of the shared
`CARGO_TARGET_DIR` two agent worktrees were both writing to. Every gate said
green and none of them had run the new code.

A red that lies announces itself. A green that lies does not, and the only
reason it was caught is that the *test names* were checked in the log rather
than the `test result: ok` line — which is exactly why the brief asks for the
lines, not the summaries. `cargo clean -p dc-client --release` and a re-run
produced 102 dc-client tests including all three, still green. The gate output
quoted in the return report is the post-clean one; the first one is worthless.

## Repro

```
# exercise the whole reporting + shutdown + exit-code path (15 s in):
DC_SIMULATE_DEVICE_LOST=15 cargo run --release -p dc-client -- --horizon 3
# expect: the boxed DEVICE LOST banner, a clean window teardown,
#         the SESSION ENDED ABNORMALLY summary as the last lines, exit 70.

# healthy control: same launch without the variable → exit 0, no banner.
```

Observed, on an RTX 3070 / Vulkan / driver 591.86 session with the trigger set
to 12 s — the banner, then a clean teardown, then the tail, then **exit 70**:

```
15:32:30 ERROR dc_client::devicelost:
╔══════════════════════════════════════════════════════════════════╗
║  GPU DEVICE LOST — this is the ROOT CAUSE of everything below.   ║
╚══════════════════════════════════════════════════════════════════╝
reason: SIMULATED by DC_SIMULATE_DEVICE_LOST (diagnostic trigger — the real device is fine)
…
[dc-client] SESSION ENDED ABNORMALLY
  root cause : GPU DEVICE LOST — SIMULATED by DC_SIMULATE_DEVICE_LOST …
  panics     : none
  exit code  : 70
```

The healthy control (same binary, variable unset, 45 s, `--horizon 3`) logged
no `ERROR`, no `panicked`, no `DeviceLost`, closed its window the ordinary way
and exited 0 — judged from the log tail, which is the whole subject of this
entry.

> blogworthy: "the crash that reported success". The whole entry turns on one
> unglamorous fact about Rust and threads — a panic only becomes exit code 101
> if it unwinds out of `main`, and Bevy does its rendering somewhere else — so a
> GPU dying mid-frame produced a process that exited 0 and a log whose last
> hundred lines blamed a mutex in `bevy_pbr`. Every layer was behaving
> correctly and the composite was a lie. The fix is not clever; it is deciding
> that the exit code should answer the question people are actually asking, and
> that the *end* of a log is the only place a root cause can't be buried.
