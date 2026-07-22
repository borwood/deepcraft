# 0067 — A degenerate geotherm is still a geotherm

*2026-07-22. The coal calibration was accepted on condition that it be a seam.
This is that seam — and the interesting decision inside it is what the slot is
allowed to ask for.*

> blogworthy: **"seam the quantity, not the verdict."** The cheapest socket for
> a threshold is a boolean: `is_coalified(unit)`. It is also the one that has to
> be replaced when the next threshold on the same physics shows up. The
> expensive part of this slice was not the code; it was arguing that the slot
> should ask for a *temperature the project does not have*, and then writing
> down, in the identity, exactly how it is faking one.

## The condition, and what it actually asked for

journal/0066 moved coalification onto the right axis: peat becomes coal when its
own **overburden** crosses a threshold, not when the seam is thick. The number
that came with it, `COAL_BURIAL_M = 8.0`, is not physical and 0066 said so at
length. Earth wants 10²–10³ m of section for peat→lignite; our record has
thirteen peat-derived units in the entire world under 50 m and one under 100 m.
An Earth-honest threshold gives a world with no coal in it. So 8 m is the ~90th
percentile of *this* record's burial distribution, under a statement of shape
rather than of target: *coal is what happens to the peat that got buried
deepest.*

The user accepted that conditionally, and the condition is worth quoting because
it is a design instruction and not a caveat:

> *"i can accept the coal etc as long as uses geotherm seam with heir etc. we
> knew coal/charcoal would depend on eco + geotherm and need to accept just
> buildin the seams in for now. the calibration is fine, we aren't answering
> deep questions about it right now."*

That is S-5 stated as an acceptance test. The number may stay wrong; what may
not stay is the *shape* — a tuned constant sitting in an `if`, with the system
that should own it named only in prose. The deliverable is that landing a
geotherm becomes **supplying a provider** rather than rewriting `promote_coal`.

## The decision: what question does the slot ask?

Two shapes were available, and the brief explicitly left the choice open.

**The predicate.** `is_coalified(BuriedUnit) -> bool`. One call, one answer, no
units to argue about; the identity is `overburden_m >= 8.0`, transparently the
shipped rule. It is the smaller change and the smaller docstring.

**The quantity.** `burial_temp_c(BuriedUnit) -> f64` — *what temperature has
this unit seen?* — with coalification as a threshold on the answer.

I went with the quantity, and the argument is `spines.md` S-8, *one quantity,
many regimes*. Coalification is not a phenomenon with its own control. It is one
rung of a thermal-maturity ladder that does not stop there: peat → lignite →
sub-bituminous → bituminous → anthracite → graphite, and straight on into
metamorphic grade, which `stubs.md` § 4 has already filed and which the
`exhum` / `t_crust` planes — populated since U8, **read by nothing**, listed in
`spines.md` § 3 as machinery with no consumer — are waiting to drive.

Under the predicate shape, landing a geotherm means supplying a *coal* provider.
Then a *rank* provider. Then a *grade* provider. Each one re-derives the same
temperature behind its own signature, and the fact that they are one physical
quantity is expressible nowhere. Under the quantity shape, the geotherm is
supplied once and every rung of the ladder becomes a constant compared against
it. That is the same reasoning water.md used to refuse "a free-water system and
a bound-water system with a coupling layer" in favour of one quantity in two
regimes — and refusing the coupling layer is the entire point of S-8.

There is a second, smaller argument. The predicate's payload would have had to
carry the unit's *facies* (you cannot answer "is this coal?" without knowing it
is peat), which quietly welds the slot to the one consumer it was built for. The
temperature slot's payload carries no facies at all, because a temperature that
depended on whether the rock was peat would not be a geotherm.

## The price, paid in the open: the identity has to fake a unit

Here is the awkward part, and I want it on the record rather than smoothed over.

The slot answers in °C. The project has **no geotherm**; the only temperature
anywhere in the sim is `climate::air_temp_c`, surface air. So the identity has
to produce degrees out of the only thing it knows, which is metres. It does the
crudest possible thing:

```rust
pub fn identity_burial_temp_c(unit: BuriedUnit) -> f64 {
    unit.overburden_m
}
```

That is a **degenerate geotherm**: 0 °C at the surface, a gradient of exactly
1 °C/m. Forty times Earth's. It is not a claim about this planet — it is the
arithmetic that makes the byte-identity a *proof* instead of a hope. The
requirement is that `t >= COAL_ONSET_C` be bit-for-bit the pre-seam
`overburden_m >= COAL_BURIAL_M`, and `x >= c` survives composition with `f` only
when `f` is the identity. Put a real 0.025 °C/m gradient and a surface datum in
there and you have inserted two rounding steps between the two comparisons; on
the boundary units — of which a record with 35 382 peat-derived units will have
some — they can disagree, and the golden would have caught it as a mystery
rather than as a decision.

