# 0090 — the runner names what the loop already did

Movement 1 of the deep-time-loop → declared-passes conversion. The deep-time
erosion sim has always been a hand-written `for it in 0..iterations` loop with a
fixed sequence of phase calls inside `Erosion::step`. This slice re-houses that
loop onto a **deep-time pass-runner**: every phase becomes a **self-declaring
pass** (`{reads, writes}` + cadence), and the order that used to be a hand-kept
list of method calls now **falls out of the declared reads/writes** via a
topo-sort — the north-star pass shape (`docs/design/north-star.md` § Passes),
instantiated one tier down from the pregen `pipeline`.

It is a re-housing: the pass bodies are the *same method calls in the same
order*. The acceptance instrument is the production goldens, and they do not
move.

> blogworthy (lens: reflexions in a deepsim codebase; "the runner names what the
> loop already did"): the moment a hand-ordered loop is replaced by a graph that
> *derives* the same order is the moment the order becomes **inspectable and
> checkable** — an illegal schedule (a cycle, a conflicting writer) becomes a
> compile/build-time rejection instead of a silent bug. The interesting part is
> that a genuinely linear relaxation pipeline (transport → weather → diffuse →
> …) does *not* fall out of a dataflow graph for free: the terrain is read,
> transformed, read again — and you have to name each **revision** as a distinct
> resource for the topo-sort to reproduce a fixed sequence. The declaration is
> the honest statement of what the loop was always doing.

## Build on the runner that exists, not a second one beside it

`pipeline.rs` already had a pass-runner embryo: passes declare reads/writes, it
topo-sorts (Kahn, lexicographic-id tie-break) and rejects cycles / multiple
creators / ambiguous writers / missing producers. But it runs the **pregen DAG
once** to build world state from nothing, over a *world-level* resource
vocabulary (Plates, Elevation, Climate…). The deep-time sim is a different
execution model — a **loop** over state that persists across epochs, with a
**rate axis** and **loop-carried edges** — and its resources are *deep-cell*
axes (terrain stocks, the strata record, the drainage solve).

Rather than write a second topo-sort next to `pipeline`'s (the project's
characteristic failure — spines A-4), the shared graph math was extracted into a
generic kernel, `dc-worldgen/src/passgraph.rs`: `schedule(decls,
require_creator)` over any `Copy + Ord` resource token. `pipeline.rs` now calls
it (its seven existing graph tests — `vanilla_order_is_the_legacy_order_then_
geology`, `cycles_are_rejected_with_names`, … — pin that its behavior is
unchanged), and the deep-time runner calls it too. The one knob that separates
the two callers is `require_creator`: the pregen DAG demands every read have a
producer; the deep-time **loop** does not, because every axis is already there
from the pre-loop seed or the previous epoch.

## The runner shape

`deeptime/runner.rs`. A `DeepPass` declares `{ id, reads, writes, reads_prev,
period, body }`:

- **ORDER** comes from `reads`/`writes` over the `DeepAxis` vocabulary, handed to
  `passgraph::schedule`. The order is a pure function of the declarations.
- **RATE** is `period` (epochs between firings) — the runner fires a pass when
  `epoch % period == 0` — and `dt` (= phase length), threaded to each pass. This
  movement **pins** the rate: every pass keeps its current effective cadence and
  `dt` is inert (no transform scales by it yet — today's magnitudes are per-epoch
  constants). The RATE knob becoming live is a later movement; here it only has
  to *reproduce* today.
- **`body`** is a bare `fn(&mut DeepStepCtx)` — no closures cross the seam. The
  rich state (`grid`, `erosion` scratch, `biota`, the tectonic schedule) lives on
  the runner's side in `DeepStepCtx`; the declaration a future SDK/WASM backend
  would marshal is the plain-data part. That is **the crossing constraint** held
  (north-star § "The crossing constraint"): plain data + opaque ids, rich Rust on
  both sides of the boundary but never across it.

The runner drives the epoch loop: for each epoch, set the paleo-sea-level stand,
then run every pass in topo order that fires this epoch.

## Erosion decomposed — the pipeline is a chain of terrain revisions

`Erosion::step` (retained for its direct callers — the profiling and
erodibility/deeptime tests) is a monolith of ~16 phase calls. Per the
integrator's course-correction, it is **decomposed into separate declared
passes**, not kept composite. The production (all-flags-on) roster is fourteen
passes:

| pass | reads | writes | rate |
|---|---|---|---|
| `climate` | — (start-of-epoch topography, lagged) | Climate | `remarch_interval` |
| `tectonics` | — | Forcing | 1 |
| `forcing` | Climate, Forcing | Forced, CrustThick | 1 |
| `expose` | — (start-of-epoch record, lagged) | Exposed | 1 |
| `frost` | Forced | Frosted | 1 |
| `drainage` | Forced | Routed | 1 |
| `transport` | Routed, Exposed | Energy, DeltaH | 1 |
| `weather` | DeltaH, Exposed, Frosted | DeltaH, Weathered | 1 |
| `diffuse` | Weathered, Exposed, DeltaH | DeltaH, Diffused | 1 |
| `isostasy` | Diffused, CrustThick | Compensated | 1 |
| `deposition` | Energy, DeltaH, Climate, Compensated | Recorded | 1 |
| `eolian` | Exposed, Climate, Recorded, Compensated | Recorded, Windblown | 1 |
| `wave` | Exposed, Recorded, Windblown | Recorded, Settled | 1 |
| `biotic` | Routed, Frosted, Recorded, Settled | BioMod, BioRecorded | 1 |

