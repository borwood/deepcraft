# 0100 — The ledger that stored mostly absence

*2026-07-25 — a pure layout change to `FactLedger`: flat facts plus a sparse CSR
index, ported from `flux.rs`. Same facts, same numbers, same world, a fraction of
the memory.*

> blogworthy: **lens 3 (reflexions in a deepsim codebase)** and **lens 1
> (AI-native development)** — a measurement taken to protect a record that did
> not exist yet turned around and condemned the record that did; and the fix was
> not designed, it was *copied from the file next door*.

## The measurement came first, and it was not about this

`docs/spikes/S19-flow-record-cost-results.md` exists to answer one question:
what would the *flow* record cost, if we built it? Nobody was auditing the fact
ledger. But the projection needed a structural analogue to calibrate against, and
`FactLedger` was the only fact-shaped record in the tree — so it got measured in
passing, as a control.

The control was the finding:

- **5,832,862 inner `Vec<Fact>`** (one per stratum slot per cell, plus one
  bedrock seam per cell), of which **5,760,856 were EMPTY — 98.8 %**
- **headers alone = 133.50 MiB = 89 % of the ledger's 150.11 MiB heap**
- the facts themselves: **16.61 MiB over 1,033,189 of them**

Turning `weather_inventory` ON cost **+156.91 MiB**, which — after the `strata`
`shrink_to_fit` win landed earlier the same day — was **~1.4× the entire rest of
the `DeepField`**. And the flag is on its way to becoming a default: the walk
blessed the band (journal/0097). So the single largest residency item in the
world was about to be, by a wide margin, *the absence of facts*.

`Vec<Vec<T>>` is such a natural way to say "a list per thing" that it does not
read as a decision. It is one. It says: *every* thing pays 24 bytes for the
privilege of possibly having a list. When 98.8 % of the things have nothing, you
have built a very expensive way of storing nothing.

## The fix was already in the tree, three files away

The temptation here — this project's characteristic failure, A-4 — is to design a
nice sparse ledger. There was nothing to design. `deeptime/flux.rs`
(journal/0096) had shipped that same day storing a *much larger* sparse
per-(cell, chapter, face) record as **flat exact-sized arrays plus a CSR index**,
and had measured its index floor at **0.056× of total** — the index is free, the
payload is the whole constraint. The S19 probe is *why* `flux.rs` was built that
way. So the work was: read how `FluxRecord` lays out, indexes and iterates, and
do that.

`FactLedger` is now:

```rust
pub struct FactLedger {
    facts: Vec<Fact>,     // grouped by slot, append-ordered within a slot
    rows: Vec<SlotRun>,   // (slot: u32, start: u32) — one per NON-EMPTY slot
}
```

One difference from `flux.rs`, and it matters. `FluxRecord`'s `cell_start` is a
*dense* offsets array — `n + 1` entries, one per cell whether or not that cell
has entries — because every cell exists and the array *is* the addressing scheme.
`FactLedger` cannot afford that: dense offsets would be `slots + 1` per cell,
which is precisely the rectangle we are trying not to allocate (5.8 M `u32` =
22 MiB of index to address 72 k non-empty rows). So the row carries its own slot
key and is emitted only for slots that have facts — a doubly-compressed row
index. A lookup is a binary search over a row list that is, in production,
almost always of length one.

**An empty ledger allocates nothing at all.** That is the case that dominates:
most cells never weather, and under the old shape each of them still paid a
header per stratum it had ever deposited.

## The triangular exploit, and why it turned out not to be the point

The slice pointed at a real property of the data: facts are **causally
triangular**. A slot deposited in chapter `c` cannot carry a fact from before `c`
— it did not exist — so the honest dense ceiling is `Σ (K − chapter)` =
25,536,276 (slot, chapter) pairs, not `slots × K` = 44,286,696. A free **1.73×**,
and every S19 projection uses it.

Exact-sizing subsumes it. We allocate neither rectangle: we allocate the
1,033,189 facts that exist, which is **4 %** of even the causal ceiling. The
causal correction is the right number to reason with when you are *projecting* a
record you have not built — which is what S19 was doing — but once you are
building it, "store what happened" beats every bound on "what could have
happened". Worth saying plainly, because the two are easy to conflate: the causal
bound is a budgeting tool, not a sizing rule.

## The trap: fact order is observable

This is the part that would have been a silent behaviour change.

`commit_chapter` drains an applied-edge log and merges each edge into the
**earliest** fact in that slot with a matching `(chapter, cause, from, to)` — the
A-4 fold from journal/0096, which deleted a second merger by making the first one
*search* instead of looking only at `last_mut()`. And `compose_unit` /
`compose_bedrock` fold facts **in order** through `apply_move`, which clamps each
move against what the portion multiset holds at that point.

So two orderings exist and only one is load-bearing:

1. **Within a slot**, first-occurrence order — because a merge lands on the first
   match, and because the fold's clamping is order-sensitive as soon as a slot
   holds more than one edge.
2. **Across slots**, nothing — slots are only ever read through `facts_for(i)`.

A flat array that groups by slot must preserve (1) exactly while inserting into
the middle of the array. `append_merged` does: it finds the slot's run, scans that
run **in order** and merges into the first match; a genuinely new fact is inserted
at the run's *end* and every subsequent row's start is bumped by one. Opening a
brand-new slot inserts the row at its sorted position and the fact at that row's
start. The memmove is `O(facts after this slot)` over a per-cell array of at most
a few dozen facts, at gen time — the clock this project spends freely, to buy the
clock it does not.

