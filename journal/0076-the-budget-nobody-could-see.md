# 0076 — The erosion budget nobody could see

> blogworthy (lens 1: AI-native development; lens 2: procgen against priors):
> the decision that was unmakeable for two days because nobody could *look* at
> it. The lever existed as a knob, the physics existed in a probe, the answer
> was measured — and none of that is the same thing as a person standing in the
> world and judging it.

## The stuck decision

Since journal/0029 the erodibility coupling has carried a standing question, and
it has never moved. The coupling modulates erosion rates per cell by the
resistance of the rock outcropping there — soft strips fast, hard stands proud —
and at the *shipped* erosion rate that contrast is real but modest: relief
+2 m aggregate, a hard bed ~0.5 pp steeper. The reason is not the model. It is
the budget. The whole landscape only removes a few metres of material against
hundreds of metres of tectonic uplift, so there is almost nothing for the
differential to differentiate. `erodibility_probe` experiment B proved this the
honest way: raise *every* erosion rate 10× together — weathering, `k_transport`,
`k_bedrock`, so the **relative** rates never move — and the same coupling cuts
20 m of relief with a hard bed standing **44.7 m** proud. A correct model
waiting on an amplitude decision, not a broken one (ROADMAP § the erodibility
rider, cause 3, "conservative amplitude").

So the call was clear: the user should raise the global erosion budget, or the
contrast, or both. And it sat there for days. Not because it was hard — because
it was **unmakeable**. The headroom world existed only inside a Rust example's
`experiment("B — HEADROOM", ...)` call. It printed histograms to a terminal. No
launch path booted it; you could not walk it, could not stand at a vista and see
whether 10× erosion carves a landscape you *want* or a moonscape you don't. The
user said it plainly on 2026-07-22: *they have never SEEN a cranked erosion
budget.* An appearance decision you cannot put in front of an eye is not a
decision anyone should make from a number.

## The term collision, first

Before the flag could be named, the corpus had to disambiguate a word it had
already tripped over. journal/0040 — "The amplitude that changed nothing" — was
about **tectonic** amplitude: `thickening_scale`, the `--amplitude` launch flag,
the m/iter the orogenic forcing multiplies. It lifts the continent; corrections
#23 measured it to buy no sub-km relief either way. That is a *different lever*
from this one. This is **erosion** amplitude — how hard the surface is allowed to
cut, not how hard the crust is pushed up. Two amplitudes, opposite ends of the
same landscape. So the flag is `--erosion-budget`, never `--amplitude`, and the
`DeepOverrides` field is `erosion_budget`, never anything with "amplitude" in it.
The ROADMAP rider called this collision out in advance; naming around it was the
first requirement, not an afterthought.

## The slice: one more Option on the door 0039 already built

journal/0039 built the sealed-path door — `DeepOverrides`, a struct of
`Option`s threaded `production_config_with` → `build_field_with` →
`Pregen::run_with`, where every `None` inherits the production default and an
all-`None` override is byte-identical to the pre-plumbing boot. `--tectonics`,
`--full-agents`, `--amplitude` all ride it. The erosion budget is simply one more
field on that door:

```rust
pub struct DeepOverrides {
    pub tectonic_history: Option<bool>,
    pub full_agents:      Option<bool>,
    pub thickening_scale: Option<f64>,   // TECTONIC amplitude
    pub erosion_budget:   Option<f64>,   // EROSION amplitude — this slice
}
```

`production_config_with` applies it exactly as experiment B does — the three
rates multiplied *together* so the relative rates the erodibility coupling reads
never move:

```rust
if let Some(mult) = overrides.erosion_budget {
    cfg.weathering  *= mult;
    cfg.k_transport *= mult;
    cfg.k_bedrock   *= mult;
}
```

No new struct, no new seam, no `WorldParams` growth. The "record the multiplier
in the world identity" requirement is met by the same precedent `--amplitude`
set: the `DeepOverrides` field *is* the identity record — it flows into
`DeepConfig`, which is what determines the world. There is no separate manifest
to update because the config is the manifest.

## The falsifier that makes it safe

The load-bearing invariant of the whole door is that its off-state changes
nothing. For a *multiplier* flag that has a sharp, provable form: `x * 1.0 == x`
exactly for f64, so `erosion_budget: Some(1.0)` must produce a `DeepField`
byte-identical to the no-flag boot — every plane the world keeps. That is
`erosion_budget_one_is_byte_identical_to_no_flag`, and it is the guard that keeps
a `--erosion-budget 1.0` launch (and every world already on disk) reproducible.
Its sibling `erosion_budget_ten_reaches_the_run` proves the plumbing is *live* —
a 10× budget cuts a measurably different surface — so the identity test is
guarding a real channel, not a no-op that only *looks* safe because every other
test happens to pass it `1.0`.

Goldens do not move: default is off, off is identity. This is plumbing, not
behaviour. The behaviour is the walk, and the walk is now possible.

## What it looks like in game

Launch the pair on one seed:

```
cargo run --release -p dc-client -- --erosion-budget 3
cargo run --release -p dc-client -- --erosion-budget 10
```

Same world, same tectonics, same everything — but the surface has been allowed to
cut 3× and 10× as hard. Stand at a vista in each and the question the corpus has
carried since 0029 finally has an eye on it: does a cranked erosion budget carve
the differentiated, benched, hard-bed-proud landscape the probe promised — or too
much of it? That judgement was measured two days ago. Now it can be *seen*.
