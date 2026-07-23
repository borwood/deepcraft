# 0078 — the seam in the tier that had no socket

*2026-07-23 · dc-worldgen / dc-core · one provider conversion + two doc riders ·
byte-identical*

> blogworthy (lens 3 — reflexions in a deepsim codebase): a provider mechanism
> that was only ever plumbed into *one* of the two tiers that need it, discovered
> the moment a seam in the other tier had to reach for it. The socket wasn't
> missing from the struct; it was missing from the code path.

## The batch, and the shape it rides

Three items on the proven journal/0060–0067 provider-seam pattern, acceptance
test byte-identity:

1. **Convert `paleo_temperature` [#11]** to a `Providers` slot.
2. **Declare `fits_in_pores` [S9]'s expected consumers** — a doc rider on an
   authority that is built, tested, and has zero callers (the inverse defect).
3. **Correct the `exhum`/`t_crust` comment [#28]** — an A-2 candidate.

All three ride **S-3** — *a summary is derived from the authority, never beside
it*. The seam names the question, the heir, and the identity value, so the
constant it holds can no longer masquerade as the rule. #28 is the A-2 axis
(*a justification outlives its premise*) — with a twist recorded below.

## `paleo_temperature`: the identity is the *wrong quantity*, deliberately

`geology.rs::deposit_deep_history` walks a column's recorded deep units bottom-up
and picks a member for each under the context it was **deposited** under. On two
adjacent lines:

```rust
let precip  = deep_precip(u.tag);   // reads the recorder's own aridity tag
let temp_c  = ctx.temp_c;           // reads TODAY's column climate
```

The aridity axis is genuinely at-deposition; the temperature axis is today's
climate stamped onto a Myr-old bed. Not a neutral stand-in — the *wrong
quantity*, and the asymmetry is visible in one glance because the honest sibling
sits right above it. That is exactly the doctrine's "a summary wearing an
authority's clothes": nothing fails a build because a deep unit was selected
under the wrong temperature; it just quietly picks a member calibrated to the
present.

The identity provider returns that same `ctx.temp_c`, so a default world is
bit-for-bit the pre-seam world. The heir is an epoch-indexed paleo-temperature
curve, and the payload hands it what it needs and nothing it can't fill (the
`wave_energy` mistake, journal/0060): the column position, the unit's
`chapter` — the tectonic epoch the record already carries — and the present-day
baseline the curve perturbs.

### Granularity: the call site lies, again

The naïve rule says *never call a provider inside a loop to answer something that
doesn't change in the loop*, and under the **identity** `ctx.temp_c` is constant
across a column's units — so the naïve reading says "materialize once per
column". But granularity follows the **heir** (journal/0060, 0061), and a
paleo-temperature curve answers *per epoch*. Each deep unit carries its own
`chapter`; the heir's answer varies per unit. Materialising once per column would
erase precisely the epoch axis the curve exists to express. So the slot is
**value-level, per recorded deep unit** — the same shape `burial_temp_c` took at
finalize, for the same reason. The identity's constancy is a property of the
stand-in, not of the question.

The cost is one always-`None` branch per deep unit on the collapse
(per-chunk-load) path — an `Option<fn>` null check, no allocation, no plane. This
is gen, and gen time is not the constraint; but even measured as a hot path it is
a predicted branch per unit, invisible.

## The finding: the collapse tier had no provider channel

The five existing slots are all consumed inside the **deep-time sim** — biotic,
erosion, finalize — which receives `Providers` through `DeepConfig`. But
`paleo_temperature` lives in the **collapse strata pass**, a different tier
entirely: the deep sim runs at pregen and writes the record; the `WorldGenerator`
reads that record back per column at chunk time and lays strata. Those two tiers
share a seed and the deep field, and **nothing else** — the `WorldGenerator` had
no `Providers` at all.

So the mechanism that exists to hold seams was only ever wired into one of the
two tiers that host them. This is the project's characteristic failure viewed
from the other side: not a mechanism reinvented next to itself, but a mechanism
*not extended* to the second place that needs it. The fix is small and additive —
`WorldGenerator` now carries a `providers: Providers` (default = all-identity,
resolved at world build like `DeepConfig`'s), threaded into `StrataCtx` (`Copy`,
one field), read at the seam. But the lesson is the one worth keeping: **a socket
in the struct is not a socket in the code path.** A future paleoclimate heir
resolves one `Providers` at world creation and hands the *same* set to both
tiers; today both are default, so both are byte-identical, and the two-tier
plumbing is in place for the day they aren't.

### Why the golden isn't enough, and the white-box test that is

`providers_golden.rs` proves an *absent* provider changes nothing — the
production world still hashes to `surface 0x176D40F11CCB006A` /
`record 0xC9C6D6F6E9089653`, the pre-slice goldens from `main`. But an absent
provider changing nothing is equally consistent with a slot that is **never
consulted** (the `burial_temp_c` falsifier's argument, one tier over). The line I
changed reads `ctx.providers.paleo_temperature(…)` now; if I'd fumbled the
substitution the golden would still pass trivially.

So there is a white-box test in `geology.rs` that drives `deposit_deep_history`
directly: with the identity set the deposited event records the present-day temp
(9.0); with a provider returning `present*100 + chapter` it records `902.0` —
proving the fn reads the provider **and** that both `present_temp_c` and the
epoch `chapter` reach it through the payload. Value-level, so per the doctrine the
world-level default byte-identity test (the golden) is the byte-identity proof;
the white-box test is the consultation proof the golden structurally cannot be.

## `fits_in_pores` [S9]: declaring the consumers of an authority with none

`dc-core`'s pore-packability rule is DECIDED, built, seven tests green, and
called by **nothing** — the inverse of a summary hardening into an authority:
here the authority exists and the *seam* is missing (`spines.md` § 3). The rider
names the two systems expected to call it — **hydrology** (groundwater
infiltration deposits fines only where they clear the pore throat) and
**diagenesis** (pore-filling cement / precipitated ore ride the same rule) — so
the next author of an infiltration path finds this rule instead of writing a
second, divergent grain-fits-throat test. This is the user's own point made
concrete: the stubbed API tells us now what an unbuilt hydro system must supply.
No behaviour change; the declaration *is* the work.

## `exhum`/`t_crust` [#28]: the A-2 that was already cured

The batch filed this as an A-2 — a comment claiming "the metamorphic-grade axes
the collapse tier **reads**" while nothing reads them. But the comment in the
tree already said "the collapse tier **WILL** read … currently consumed by
nothing", rewritten at commit `11d43859` (2026-07-21) — *before* the
2026-07-22 audit that flagged it. The audit quoted it with the "WILL" dropped,
which turned an honest future-tense note into a false present-tense claim on
paper only. So the A-2 wasn't live: prose that already says "consumed by nothing"
is not a justification outliving its premise. (Recorded as corrections #40 — the
falsified claim is the audit's characterisation, not the code.)

I tightened it anyway to fully earn its keep: it now cites `spines.md` § 3
(built-but-unconsumed) explicitly, states the no-consumer status as the load-
bearing fact, names the metamorphism heir and its now-existing arrival address
(the `burial_temp_c` geotherm, journal/0067), and keeps the careful distinction
the spines note already drew — `t_crust` *is* read inside the sim by `isostasy()`;
the unconsumed thing is the *exported plane*, not the value. The row stays in
§ 3: the comment correction does not consume it.

## What shipped

- New slot `providers::paleo_temperature` (paleoclimate group — first slot in a
  new group), payload `PaleoUnit`, identity `identity_paleo_temperature`.
- `Providers` gains the field, accessor, `Slot::PaleoTemperature`, and its arms
  in `ALL` / `name()` / `is_supplied()` / the none-path test.
- `WorldGenerator` and `StrataCtx` carry `Providers`; the seam is read at
  `deposit_deep_history`.
- Doc riders on `packing.rs` (S9 consumers) and `field.rs` (exhum/t_crust).
- Tests: 2 slot-identity unit tests, 1 white-box consultation test in
  `geology.rs`, 1 none-path arm, 3 in `providers_paleo_temperature.rs`.

Gates green, `cargo clean -p dc-worldgen -p dc-core --release` before the final
run. `material_properties` / `is_granular` left entirely untouched — they couple
to the block↔material collapse and are held for a design pass. No
NEEDS-RATIFICATION: a byte-identical slice carries none.