The tripwires were already in the tree and stayed unmoved:
`one_fact_per_agent_per_firing`,
`a_weathered_cell_carries_one_fact_per_agent_per_chapter_and_accumulates` (which
asserts no duplicate `(chapter, cause)` and `Σ shares == the composed product`),
`bedrock_facts_key_stably_as_the_record_grows`, and the golden
`the_production_world_still_hashes_to_the_pre_slice_goldens`. A new test,
`fact_order_within_a_slot_is_preserved_across_interleaved_slots`, states the
invariant directly rather than trusting the others to notice — it interleaves two
slots and repeats a cause non-consecutively, which is exactly the shape the
old-vs-new insert paths could disagree on.

## Exact-sized at the end, not grown in place

`strata`'s `Vec` doubling slack was **54.02 MiB = 33 % of all `DeepField`
residency** before it was reclaimed earlier the same day. A per-cell growable
`Vec` pays that, always. So the finalize step — `finalize_ledgers`, which re-keys
the in-loop accumulator's stable sentinel slot 0 onto the final record's bedrock
index — is now also the **compaction**: `FactLedger::rekeyed` builds the resident
ledger with both arrays exactly the size of their contents, and the accumulator's
growth room is dropped on the floor. The compile is over at that point; growing
room is pure waste.

## An aside: the probe was broken and said so

Re-running `examples/flow_cost_probe.rs` panicked before it printed a byte:

```
assertion `left == right` failed: itemised baseline must agree with
DeepField::resident_bytes (the shipped helper)
  left: 113820517   right: 156454637
```

The difference is 42.6 MB — exactly the flux record. FLOW slice 1 landed *after*
the probe was written, so `resident_bytes()` grew a term the itemisation did not
have. The assertion is the reason this was a five-second diagnosis instead of a
wrong number in a journal entry: a probe that itemises an authority must be
**checked against** that authority, or it becomes a summary wearing an
authority's clothes. Fixed here by adding the missing row, which is also the
first time the flux record has appeared in the residency table.

## The numbers

Production world, seed 1337, `Extent::Medium`, production flags,
`examples/flow_cost_probe.rs`, `weather_inventory` **ON**:

Both columns are **real runs of the same probe**, the pre-slice sources rebuilt
and re-measured rather than quoted from S19.

| | before (`Vec<Vec<Fact>>`) | after (CSR) |
|---|---:|---:|
| **`DeepField` total, flag ON** | **311.02 MiB** | **179.12 MiB** |
| cost of turning the flag ON | **+161.81 MiB** | **+29.91 MiB** — **5.41× less** |
| ledger heap | 155.01 MiB | **16.31 MiB** — **9.5×** |
| — payload | 21.51 MiB (5.74 MiB of it `Vec` slack) | 15.77 MiB, **exact** |
| — index / headers | 133.50 MiB (**86 %**) | **0.55 MiB** (**3.4 %**) |
| ledger structs | 6.80 MiB (24 B × 297,025) | 13.60 MiB (48 B × 297,025) |
| non-empty slots | 72,006 of 5,832,862 (**1.2 %**) | 72,006 rows, and nothing else |
| **gen: build the flag-ON field** | **85.6 s** | **34.7 s** |

The flag-OFF baseline both columns share is **149.21 MiB** — identical in both
runs, byte for byte, which is the control that makes the ON comparison mean
something. (Of it: strata heap 84.47 MiB, flux record 40.66 MiB.)

**The index is 3.4 % of the ledger** — below `flux.rs`'s 5.6 % floor, because a
row is paid only where facts exist. The payload is 97 % of the heap, which is
the shape you want: a record whose size is its contents.

The fact population is **unchanged to the fact**: 1,033,189 facts across 72,006
slots in both runs. That is the byte-identity claim in its most direct form —
not "the tests pass" but "the same one million facts, in the same slots".

Two things in that table deserve to be said out loud rather than glossed.

**The struct grew.** Per-cell `FactLedger` went 24 → 48 bytes (one `Vec` became
two): **+6.80 MiB**. It is paid back twenty times over, but it is *exactly the
same defect* this entry is about, one level up — a header per cell, paid whether
or not the cell has anything. See the follow-up below.

**Gen time went DOWN by 51 seconds, and that was not the goal.** Building the
flag-ON field went 85.6 s → 34.7 s, ~2.5×. The compaction and the CSR inserts do
add work, and they are swamped: `finalize_ledgers` used to allocate and zero
**5.8 million `Vec` headers — 133 MiB of memory it then never wrote to**. Not
paying for absence is not only a residency property. (Gen time is free by
doctrine, so this is reported, not celebrated; but it is a useful reminder that
"free" and "harmless" are different claims.)

One footnote on provenance: S19 recorded the flag-ON cost as **+156.91 MiB**;
today's like-for-like re-measurement of the same pre-slice code reads
**+161.81 MiB**. The header term is identical (133.50 MiB) and the fact count is
identical; the 4.9 MiB is inner-`Vec` capacity slack, which moved when the
journal/0096 merger fold changed how facts are appended. The re-measured number
is the one this entry uses — a stale baseline quoted from a document is exactly
the sort of thing this project's corrections file is full of.

The facts, `weathering_product_m`, the band and the world are **identical** —
this was a pure layout change, and every existing test passed unmoved and by
name.

## What is still on the table

The per-cell `FactLedger` struct is now two `Vec` headers × 297,025 cells. The
*full* `flux.rs` shape — one record for the whole grid, with the cell as the CSR
row — would collapse that too, but it means changing `DeepField::ledgers` and
`ledger_at_voxel`, which belonged to another agent's write-set this cycle. Filed
in the ROADMAP, not done. The remaining lesson generalises past this record: any
per-cell owning container in a 297 k-cell field is a header multiplied by 297 k
before it has stored anything, and the deep field has several.
