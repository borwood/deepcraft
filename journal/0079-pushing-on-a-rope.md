# 0079 — pushing on a rope: the erosion budget was never the amplitude lever

*2026-07-23. A measurement pass that started as walk prep and ended by retiring
the question it was meant to answer. No world changed; the finding is the
deliverable (design passes get journal entries too — user, 2026-07-22).*

> blogworthy (lens 2: procgen against priors; lens 4: respect for earth
> processes): **the flag that answered its own question by doing nothing.** We
> built `--erosion-budget` (journal/0076) specifically so a person could *see* a
> cranked-erosion world and rule on it. The faithful instrument said there is
> nothing to see — and the reason why is a real piece of geomorphology: a graded
> landscape does not care how fast you let it erode.

## The plan, and the null that stopped it

The session's ratified first move was the amplitude walk: boot the client world
at `--erosion-budget 3` and `10`, stand at a vista in each, and finally answer
journal/0029's standing "conservative amplitude" call from an eye instead of a
histogram. I built a tour-map probe to locate stations — the strongest hard-bed
contacts, the basement shield, the massif crest — and ran it.

The numbers stopped the walk before it started. Going 3× → 10× moved the surface
by a **mean of 0.2 m** across 44 000 land cells; relief was **1287 m at both**;
the single strongest hard-over-soft contact in the entire world gained **3.5 m**.
A walk of two visually identical worlds is not a walk.

## The instrument had to be proven before the null could be trusted

This is the journal/0030 trap in its purest form: a null from an instrument that
cannot see the question is worth nothing. And my first probe *had* a flaw — it
applied the budget by hand-multiplying a `DeepConfig` and calling
`deeptime::run_with(pregen, cfg)`, where the launch path is
`build_field_with(grid, seed, DeepOverrides{ erosion_budget })`. Different entry
points; the `run_with` path could have been riding on `pregen`'s already-1×-eroded
state, collapsing the 3×/10× difference into an artifact.

So the probe was rewritten onto the **exact launch path** and made to prove its
own faithfulness: the 1× field is asserted **byte-identical to the shipped
`build_field`**, or the probe panics. Then a sweep — 1× / 3× / 10× / 30× — so the
*shape* of the response is visible, not just a pair. Faithful result: relief flat
at 1287 m across the whole sweep, mean |Δ| vs 1× climbing only 0.00 → 0.10 → 0.26
→ 0.35 m, max |Δ| anywhere **3.5 m at 30×**, and lithologic separation
(fine 561 m, coarse 503 m, soil 668 m) that **does not widen one metre** from 1×
to 30×. The null was real, and now it was trustworthy.

## The mechanism: equilibrium, not starvation

A trustworthy null still needs a *why* — mechanism is a hypothesis until the
numbers name it. Two explanations fit a flat budget response: **(a)** the surface
is already graded to base level and the rate only sets the approach speed, or
**(b)** the surface is supply-limited — uplift dominates and there is almost
nothing to erode. They have opposite signatures under one measurement: total
lowering against the zero-erosion counterfactual (initial bedrock + all uplift).

| budget | mean lowering | max lowering | median | regolith H | mean surf |
|---|---|---|---|---|---|
| 1× | **41.1 m** | 1413.7 m | −9.6 m | 5.06 m | 517.5 m |
| 3× | 41.0 m | 1413.6 m | −9.7 m | 7.78 m | 517.5 m |
| 10× | 40.9 m | 1413.7 m | −9.8 m | 11.04 m | 517.7 m |
| 30× | 40.8 m | 1413.7 m | −9.9 m | 13.36 m | 517.8 m |

Decisively **(a)**. The landscape erodes a *lot* — mean 41 m removed, max **1.4 km**
— so it is emphatically not supply-limited. But that lowering is **flat across the
whole 1×→30× sweep** (if anything it dips slightly), which also rules out
iteration-starvation: too little time would make more rate erode more, and it does
not. The surface is graded to base level at 1× already; multiplying the rate
reaches the *same* equilibrium faster, not deeper. That is exactly the code:
incision is `inc_pot.min(room).min(max_inc)` where `max_inc = r − floor` caps a
cell at the material down to its base level, no matter how large `k_bedrock` grows
(`erosion.rs:1149`).

The one thing that *does* respond is regolith: weathering ×30 thickens the loose
mantle 5 → 13 m (`weathering × rate_mult × exp(−h/h_star)` — the cover taper is
self-limiting but not zero). Yet the surface does not lower with it (517.5 →
517.8) — the extra weathering builds a mantle transport won't remove, and it even
*armors* slightly (mean lowering 41.1 → 40.8). The median cell sits ~10 m *above*
its counterfactual: most ground gently aggrades while a minority incises to
kilometres, the classic erosional–depositional landscape at grade.

## Why this closes the question instead of moving it

The finding does not stand alone — it is the third leg of a tripod:

- **journal/0040** — tectonic amplitude (`--amplitude`/`thickening_scale`) bought
  no sub-km relief; the continent rose, the landforms did not.
- **journal/0041 / S13** — only 5 % of the roughness budget reaches the ground;
  the mountain-shape wavelengths (levels 1–4) are discarded at the L_DEEP
  override, and the 0040 summit is a *genuine simulated plateau*.
- **this** — erosion is equilibrium-limited, not rate-limited.

All three point at one thing: **the landscape's relief is bottlenecked on the
relief-*generating* side — the deep-field elevation structure and uplift — not on
erosion.** Cranking the erosion budget is pushing on a rope. The
`--erosion-budget` flag was worth building: it *answered* the amplitude call, and
the answer is "erosion is not where the drama comes from." It stays as a dev tool
(a faithful lever on regolith and on approach-to-grade), not as the amplitude
knob.

So journal/0029's cause 3 ("conservative amplitude") is **reframed, not retired**:
it was never an erosion-rate decision. It is a deep-field relief-generation
problem — the same one S13 left owed (the 100–500 m landform band that decays
away, the plateau that no amount of erosion can turn into a range). That is a
foundational design thread and gets its own pass; this entry's job was to prove
which lever it lives on.

## What it cost, and the reusable instruments

Two probes now live in the tree, faithful and committed:
`examples/amplitude_tour.rs` (the budget sweep + station map, with the
1×==shipped faithfulness assertion) and `examples/amplitude_mechanism.rs` (the
lowering-vs-counterfactual discriminator). The tour probe's stations were built
for a walk that is not happening; they remain valid the day a relief-generation
change *does* want walking.

The methodological note, again: the faithful instrument and the mechanism probe
together cost maybe twenty minutes of compute and saved a live co-walk of two
identical worlds followed by a wrong conclusion ("the budget does nothing, so the
model is broken"). The model is not broken. It is *finished* eroding, and that is
a different sentence with a different heir.
