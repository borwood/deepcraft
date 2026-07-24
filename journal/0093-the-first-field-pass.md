# 0093 — the first field pass: the geotherm and the `temperature` condition-field

The deep-time pass-runner (journal/0090) had, until now, only ever run one shape
of pass: **cellular** ones that move material — erosion, weathering, the biotic
layer. material-behavior.md §5 always named a second shape, the **field pass**:
a solve that reads material and **plants an environment field** the cellular
passes and the formation predicates read, running no edges of its own. §14
(DECIDED 2026-07-24) said the *first* one would be the **geotherm**, standing up
the `temperature` condition-field. This is that slice.

## What a geotherm is here, and why it is a field and not a provider

The sim has carried, since journal/0067, a provider slot named `burial_temp_c`:
"what temperature has this buried unit seen?" — asked once per candidate peat
unit at run finalize, answered by a **degenerate geotherm** (0 °C at the surface,
a gradient of exactly 1 °C/m, so "temperature" was numerically the overburden in
metres). That let `t >= COAL_ONSET_C` be bit-for-bit the pre-seam
`overburden >= 8 m` burial rule, which is exactly what a stand-in should do:
answer in the right *shape* while it waits for the real thing.

The real thing is a geotherm — `T(depth) = surface_T + gradient·depth`, the
gradient set by crustal heat flow. And the moment you try to land it as a
provider *heir*, it refuses the socket. A provider is a stateless
`fn(BuriedUnit) -> f64`: no captured state, no per-cell planes. But a real
gradient **is** a per-cell field — it varies with the crust under each column.
The value a provider can hold is a number; what the geotherm needs to hand over
is a field. So the honest home was never the provider set at all. `burial_temp_c`
**graduated out of `providers/`** and became the first §5 field pass. That is the
slot's own prediction coming true: *the distance between the question and today's
answer is the seam's most useful output* — and here the answer was "this is not a
provider; it is a field."

## The field pass on the runner

`geotherm::march` reads `t_crust`, `crust_kind`, and the current chapter's
analytic tectonic setting (`tectonics::dominant_kind`), and writes a per-cell
geothermal gradient (°C/m) into `grid.geotherm` — the `temperature`
condition-field, id `dc:field/temperature`. It **plants a field and runs no
edges**: it never touches `R`/`H`/the strata record, so it perturbs the erosion
result not at all; only coal rank (post-loop) reads what it writes.

