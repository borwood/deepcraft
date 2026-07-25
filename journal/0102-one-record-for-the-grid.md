# 0102 — The header you pay 297,025 times

*2026-07-25 — the same defect as journal/0100, one level up: `DeepField` held a
`Vec<FactLedger>`, an owning container per deep cell. The fix is the shape from
the file next door, applied one dimension higher: one record for the whole grid,
with the **cell** as the CSR row.*

> blogworthy: **lens 3 (reflexions in a deepsim codebase)** — a slice whose entire
> content is *"do the thing you did yesterday, one level up"*, and whose most
> valuable output is not the megabytes but the sentence that generalises them.

## The regression the previous slice flagged on itself

journal/0100 cut the fact ledger's heap 9.5× — 155.01 → 16.31 MiB — by replacing
`Vec<Vec<Fact>>` with flat facts plus a sparse CSR index over slots. It also, in
the same edit, grew the per-cell `FactLedger` struct from 24 bytes to 48: one
`Vec` became two. Across a production deep grid of **297,025 cells** that is
**13.60 MiB**, and the slice wrote it into its own results table rather than
letting it hide inside a twenty-fold win:

> **The struct grew.** … It is paid back twenty times over, but it is *exactly the
> same defect* this entry is about, one level up — a header per cell, paid whether
> or not the cell has anything.

That is the whole brief for this one. There is no new insight to find; the
insight was found yesterday and deliberately left with an address.

## The number that makes it not a rounding error

13.60 MiB against a 156 MiB flag-off field is 8.7 %. That alone would be worth
doing and not worth writing about. What makes it worth writing about is the
denominator on the *other* side: **225,019 of the 297,025 cells — 75.8 % — carry
no fact at all.** They are marine, or they are border wilds, or they simply never
stood above water long enough to weather. Every one of them was paying two `Vec`
headers for the privilege of possibly having facts.

So the shape of the waste is identical to journal/0100's, and it is worth stating
in the form that outlives both structs:

> **Any per-cell owning container in a 297 k-cell field costs a header per cell
> before it holds data.** The default for anything per-cell is *one grid-wide
> record with CSR rows*, never `Vec<Something>` per cell.

`Vec<Vec<T>>` does not read as a decision. Neither does `Vec<Thing>` where `Thing`
owns a `Vec`. Both are. The deep field has several more.

## The port, and the one thing that is different a level up

`deeptime/flux.rs` stores its record as flat `entries` plus a **dense** `cell_start`
offsets array of `n + 1` `u32`s. journal/0100 could not use the dense form,
because dense-offsets-*per-slot* is precisely the 5.8 M-entry rectangle it was
avoiding; it gave each row its own slot key instead, a doubly-compressed index.

One level up, the dense form is available again — because every **cell** exists.
So `LedgerField` is both at once:

```rust
pub struct LedgerField {
    facts: Vec<Fact>,          // every fact in the world, cell-major then slot-major
    rows: Vec<SlotRun>,        // (slot, start) — one per NON-EMPTY (cell, slot)
    cell_row_start: Vec<u32>,  // n + 1 — cell i's slice of `rows`
}
```

The **outer** index is dense (`flux.rs`'s shape: 4 B/cell, 1.13 MiB, paid for every
cell whether or not it weathered). The **inner** index is sparse (journal/0100's
shape: 8 B, paid only where facts exist). Which compression is right is not a
matter of taste — it is decided by whether the thing being indexed is guaranteed
to exist. Cells are; slots are not.

`SlotRun::start` stays **absolute** into the flat fact array, so a row's run ends
where the next row begins — uniformly, *including across a cell boundary*. That is
the one place this could have been quietly wrong, and it has a test with the
smallest configuration that can catch it: two adjacent non-empty cells, where cell
0's last row is followed not by another of its own rows but by cell 1's.

## A view is not a struct

The consumer side is where a layout change usually leaks. `ledger_at_voxel`
returned `Option<&FactLedger>` and there is no longer any `FactLedger` to point
at. It now returns a `LedgerView<'_>` — a borrowed window carrying the whole fact
array, this cell's rows, and the cell's end offset — with exactly the read surface
the owned struct had: `facts_for`, `bedrock_composition`, `weathering_product_m`,
`is_empty`, `total_facts`, `slots_with_facts`.

The measure of whether that worked: **not one call site changed.** The collapse
tier's

```rust
.ledger_at_voxel(cx * 32 + 16, cz * 32 + 16)
.map_or(0.0, |l| l.weathering_product_m(deep_units.len()))
```

compiles untouched, as does `field.ledgers.get(i).map_or(0.0, …)` in the tour and
the profile probe, and `for (i, ledger) in field.ledgers.iter().enumerate()` in the
S18 test — because `get` still returns an `Option` of something with the method,
and `iter()` still yields one item per cell. The only edit outside the two owning
files was three lines inside one test that wrote `finalized[0]`; a `Vec` can be
indexed and a record that hands out views cannot. Same test, same name, same
assertions.

That is the honest test of a "pure layout change": if the read surface is really a
*surface*, replacing what is behind it should be invisible.

## Where the per-cell container is still right

`FactLedger` did not go away, and this is the interesting half of the design.

The weathering pass appends into **one cell's** ledger every epoch, two hundred
times, merging into the earliest matching fact and inserting new ones in the
middle. In a per-cell array that is a memmove of at most a few dozen facts. In a
grid-wide array it would be a memmove of *every fact after this cell* — up to a
million, per firing. Porting the layout all the way into the accumulator would
have been the letter of the shape and a catastrophe in fact.