So the units are wrong on purpose, loudly, in the type. This is the same
deliberate mismatch `depth_to_water` already carries: that slot is named for
metres below the surface and its identity answers a dimensionless 0..1 wetness
index, and its docstring calls the mismatch *"the seam's most useful output"*.
The rule those two now jointly establish: **the slot is named for the question,
not for what today's answer happens to be**, and the distance between them is
information you want visible rather than hidden behind a plausible number.

### The landmine that shape creates, and how it is pinned

A temperature slot with a fake temperature scale has an obvious failure mode,
and it is severe: an heir lands a real geotherm, `COAL_ONSET_C` is still `8.0`,
and now every unit in the record is 300 m down at 30 °C and **the whole
sedimentary pile is coal**. Not a drift — a world-scale defect on the first run
after the heir arrives.

`COAL_ONSET_C` is therefore defined as `= COAL_BURIAL_M`, not as a second
literal `8.0`, and both constants carry the same sentence: *this constant and
the identity geotherm are one calibration in two places and must retire
together.* A unit test compares thresholding the identity against thresholding
the burial depth across the boundary (`7.999_999_999`, `8.0`, `8.000_000_001`),
so the day someone changes one and not the other, the failure is a red test
naming both rather than a world made of coal.

## Where the slot is filed, and why there is no `diagenesis` group

`providers/mod.rs` groups slots by **owing system** — hydrology, ecology,
materials, structural — the same four buckets the 34-seam inventory uses, and
the grouping is load-bearing twice over: it is a map of who owes what, and it is
what lets two concurrent conversions insert at two different points and merge
without a human.

The brief offered *materials* or a new *diagenesis* group. It is neither: the
slot is filed under **structural**. The rule is that a slot goes under the
system that will **answer**, not the one that asks — `parent_p` sits under
materials though ecology consumes it. Diagenesis is the asker here. A geotherm
is a property of the crust, and crustal heat flow is tectonics' to describe; the
heir's input is `t_crust`, a tectonics plane. A *diagenesis* bucket would have
been a group named after a consumer, which is exactly the distinction the four
buckets exist to hold — and it would have been a fifth bucket that no other seam
in the inventory could ever join.

## Granularity, and the one rule that had nothing to say

The standing rule is that a provider must never be called inside a hot loop to
answer a question that does not change inside that loop, sharpened by 0060 into
**granularity follows the heir, not the call site**.

This slot is **value-level, per candidate unit** — and for once neither half of
the rule is under strain. `BioticSim::finalize` runs **once, at the end of the
whole run**; there is no loop to be hot. And a plane is structurally impossible:
the question is asked of a *unit*, not of a cell, and a column has as many units
as its history had events. The provider is asked only about candidates — non-top
peat — which is unobservable (the slot is a pure `fn` of its payload, so an
unasked question has no answer that could have differed) and keeps the call
count proportional to the peat rather than to the record.

The payload assembly is the only place the call site pushed back. A geotherm's
upper boundary condition is the surface temperature, which the record does not
know — so `finalize` computes it per column from the sim's one climate model and
hands it over. That meant lifting the row latitudes out of the loop first, since
`lat_deg` takes `&grid` while the loop holds `&mut grid.strata`: one `w`-long
vector, rather than materializing an `n`-long temperature plane the identity
would never read.

## Two things I filed instead of fixing

**Overburden is depth below the *present* surface.** The record keeps no memory
of section that was deposited and later stripped, so a unit that was once deeply
buried and then exhumed reads as shallow — and coal rank is *irreversible*. Real
coalification is a ratchet on maximum burial; ours is a function of current
burial. That is a genuine wrong answer, not merely a coarse one, and it lives in
the payload's docstring where an heir will read it. Fixing it needs the record to
carry a high-water mark, which is state, which is a different slice.

**No time axis.** Rank is time-at-temperature (Lopatin's TTI, vitrinite
reflectance), not peak temperature. `BuriedUnit` deliberately carries no age
field: the record stores a tectonic chapter, not a duration, so an age field
today would be a payload the heir cannot fill — which is precisely how
`wave_energy` acquired a payload struct its own heir cannot fill (0060). When
there is a duration, adding the field is one line and a compile error at one
call site.

## What it cost

Nothing, which is the point. `GOLDEN_RECORD` and `GOLDEN_SURFACE` both pass
unchanged with the default provider set; the identity path is the pre-seam
expression, bit for bit. The falsifiers that make that mean something are the
two new ones in `providers_burial_temp_c.rs`: a **frozen** geotherm
(`NEG_INFINITY`) leaves the production world with *zero* coal and the peat count
up by exactly the coal count the identity world had — promotion is a retagging,
so the totals must conserve — and a **molten** one (`INFINITY`) promotes every
buried peat while leaving every column's topmost unit peat. That last assertion
is the one worth having: "the living surface is never coal" is written out in
`promote_coal` as a structural guard rather than implied by a positive
threshold, and an infinite geotherm is the only configuration in which the
difference is observable.

A golden that passes proves the absent provider changes nothing, which is
equally consistent with a slot nobody ever consults. The frozen geotherm is what
proves it is consulted.
