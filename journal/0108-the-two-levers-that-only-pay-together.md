# 0108 — the two levers that only pay together

*2026-07-25.*

S20 measured the fact ledger and found a gigabyte waiting at the end of the
per-depth road: 17.45 MiB today, **973.40 MiB** once weathering runs at every
depth instead of only at the bedrock seam. The user read the four options and
picked **3 + 2c** — resident fold + paged facts, *and* the compact 8-byte fact.
This slice is the **2c half only**. The pager is the reserved continuation and
this slice deliberately builds none of it.

The whole content of 2c fits in one sentence: **`Fact::InPlace` goes from 16
bytes to 8, and nothing is deleted.** What makes it worth an entry is that
the sentence is only true because of a measurement that says the obvious
version of it does not work.

> blogworthy: **reflexions in a deepsim codebase** (lens 3) — the shape of the
> finding is "two optimizations that are each worthless alone and worth 2× in
> company", which is a thing alignment padding does to you and a thing you will
> re-derive wrongly every time unless the *type system* keeps saying it.
> Also **AI-native development** (lens 1): the spike published nine test sites
> that would need re-derived tolerances. One did. The over-prediction was not
> sloppiness — it was a verdict stated without its condition, and the condition
> was named two subsections later in the same document.

---

## The fact, before

```rust
pub enum Fact {
    InPlace {
        chapter: u8,
        cause: u8,          // the agent
        from: (u8, u8),     // (material, form)
        to: (u8, u8),
        fraction_m: f64,
    },
}
```

Six bytes of payload, two of padding, eight of `f64`. **16 B.** Multiply by the
61 460 352 facts a per-depth pass would commit and you have 937.81 MiB of
payload — **96.3 % of the projected residency**. The CSR index is not worth
attacking; the fact width is the whole game.

## Lever one, alone: worth nothing

The four endpoint bytes are the obvious target. There are 26 materials and 5
forms; an endpoint is one byte of mixed radix and an edge is two. Replace
`from`/`to` with a `u16` and you have reclaimed four bytes.

You have reclaimed nothing. `fraction_m: f64` wants 8-byte alignment, so the
struct rounds back up to 16 and the compiler quietly spends the four bytes you
saved on padding. S20 § 3.1 measured it rather than reasoned it, and this slice
kept the finding alive **as an assertion about a shipped type** rather than a
probe's local struct:

```
Fact<FracM>   — the gen-time accumulator, WITH the edge id — is 16 B
Fact          — the resident fact, edge id AND f32       — is  8 B
```

The accumulator is not a museum piece kept around to prove a point; it exists
for a reason given below. But it happens to *be* the edge-id-only shape, which
means the "an edge id alone saves nothing" finding is now checked by the
compiler on every build, against types the game actually uses.

## Lever two, alone: worth half

Narrow `fraction_m` to `f32` and keep the endpoints: 6 bytes of payload align to
4, so the struct is **12 B**. One word back, not two.

## Together: 8 B, zero padding

`u8 + u8 + u16 + f32` = 8 bytes with nothing wasted. The two levers are a
**single combined decision**, and "cheap encoding wins" is not two independent
wins you can take one at a time.

The gate asserts this as a *bound*, not a snapshot — each side equal to the sum
of its own fields, so what is checked is "no padding", not a remembered byte
count:

```rust
let payload_8 = size_of::<u8>() + size_of::<Cause>()
              + size_of::<EdgeId>() + size_of::<StoredFrac>();
assert_eq!(size_of::<Fact>(), payload_8);
```

## What the edge id is actually for

Bytes are the consequence. The reason to want an `EdgeId` is that
**`EdgeId::declared` is its only constructor**, and it returns `None` unless the
transition is one material-behavior.md §3 declares — 5 forms → 20 directed form
edges, plus the same-form material-change class; the null edge (`from == to`)
and `Void → Void` refused. A `Fact` can only be built from an `EdgeId`.
Therefore:

> **a fact structurally cannot name a transition the graph does not declare.**

Before this slice a fact carried four free bytes and could name any pair
whatsoever, including a null edge that moves nothing and would sit in the record
as an event that did not happen. That is S-8 with the transitions finally
*declared* instead of merely conceptual, and it is S-6's rule ("declared
relations, never incidental") applied to a data axis rather than to pass order.