The load-bearing discovery: a **field relaxation pipeline** does not fall out of
a dataflow graph the way a DAG-that-builds-state does. Nearly every erosion phase
*reads the terrain and writes the terrain*, and several read an **intermediate**
terrain state (the recorder reads the post-isostasy terrain; then the wind/wave
agents transform it *further* and self-record). A single shared "Terrain" axis
can't express that — the topo model orders a reader after **all** writers, which
would drag the recorder to the end. So each terrain-transforming stage exposes
its output as a distinct **revision token** the next stage consumes: `Forced →
… → Weathered → Diffused → Compensated → Windblown → Settled`. `DeltaH` (the
per-epoch thickness change) is the other spine — `transport` **creates** it,
`weather`/`diffuse` add to it, `deposition` reads it, which forces transport →
weather/diffuse → deposition for free. The topo-sort then reproduces `step`'s
exact phase order (the two independent per-cell passes `expose` and `frost` are
interleaved differently by the id tie-break, byte-identically because their
writes are disjoint).

`transport` keeps its incision fused (entrainment and bedrock incision are one
interleaved per-cell flux chain; splitting them is an algorithm change, not a
re-housing — out of scope). `isostasy` folds `track_exhumation`; `transport`
folds the tectonic bedrock snapshot — exactly where `step` had them.

## The two preserved facts

**The biology↔erosion one-epoch lag is a declared loop-carried edge.** `biotic`
*creates* `BioMod` (the `bio_weather`/`bio_resist` planes); the erosion
`weather`/`diffuse`/`eolian` passes read them — but as **`reads_prev`** (the
previous epoch's value), which is *deliberately not handed to the topo-sort*. So
the within-epoch graph runs `…erosion… → biotic` (via `Recorded`/`Settled`) with
no back-edge. Move that `BioMod` read into `reads` and the runner **rejects the
schedule as a cycle** — the guarantee ecology.md wanted (a single-epoch pass
graph would refuse the biology↔erosion cycle) is now *enforced by the runner*
rather than trusted from a comment. A test (`the_biology_erosion_lag_must_be_
loop_carried_or_the_runner_rejects_a_cycle`) proves the rejection.

**`climate`'s coarser cadence is a low-rate pass.** It re-marches every
`remarch_interval` (20) epochs, not every epoch — the one real instance of the
RATE axis already in the loop. It is modeled with `period = remarch_interval`,
seeded before the loop (epoch 0's march is initial conditions) and firing on its
in-loop multiples — exactly the old `it > 0 && it % remarch_interval == 0`.

## The wrong turn: the stale sea level

The first decomposition moved a golden. The materials/table fingerprints of the
*small* world were byte-identical but its *blocks* moved; the *medium* world
moved on all three — a small, threshold-sensitive numerical divergence, the
signature of a sea-level effect.

`Erosion::step`'s **first line** is `self.sea_level = sea_level;`. The phase
methods (`flood`, `transport`, `record`, `wind`, `wave`) read `self.sea_level` —
the sinusoidal paleo-sea-level stand that drives the shoreline, marine
deposition, and the entire wave agent. Decomposing `step` into passes that call
those methods *directly* dropped that assignment, so every erosion sub-pass ran
against the stale constant `SEA_LEVEL_M` instead of `sea_level_at(cfg, it)`. The
fix: the runner sets `erosion.set_sea_level(ctx.sea_level)` once per epoch,
before the passes fire. With that, **the goldens do not move** — which is the
confirmation the re-housing is faithful, exactly as the integrator predicted a
same-order/same-rate decomposition should be.

## Byte-identity (the acceptance instrument)

All green **by name**, production config (biotic + erodibility + tectonic_history
+ full_agents all on):

- `contents_contract::generated_world_is_byte_identical_to_the_pre_contract_goldens` — **unmoved** (3/3 in suite).
- `providers_golden::the_production_world_still_hashes_to_the_pre_slice_goldens` — **unmoved** (2/2).
- `full_agents` 10/10 · `tectonic_history` · `deep_config_plumbing` 10/10 · `deeptime` 6/6 · `erodibility` 13/13 — green.
- lib unittests 85/85 (the new `passgraph` + `runner` graph/cadence/cycle tests, plus `pipeline`'s unchanged suite).

## Shape compliance

Instantiates three north-star pieces at the deep-time tier: the **pass-runner**,
**self-declaring passes**, and **order × rate** (material-behavior.md §5). The
`pipeline.rs` runner is **built on, not duplicated** (A-4 guarded — the topo-sort
is now one shared `passgraph` kernel). The crossing constraint is held. What this
movement does **not** touch, by scope: R/H stay separate planes, no weathering
rate/behavior change, no genesis passes.
