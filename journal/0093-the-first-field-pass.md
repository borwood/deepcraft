# 0093 — the first field pass: the geotherm and the `temperature` condition-field

> **WIP PLAN (committed early per resource rules; narrative fills in as the
> slice lands).**

## What this slice does

Stands up the **first §5 field pass** on the deep-time pass-runner: the
**geotherm**. It computes a per-cell **geothermal gradient** (°C/m) from the
tectonic state and plants it as the `temperature` **condition-field**
(`dc:field/temperature`, §14). Coal rank stops asking the degenerate
`burial_temp_c` provider (1 °C/m identity) and reads the geotherm's temperature
at the seam's burial depth instead. `COAL_ONSET_C` is recalibrated in the same
slice. **NOT byte-identical** — coal distribution moves by design.

## The build (plan)

1. **`deeptime/geotherm.rs`** — the field module:
   - `FIELD_TEMPERATURE = "dc:field/temperature"` (opaque field id).
   - `gradient_c_per_m(setting, crust_kind, t_crust) -> °C/m` — v1 **linear**
     gradient. Rifts/ridges/arcs steep (~42–50 °C/km), thick/cratonic crust
     shallow (~15–20), continental average ~22–27. Thickness modifier is a
     deviation from the crust kind's own baseline (thinned = hot, thickened =
     cold).
   - `temperature_c(surface_t, gradient, depth) = surface_t + gradient·depth`.
   - `march(grid, tec, chapter, cfg)` — writes `grid.geotherm` (the field-pass
     solve), reading `t_crust`, `crust_kind`, and the analytic tectonic setting
     (`tectonics::dominant_kind`).
   - `BurialColumn` (moved here from the retired provider) now carries the
     per-cell gradient.
2. **The pass on the runner** — `dc:deep/geotherm`, `reads {Climate, CrustThick}`,
   `writes {Geotherm}`, **low period** (heat flow evolves slowly). Present only
   when tectonic history is on (it needs the crust planes); seeded pre-loop,
   re-marched on its cadence.
3. **Subsume `burial_temp_c`** — retire the provider slot; `promote_coal` reads
   the geotherm temperature. Recalibrate `COAL_ONSET_C`.
4. Export `geotherm` on `DeepField` so `temperature` is a real read field
   (measurement + the future metamorphism heir).

## Scope held

NO formation-predicate evaluator / vocabulary registry, NO metamorphism, NO
nonlinear/mantle geotherm, NO capability tiering (north-star Deviation #2 — a mod
authors field passes exactly as the defaults do).

## Measurements (fill in)

- Temperature sanity: `(province, depth) → T` — TODO.
- The coal shift (walk reason): degenerate-stub coal vs geotherm coal — TODO.
- Recalibrated `COAL_ONSET_C` — TODO.