The strong form matters and is asserted: `InvCtx::apply_edge` on an undeclared
edge **does not run at all**. Refusing to *record* while letting the move happen
would leave the inventory holding a change with no provenance — the worst of
both. It returns 0 and touches nothing
(`an_undeclared_edge_does_not_run_at_all_not_merely_goes_unrecorded`).

## Where the narrowing happens, and why that is the whole design

`f32` for a physical quantity invites the obvious objection, and the obvious
objection is about **accumulation**: round every add and the error compounds
with the number of adds. The weathering pass fires every epoch for 200 epochs;
a naively narrowed accumulator would round its running total 200 times.

So the narrowing happens **exactly once, at persist** — and the type system says
so rather than a comment:

| alias | `Q` | width | where it lives |
|---|---|---|---|
| `Fact<FracM>` | f64 | 16 B | `FactLedger` — the gen-time accumulator |
| `Fact` (default) | `StoredFrac` = f32 | **8 B** | `LedgerField` — the resident record |

`Fact<Q>` is generic over the width its fraction is stored at, with exactly two
inhabitants. The accumulator's `*q += share` never rounds. `Fact::narrowed()` is
the single `as f32` in the tree and `LedgerField::from_accumulators` is its only
caller, so *"where does precision get lost?"* has one grep answer. Every read
widens back (`fraction_m() -> FracM`) and every sum stays `f64`.

The measured consequence, from S20 § 4.1 on the production world: max fold error
**1.2305e-7 m**, max relative **5.766e-8** — which sits *at* f32's own 2^-24
resolution of 5.960e-8. That equality is the point: it is the signature of a
**single** rounding, not an accumulating one. Against one eighth of a voxel
(0.1125 m), the only quantity that can change what a player sees, that is **one
part in 914 000**.

The probe measures the counterfactual on the same shares, so the claim is a
number rather than an argument: narrow-once versus narrow-at-every-add, at 1,
10 and 200 firings.

## Nothing is deleted