So the split is by **clock**, not by structure: `FactLedger` is gen-time scratch,
per-cell and growable, exactly as the compile needs; `LedgerField` is the resident
record, grid-wide and exact-sized, exactly as runtime needs. `finalize_ledgers` is
the one place they meet, and it was already the compaction step — journal/0100 had
made it so — which is why this collapse cost one function body and no new concept.

That is worth saying plainly because "a per-cell owning container is a defect"
would be the wrong lesson. The defect is a per-cell owning container that is
**resident**.

## An aside: the probe was broken again, in exactly the same way

Re-running `examples/flow_cost_probe.rs` to take the before-numbers panicked
before printing a byte. Same assertion, same shape, different missing term:
`DeepField::resident_bytes` had grown a `head` plane when FLOW continuation (a)
merged (journal/0098), and the probe's itemisation had not.

journal/0100 hit this for the flux record and CLAUDE.md gained a rule out of it —
*`cargo test` builds examples but never runs them, so re-run the probes by hand
after any merge that changes what they measure.* Two weeks would have been an
excusable gap. It was **one day**, and the rule that was supposed to catch it was
already written down. The assertion caught it anyway, which is the actual moral:
the rule is advisory and the assertion is not. An itemisation that is checked
against its authority cannot silently drift; one that isn't will, and will do it
again next merge.

## The numbers

Production world, seed 1337, `Extent::Medium`, production flags,
`examples/flow_cost_probe.rs`. Both columns are **real runs of the same probe** on
this machine within the hour, not quoted from a previous entry.

| | before (`Vec<FactLedger>`) | after (`LedgerField`) |
|---|---:|---:|
| **`DeepField` total, flag ON** | **186.07 MiB** | **173.61 MiB** |
| cost of turning the flag ON | **+29.91 MiB** | **+17.45 MiB** — **1.71× less** |
| — **per-cell struct overhead** | **13.60 MiB** (48 B × 297,025) | **0 B — there is no per-cell struct** |
| — ledger heap | 16.31 MiB | 17.45 MiB |
| —— payload (the facts) | 15.77 MiB, exact | 15.77 MiB, exact — *identical* |
| —— index | 0.55 MiB (slot rows) | 1.68 MiB (0.55 slot rows + **1.13 dense cell offsets**) |
| **per-cell index cost** | **48 B/cell** | **4 B/cell — 12×** |
| facts / non-empty slots | 1,033,189 / 72,006 | 1,033,189 / 72,006 |
| **flag-OFF baseline** | **163,748,661 B = 156.16 MiB** | **163,748,661 B = 156.16 MiB** |
| gen: build the flag-ON field | 25.9 s | 24.7 s |

The flag-OFF baseline is **the same integer in both runs** — 163,748,661 bytes,
not "156.16 MiB both times". That is the control that makes the flag-ON comparison
mean anything: nothing that was not the ledger moved. (Flag-ON before, in bytes, is
195,112,933 — the printed 186.07 MiB, and exactly `182,043,837 + 297,025×48 −
297,026×4`, which is the arithmetic the two runs have to agree on if the change
really was only the index.)

**Read the index row honestly.** The ledger's *heap* went slightly **up**, and its
index went from 3.4 % of the heap to 9.6 %. Both are true and neither is the point:
the 1.13 MiB the index gained is the dense per-cell offsets array, which is the
**cheap replacement** for 13.60 MiB that used to be counted outside the heap as
"ledger structs". The comparison that matters is per-cell: **48 bytes → 4 bytes**.
A row of an accounting table can move the wrong way while the accounting improves,
which is why the table carries the term that vanished as its own line.

**The fact population is unchanged to the fact**: 1,033,189 facts across 72,006
non-empty slots, before and after. That is byte-identity in its most direct form —
not "the suite is green" but "the same million facts, in the same slots".

**Gen time did not move.** 25.9 → 24.7 s on the flag-ON field, against a flag-OFF
pregen that moved 20.2 → 22.3 s in the *opposite* direction on the same two runs.
Both deltas are machine noise between sibling agents; the honest reading is *no
measurable change*, which is what a one-pass compaction of an array we were already
building should cost. (Unlike journal/0100, where 133 MiB of headers stopped being
allocated and the clock noticed.)


## What is still on the table

**The lever this slice was named after is discharged, and the generalisation is
not.** The deep field still holds several per-cell owning containers, and the
largest by far is the one this entry did not touch: `strata: Vec<DeepStrata>` is
**9.06 MiB of 32-byte structs**, 5.8 % of the flag-off field, on top of an 84.47 MiB
heap — and 33,680 of those cells (11.3 %) have an empty record. The same collapse
applies verbatim: one grid-wide `DepUnit` array with the cell as a CSR row. It is
larger than this slice was because `DeepStrata` is written per-epoch during the
compile and read all over the collapse tier, so it needs the same clock split
(`DeepStrata` stays the gen-time accumulator; the resident record is grid-wide) and
a wider read of its call sites. **Filed in the ROADMAP, not done.**

Worth stating the rule of thumb the two slices together produce, because it is
cheaper than re-deriving it a third time:

> A per-cell container is a **gen-time** shape. If it survives into the shipped
> field, its headers are multiplied by the cell count and paid forever. Compact at
> the seam where the compile ends — which, in this codebase, already exists and is
> called `finalize_*`.

