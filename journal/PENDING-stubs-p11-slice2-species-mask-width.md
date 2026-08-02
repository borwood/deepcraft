# PENDING stubs.md entry — the deep species mask is one word wide

**Ordinal pending** (a parallel user session is live; the integrator assigns it at
merge). Opened by P11 slice 2's foundation,
`crates/dc-worldgen/src/deeptime/species.rs`.

## The stand-in

`SpeciesLayout`'s per-cell presence mask is a **single `u64`**, so the deep species
axis is capped at **64 materials** (`species::MAX_DEEP_SPECIES`, asserted at
`SpeciesAxis::new` with a message naming the fix). The axis is derived from the
registered content — every material a geology member deposits, plus the basement — so
a large enough content pack trips it.

## Why it is the shape it is

One word keeps `slot_of` to an AND, a popcount and an add, which is what makes a
sparse row affordable in the transport hot loop at all. Multi-word is a **mechanical
widening**, not a redesign: `n × ceil(W/64)` words, and the rank becomes the popcounts
of the preceding words plus the rank in this one.

## Blast radius, and why it is small today

**Not the binding constraint.** `EdgeId`'s mixed-radix packing already caps the
*material registry* at **51** (stubs #21, compile-asserted at `inventory.rs:250-255`),
which is stricter. So today the axis cannot reach 64 even in principle, and this entry
becomes live only after #21's heir (the interned per-world edge dictionary, S20 § 3.2)
lifts that ceiling.

## Heir

The same slice that lifts stubs #21. The two ceilings should move together, and this
one is the cheaper half.

## The honest failure mode

It is an `assert!` with a message that names the fix, not a silent truncation — a
world that would need a 65th deep species refuses to build rather than losing a rock.