S20's option 4 family bought more bytes by dropping an axis — chapter (−731
MiB), agent (−625 MiB), or both. Those are the options that break something.
Drop `chapter` and the derived story layer §1 specifies has no field state to
look up; drop `cause` and the DECIDED of *one day earlier* (sum, not product,
*because* only a sum makes each agent's share well-defined) becomes unobservable
in the record. A per-slot scalar with a fact's paperwork is **A-1** exactly.

2c keeps **all four axes** — when, who, what edge, how much — and narrows one
representation. `every_axis_survives_the_narrowing` walks a committed cell
before and after the persist step and asserts chapter, cause, edge id and both
endpoints equal across it. This is the version of "make it smaller" that S-9
permits: the sparse overlay still carries every fact a derivation cannot
predict, because the facts *are not re-derivable* (material-behavior.md §1) and
deleting them is irreversible.

## The dictionary, and the stub it makes loud

An `EdgeId` is positional in the material registry. So were the four endpoint
bytes it replaces — the pre-slice fact stored `MaterialId`, a positional
`MatRepr` under a `MATERIAL_COUNT == 26` compile-time assert. **The id inherits
the exposure; it does not create it.** Nothing in the tree serializes a ledger
today, so any id stable within one compiled binary suffices *now*.

But "ready-made worlds are the sanctioned answer" means a ledger will be
persisted, and a positional id read back against a changed registry is
*silently reinterpreted*: granite's facts become diorite's and every test still
passes. So the slice ships **`EdgeDict`** — `id → (from-material-NAME,
from-form, to-material-NAME, to-form)` — with `validate()`, which re-resolves
every row's id from its own *names* against the live registry. A registry change
becomes **detected** instead of silent.

Two things about it are deliberate:

- **It is derived, never stored.** `EdgeDict::of_facts` scans. A dictionary
  carried as a field beside the facts is a second copy of what the facts already
  say, and a second copy can disagree — the A-1 discipline applied to the
  guard rather than to the thing guarded. It is wanted only at persist, which is
  not a clock anything here spends.
- **On the shipped world it has one entry.** Every weathering fact in 297 025
  deep cells is `dc:granite/structure → dc:granite/loose`. That is not a
  weakness of the dictionary; it is stub #16 (bedrock is a flat granite seam)
  showing through, and the dictionary is the instrument that will make it
  obvious when that stub is discharged.

The mixed-radix packing caps the registry at `256 / 5 = 51` materials. That is
**stub #21**, and it fails at *compile* time with a message naming the entry —
a stand-in that cannot become the definition without a build error. Its heir is
the **interned** id (indexing the dictionary rather than encoding positions),
which the pager slice has to build anyway.

## The correction: nine predicted, one moved

S20 § 4.2 classified every stored-value assertion in the tree and published a
table of **nine sites** that would fail under f32 storage. **One did.**

The table was computed against a model where `Fact` stores `f32` *everywhere*.
The design that shipped narrows only at persist, so the eight sites that read a
`FactLedger` are reading the **f64 accumulator** — `f64` vs `f64`, no
representation change at all, and their original `1e-12` is still the honest
bound. They pass untouched.

The one that moved is `weather_inventory.rs`'s
`bedrock_facts_key_stably_as_the_record_grows`, which compares a band summed
from the accumulator against the same band summed from the *finalized* record —
the only assertion in the tree that straddles the persist boundary. It is
**exactly** the site S20 § 4.3's final paragraph predicted, moving in exactly
the direction it predicted (Class A → Class B).

So the mechanism the spike named was right and its table was conditional
without saying so. Corrections #53.

### The replacement bound is derived, not nudged

journal/0106 refused "loosen the tolerance until it goes green" as an option,
and this is not that. The new bound is a published formula:

```
|a − b|  ≤  FOLD_DEPTH_HEADROOM · 2^-24 · max(|a|,|b|)  +  NEAR_ZERO_FLOOR
```

- `2^-24` is **f32's own stated resolution** — the most a single
  round-to-nearest can move a value, relative to the value.
- `FOLD_DEPTH_HEADROOM = 8` is the next power of two above `√24`, and 24 is the
  **deepest fold measured on the production world** (3 agents × 8 chapters). So
  the bound survives folds four times deeper than anything the world produces
  **without being re-tuned** — which is the property that keeps it evidence.
- `NEAR_ZERO_FLOOR = 1e-12` because a purely relative bound degenerates near
  zero, and both sides of such a comparison still ride f64 there.
- The **physical cross-check**: at the largest production fold (6.09 m) the
  bound is `8 · 2^-24 · 6.09 = 2.9e-6 m`, five orders of magnitude below the
  0.1125 m eighth. A disagreement this bound forgives cannot reach the world.

The derivation lives in the doc comment on `stored_fold_tolerance`, beside the
constant, which is where S20 § 4.3 said it belongs.

**The one honest loss, stated rather than hidden.** The failing assertions were
asserting something real — *"the fact records exactly what the edge moved"* —
and in the resident record that is now *"records what the edge moved, to 24
bits"*. That is a true, tiny loss of information. The response is to write the
weaker claim down, not to pretend the stronger one survived.

## Residue

- **The pager is not built.** S20 § 5 is unchanged and unpaid: a first
  filesystem dependency in a headless crate (properly an injected port, not
  `std::fs` in `dc-worldgen`), a versioned derived artifact with a validated
  staleness path, a writer/reader split across process lifetimes, two artifacts
  rather than one, and a new *"provenance unavailable"* answer. `dc-core`,
  `dc-sim` and `dc-worldgen` still contain zero `std::fs` / `std::io` /
  `File::` in `src/`.
- **The resident fold-in-the-CSR-row is not built either.** That is the other
  half of option 3: `SlotRun { slot: u32, start: u32 }` becoming
  `{ slot: u32, product: f32 }` — the same 8 B, the fold riding free inside the
  index.
- **`flow_cost_probe`'s pre-CSR counterfactual** priced its historical heap at
  `size_of::<Fact>()`. That number silently meant "16" and now silently means
  "8", which would have quietly rewritten a historical figure. It is now
  `size_of::<Fact<FracM>>()` — still a live type's width, still 16, and still
  something a future change has to walk past deliberately.
