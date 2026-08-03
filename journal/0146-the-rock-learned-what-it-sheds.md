# The rock learned what it sheds — and declined to say what it is

*2026-08-03 · FS-A, the release-spectrum slice (P10's ruled first build) · merge
`4635056` · rulings executed: U7/R2 ("authored edges"), U1 (grain is an axis), U4
(five grades), U3 deliberately unforced*

> blogworthy: **respect for earth processes** — the first content whose numbers came
> from published weathering-mantle studies rather than from tuning, authored as data
> the way the north star always said behavior would be; and **reflexions in a deepsim
> codebase** — a slice whose acceptance gate PASSED and whose writer was withheld
> anyway, because the number said yes and the meaning said no. Lens 1 rides along:
> the withholding was an agent's own call, escalated rather than shipped.

## What granite sheds

Until tonight, weathered granite produced "loose granite" — one product, no size, no
character. Real granite grus is one of the better-measured things in geomorphology:
coarse quartz-feldspar sand and grit dominating, a clay fraction from the feldspars,
and famously *little in between* — the silt trough, the same gravel–sand gap the
grain-size literature calls first-order. The release spectrum makes that authorable:

```
(granite, structure→loose) → 350‰ gravel · 450‰ sand · 50‰ silt · 150‰ clay
```

— a **declared edge product table on the material definition**, per the user's U7
ruling ("R2 it is — authored edges") and its generality rider: the *engine* type
admits any registered `MaterialId` as a product (mods may transmute), while
provenance-keeping — loose grades of the *source* identity, never anonymous sand —
is the default pack's authoring choice under U1. Eight source rocks got tables, each
carrying its literature band and confidence in a doc comment: grus studies for the
granite, Columbia River Basalt saprolite for the basalt's 600‰ clay, Pettijohn
arenite bounds for the sandstone. Measure-against-the-literature moved to authoring
time, which is where a content-first engine wants it.

Two shapes worth keeping. Shares are **integer per-mille**, so "sums to one" is an
equality the validator checks exactly, not a float tolerance — the Law-3 habit
applied to content. And a material with no spectrum takes the old single-product
path *by construction*: the vanilla registry collapses to a bit-identical edge, and
a test proves the facts unchanged to the bit. Zero goldens moved on a slice that
touched weathering.

## The writer that wasn't written

The plan said: weathering emits per the spectrum, and writes each product's grade
into the record's three grain bits (slice 3 had just reserved them, `GRAIN_UNSET`).
The U5 gate said: measure the grain split factor first — how much does the record
fragment when grade joins the merge key? Measured: **≤ 1.0329×**, comfortably under
the ~1.1 bar. The gate passed.

The agent building it then declined to write, and the reasoning survives contact:
on the deposition path the units being graded carry *drawn* member identities, so
grading a bed by its own material's release spectrum answers *"what would this rock
shed if weathered?"* — a fact about the sheet, derivable any time — not *"what grain
is this bed made of?"* — a fact about history. It is the slice-3 audit's rejected
O-1 (grade as a pure function of species) sneaking back in through the writer, a
constant wearing state's clothes. The honest writer is **propagated** grain: the
grade travels with the load through transport and is recorded where deposition
happens — which is exactly P10's transport slice, where the axis was always going
to earn its bits.

So FS-A shipped with every unit still honestly `GRAIN_UNSET`, a marked seam
(`grain_write_seam` + `set_grain`) one commit wide, and the count-model tripwire
still asserting that an unwritten axis splits nothing. The gate that passed and the
writer that waited are, together, the U5 ruling working: *gated on a measured split
factor* was never only about the number.

## The night around it

The slice merged through a live collision with the parallel bodies thread — their
arc gate landed while FS-A's branch carried band-aid fixes to the same files; theirs
won (their fixes were semantic, ours cosmetic), and the ordinal guard caught the
week's second journal-number collision at commit time. On the shipped world, with
the instrument flag on: 72,001 cells weather, **130,427 m of rock shed as 35 %
gravel / 45 % sand / 5 % silt / 15 % clay**, worst per-cell residual exactly zero.
The acceptance walk — a stripped upland against a distal basin, asking whether the
grades *read* — is owed, and it will collide immediately with the open
grade-legibility question the user has thoughts on. That is the right collision to
have next.
