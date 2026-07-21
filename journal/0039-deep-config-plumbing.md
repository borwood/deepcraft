# 0039 — Deep-config flag plumbing: opening the sealed gen path

The tectonic-history spike shipped (journal/0036), the full-agent roster
shipped (journal/0034), erodibility and biology were flipped on in production
(journal/0029, /0030). Every one of those lives behind a `DeepConfig` flag, and
`deeptime::field::production_config` decides which flags a real world is born
with. Two are still deliberately off there — `full_agents` and
`tectonic_history` — because turning them on is a terrain-reshaping,
world-appearance call the user has to *see* before ratifying.

And that was the problem. The user could not see them. There was no door.

## The sealed path

The client boots a world through exactly one call:

```rust
Pregen::run(WorldParams { seed, extent })
```

`WorldParams` carries a seed and a size, nothing else. Inside, the deep-time
pass calls `build_field(grid, seed)`, which calls `production_config(grid,
seed)`, which hard-wires `full_agents: false, tectonic_history: false`. There is
no parameter anywhere on that path that a launch flag could ride. To boot a
world with tectonics on, you would have had to edit `production_config` and
recompile — which is not a walk, it is a fork. The combined amplitude /
tectonic / full_agents walk the roadmap wanted was blocked on a plumbing slice.

## Why `WorldParams` did not grow a field

The obvious move is to add the flags to `WorldParams`. It is also the wrong
move. `WorldParams { seed, extent }` is a bare struct literal at roughly thirty
call sites — every deeptime test, every spike harness, every integration test
writes it out longhand. Adding a field breaks all of them at once, and worse, it
puts gen-time sim knobs into the *identity* of a world request, where the ~30
readers now have to think about them. The flags are not part of "which world";
they are a bounded override on "how deeply we simulate it."

So the override rides beside the params, not inside them. A new struct,
`DeepOverrides`, three `Option` fields:

```rust
pub struct DeepOverrides {
    pub tectonic_history: Option<bool>,
    pub full_agents:      Option<bool>,
    pub thickening_scale: Option<f64>,
}
```

`None` means *inherit the production default*. An all-`None` (`Default`)
`DeepOverrides` is therefore, by construction, a no-op: it produces a config
byte-identical to `production_config`. That is the whole safety argument —
existing worlds are reproduced exactly because the empty override changes
nothing.

The plumbing is a `_with` sibling at every layer, each one leaving the old
entry point untouched:

- `production_config_with(cells, seed, &overrides)` — starts from
  `production_config`, overwrites each `Some` flag.
- `build_field_with(cells, seed, &overrides)` — `build_field` through the
  `_with` config.
- `PregenCtx` gains a `deep_overrides` field; `deep_time_pass` reads it.
- `Pregen::run_with(params, &overrides)` — and `Pregen::run(params)` becomes a
  one-liner `run_with(params, &Default::default())`.

Not one existing `Pregen::run` or `WorldParams { .. }` call site changed. The
thirty literals stayed literal.

## The proof that matters

The load-bearing test is not "the flag works" — it is "the flag *off* is
invisible." Three layers of the same byte-identity claim:

- `production_config == production_config_with(.., &Default)` (compared through
  `Debug`, since `DeepConfig` has no `PartialEq` — a total field check).
- `build_field == build_field_with(.., &Default)` on every plane the world
  keeps: `surf`, `strata`, `recv/area/lake`, `exhum`, `t_crust`, chapters.
- `Pregen::run(p)` and `Pregen::run_with(p, &Default)` land on a byte-identical
  `DeepField` at the actual client seam.

Then the two "it genuinely bites" falsifiers: `tectonic_history: Some(true)`
populates the drainage export and chapter table that are empty on the default
path; `full_agents: Some(true)` perturbs the eroded surface. And the amplitude
knob, which only has meaning with tectonics on, moves the surface when dialed
from 40 to 320. Eight tests, all green.

## The client end

The flags reach the worldgen authority as one bundle, `GenOptions { extent,
deep }`, held as a Bevy resource so a key-2 scale switch rebuilds the world with
the same options rather than silently reverting to defaults. `main.rs` parses
four flags in the `--fullbright` / `--pack <v>` idiom already in the file:

- `--tectonics` → `tectonic_history: Some(true)`
- `--full-agents` → `full_agents: Some(true)`
- `--amplitude <n>` → `thickening_scale: Some(n)` (only bites with `--tectonics`)
- `--extent <small|medium|large>` → the world size (default medium)

A garbage `--amplitude` or `--extent` value prints a clear message and falls
back rather than aborting the boot — a walker fat-fingering a flag should get a
default world with a warning, not a crash.

One wrinkle worth recording: `Authority::new` is now only ever called from
tests (production boots through `new_with` to carry the `GenOptions`), and a
binary crate flags an otherwise-uncalled `pub fn` as dead code under
`-D warnings`. Gating it `#[cfg(test)]` is the honest fix — it says out loud
that the default-options constructor is a test convenience now, not a
production entry point.

> blogworthy: the override-beside-the-params pattern. When a config knob is a
> bounded deviation rather than part of a request's identity, threading it as a
> sibling `_with` channel with an all-inherit `Default` buys you a
> zero-churn, provably-invisible-when-empty extension — the thirty existing
> call sites never learn the knob exists.

## What it unblocks

The amplitude / tectonic / full_agents walk. A walker can now type

```
cargo run --release -p dc-client -- --fullbright --tectonics --full-agents --amplitude 160 --extent large
```

and stand in a world that was born with all of it on. The next slice
(erosion-supply calibration) also needed flagged worlds to render; it is
unblocked too.