On the runner it is a declared pass — `reads {Climate, CrustThick}`,
`writes {Geotherm}` — at a **low cadence** (`period = 40`, coarser than the
climate re-march's 20): heat flow evolves slower than the surface climate, which
is the RATE axis of the pass-runner (§5 "order × rate") made concrete for the
first time on a field pass. Like the climate march, a coarse-rate pass is seeded
before the loop and re-marched on its cadence. It is scheduled only on the
tectonic-history path, because it solves over the crustal columns, which exist
only there; off that path the field is empty and coal falls back to a continental
average. The topo-sort places it after `forcing` (which writes `CrustThick`) and,
having no in-epoch reader, it drops into the ready pool by id — between `frost`
and `transport`. A test pins its declaration and cadence; another pins its
absence off the tectonic path.

**No capability tiering** (north-star Deviation #2): the geotherm is authored
exactly as any other pass, and a mod would author a field pass the same way. One
authoring shape, no walls.

## The gradient model (v1 linear)

`gradient = f(tectonic heat flow)`. The primary signal is the tectonic setting:
ridges (~50 °C/km) and rifts (~45) and arcs (~42) run hot; a cold subducting slab
(trench, ~16) runs cold; a collisional welt (orogeny, ~25) is cool near-surface;
a plate interior sits at its crust kind's baseline (continental ~22, oceanic ~32,
margin ~26). A **crustal-thickness modifier** then reads `t_crust` as a deviation
from that crust kind's *own* seed thickness — thinned crust (a rift) runs hotter,
thickened crust (an orogenic root, an old shield) colder — which is what
separates a **rifted** continental cell (thin, hot) from a **cratonic** one
(thick, cold) though both are `Continental`. Bounded to 12–55 °C/km so the
self-reinforcing thickness feedback cannot run away. **v1 is linear in depth**;
nonlinear / mantle-heat is a ROADMAP followup, not built here. The rift-hotter-
than-craton ordering and T-rises-with-depth are pinned as pure-function tests.

## Subsuming `burial_temp_c`, and recalibrating the onset

Coal rank now reads the geotherm: `promote_coal` computes `surface_T +
gradient·depth` at each candidate's mid-slab burial depth and compares against
`COAL_ONSET_C`. The `burial_temp_c` provider slot is gone — field, accessor,
`Slot` variant, module, its golden falsifier — retired, not left dead beside its
heir.

The recalibration was the sharp part, and stub #14 wrote the warning in full: *a
real gradient against the old 8.0 onset turns the whole record to coal.* It does,
and the reason is geometry. This world buries peat only a few metres to ~100 m,
so the `gradient·depth` term is a fraction of a degree — while the *surface* term
(warm wet lowlands, where peat forms) is 20-something °C. Measured on the
production Small world, the candidate units' geotherm temperatures cluster
tightly at **~21–26 °C**; an onset left at 8 would promote every one of them. So
the onset is now a genuine temperature, calibrated to *this* record's
distribution (an Earth-true 50 °C would promote nothing at all here).

At **`COAL_ONSET_C = 25.0 °C`** the geotherm promotes **~23 %** of candidates on
Small, against the retired 8 m rule's **~12 %** — the same order, plausibly not
degenerate. What the geotherm *changes* is less the count than the **place**:
coal follows warm crust now — warm lowlands, and (where burial is deep enough for
the gradient to bite) steep-gradient rift/arc crust — instead of "wherever peat
happened to be buried deepest, regardless of climate." That relocation is the
reason to walk it (§14: "we measure the shift").

> **Honest caveat, filed as needs-measurement.** Because burial in this record is
> shallow, the *tectonic* half of the geotherm barely moves coal — the gradient
> contributes under a degree, so coal today is essentially thresholded surface
> temperature (warm-lowland swamps, which is geologically right for coal but is
> not the rift/arc signal the gradient encodes). The gradient's real payoff is
> the metamorphism slice: `exhum` (the P axis) over deep crust with the geotherm
> (the T axis) is where the tectonic contrast finally bites, because grade reads
> the deep column, not the shallow peat.

## The coal shift, measured (the walk reason)

Production Small, seed `0x…0059`, 1392 candidate units:

| rule | coal | of candidates |
|---|---|---|
| degenerate 8 m burial (retired) | 166 | 11.9 % |
| geotherm, onset 25 °C | ~320 | ~23 % |

Candidate temperature distribution (°C): p05 20.9, p50 22.4, p90 25.4, max 26.1.
The gradient field on this seed spans 44.6–51.6 °C/km — a hot, boundary-dense
crust (few cratonic-cold cells on a Small world), which is why the field-variation
test asserts a modest spread and leaves the full 12–55 range to the unit tests.

*(Numbers above confirmed against the onset-25 build; see the `geotherm` test's
`--nocapture` output.)*

## Goldens

They moved, by design, and only where coal reaches:

- **Surface goldens did not move** — the geotherm touches no surface plane
  (`surf`/`regolith`/drainage/`exhum`/`t_crust`), so `GOLDEN_SURFACE` is
  byte-identical. That is a clean partial proof the erosion result is untouched.
- **The record golden moved** (`GOLDEN_RECORD`, `providers_common`) — coal tags
  changed. Recaptured from the new world; the two suites that assert it
  (`providers_golden`, `providers_depth_to_water`) read the one constant.
- **The two Medium collapsed-world goldens moved** (`contents_contract`) —
  coal is diggable, so the collapsed blocks and material sidecar shift where coal
  did. The Small collapsed world's sampled columns hold no record, so it did not
  move (the standing control).

The moved values are recaptured from the new world; the surface staying put is
the independent check that this is a coal change and nothing more.

> blogworthy (lens 3, reflexions in a deepsim codebase): *the seam that told you
> it wasn't a seam.* `burial_temp_c` was filed as a provider for a year, with a
> careful argument for why a temperature (not an `is_coalified` predicate) was
> the right question. The argument held — but the *answer* turned out to be a
> field, not a value, so the heir couldn't fit the socket the question was filed
> in. The provider seam's own doctrine ("granularity follows the heir") is what
> made that legible: when the heir is a field solved over the crust, no
> `fn(unit)` can be its shape. The stand-in didn't just get replaced; it named
> the category error in filing it as a provider at all.
