# 0085 — Weathering wears the behavior shape (S16)

The north star names a destination: materials, their behavior, and the passes
over them, all authored in one uniform, self-declaring, compiler-validated
shape — a **pass** that declares its `{reads, writes}`, a **material behavior**
that is *pure*, and a **`ctx` that is a capability, not a god-object**. It is a
beautiful diagram. The question this spike existed to answer is the only one
that matters about a diagram: does a *real* deep-time behavior, over the *real*
height representation, actually fit inside it — and if it doesn't, *where* does
it tear?

We chose the subaerial bedrock→regolith weathering conversion as the subject.
It is the smallest complete behavior in the deep-time engine — one gate, one
rate, one transfer — and it is load-bearing: journal/0029 found it is the
*rate-limiting* phase on hillslopes, the term the whole landscape's lowering
collapses to. If the shape can hold weathering byte-for-byte, it can hold
combustion and diagenesis; if it can't hold weathering, the shape is wrong.

## The instrument, not the gate

The discipline here was to treat **byte-identity as an instrument**. The deep
goldens (`GOLDEN_SURFACE`, `GOLDEN_RECORD` — an FNV over the whole production
world's surface planes and strata record) are a bit-exact fingerprint of a
four-flags-on deep-time run. If the reformulated weathering phase leaves them
*unmoved*, the new shape is a clean *view* over the current representation,
inventing nothing. If it *can't* leave them unmoved without adding state the
deep sim doesn't track, that's not a failure to hide — it's the finding, and it
would prove the height representation is lossy about *form*.

So the rule was: preserve the exact f64 grouping (multiplication is not
associative — the same discipline the provider conversions hold), and if the
abstraction forces a different grouping, *report* it rather than fake it.

## What fit cleanly — the transfer is a view

The core mapped perfectly. Weathering's whole effect is
`R -= wth; H += wth; dH += wth` — bedrock converts to regolith in place,
mass-neutral in the column, with the strata recorder's net-ΔH accumulator
growing because the record mirrors `H`. That *is*
`Transform::form_change(Structural → Loose)`, applied by a pass-owned
`WeatherApply::apply(transform, rate)` that scales by `rate·dt` (`dt = 1` at the
deep tier, and `rate × 1.0 == rate` for finite f64, so the scaling is
byte-transparent). No new plane. No new per-cell field. The `WeatherCtx` copies
`R, H` by value *before* the write, so the gate (`R + H > sea`) and the cover
taper (`exp(-H/H*)`) read exactly the pre-update state the old kernel read. The
goldens did not move. The transfer is a view.

And the capability discipline turned out to be *structural*, not aspirational.
A behavior holds a `&WeatherCtx`, which exposes no write path at all; the only
writer is `WeatherApply`, which the pass builds and never hands over. Purity
isn't a convention a behavior is trusted to honor — it's the only thing the type
it can name permits. That is the north star's "the ctx is a capability" cashed
out in the borrow checker.

## Where it tore — and the tear is the map

Two places strained, and both are the *same* underlying fact: **the height tier
has no per-cell material.** `R` is basement everywhere; the strata record is a
stack of `H`.

First, the **rate is a share-weighted blend**, not a material property. The
production weathering rate at a cell is
`blend_susceptibility(window_shares, table)` — a dot product over the several
lithologies filling the near-surface window (journal/0072's cure for the S-4
plurality-flip step). There is no single "outcropping material" to read a
`weatherability` field off. So the north star's `for m in
ctx.materials_with(Weather)` loop **degenerates to one synthetic body**, and
`self.weatherability` cannot be the byte-identical source. To stay bit-exact,
the adapter delivers `weatherability()` as the *blend*, through the ctx.

Second, **weathering is a sum over agents, and the two-factor sketch has one
slot.** The north star writes `base × (biotic × weatherability) × taper`. But
production already runs *two* agents attacking the rock in place: mechanical
abrasion (the `weatherability` stand-in) and the periglacial **frost** agent
(journal/0034). Byte-identity forced the shape to grow a third factor —
`base × ((biotic × weatherability) × frost) × taper` — threaded exactly where
the old kernel's `(wmult × sus) × frost` sat. That is the honest shape of the
physics (`lithology.rs`: "in-place weathering is the sum of every agent's attack
on rock that has not moved yet"), and it means the behavior interface must one
day express *a reduction over an agent set*, not a fixed product — the very door
the karst/dissolution agent will walk through.

Neither tear required inventing deep state to *fake* the identity. So the
verdict is the clean one: the height representation is a faithful view for the
weathering **transfer**, and lossy about material **identity** — which is
precisely the deep-cell material-inventory question already filed as coupled to
Crux 1. The spike didn't discover a new problem; it *localized* the known one to
one ctx method (`weatherability()`) and one missing capability
(`materials_with`).

## The material property, and the trap we didn't fall into

We did add `weatherability` to `MaterialProps` in dc-core — the property the
north-star behavior *wants* to read. The temptation was to make it a restatement
of mechanical extraction resistance, and that would have been the exact
one-number-erodibility trap `lithology.rs` was written to avoid: a basalt is
mechanically tough (hard to *dig*) yet chemically rots to clay readily, so
competence and weatherability are independent, the same way solubility is. So
`weatherability` is its own axis, ordered soft→hard with the reference clastic
pinned at exactly `1.0`, and — because the deep-time rate is still driven by the
abrasion proxy until the cell inventory lands — pinned by test to *agree in
ordering* with that proxy. It is the authority; the deep blend is its summary
(the "a summary is not an authority" convention, made a test).

> blogworthy (lenses: deepsim-reflexions; AI-native development): byte-identity
> as a *diagnostic instrument* rather than a regression gate — the reformulation
> that passes tells you the abstraction is a clean view; the reformulation that
> *can't* pass without new state tells you exactly which representational
> upgrade the design still owes. We ran the same bit-fingerprint two ways and
> read the tear, not just the pass.

## North-star convergence

This converges the deep-time weathering phase onto the ratified
Pass/Material/ctx/Transform shape byte-for-byte, proving `form_change` is a clean
view over the height stocks while pinpointing the single remaining seam — a
per-cell material inventory (Crux 1) — that a full material-behavior world needs.
Keepable shape; one seam left open, and now named.

Numbers and evidence: `docs/spikes/S16-weathering-behavior-shape-results.md`.
