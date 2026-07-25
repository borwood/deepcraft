# S20 — what should the fact ledger COST, and where should it LIVE?

*Measured 2026-07-25. Probe: `crates/dc-worldgen/examples/s20_ledger_residency_probe.rs`
(gated, `[[example]] test = true`). World: seed **1337**, `Extent::Medium` — the world
`dc-client` boots.*

**This spike does not make the call.** It prices four options and states, for each, what
it costs and which of the arc's commitments it keeps or breaks. The decision is the
user's.

> NUMBERS PENDING — the production run is queued behind a sibling's gate.

---

## 0. The question, and why it has a clean answer shape

journal/0106 closed R3 and moved the blocker. Gen time is affordable (25.7 s → 46.4 s;
*gen time is not a constraint*). Residency is not: the `LedgerField` sidecar projects
**17.45 MiB → 973 MiB**, and it is **resident** — it lives in `Pregen.deep.ledgers`,
which `WorldGenerator` holds for the process lifetime.

The design conversation named the discriminator, and the code agrees with it exactly:

- **"What is this rock?"** is `collapse.rs:1509` — `ledger_at_voxel(...).weathering_product_m(...)`,
  read **once per chunk column, for every chunk, forever**. It consumes **the fold**: one
  scalar. It never looks at `chapter`, at `cause`, or at the endpoints.
- **"What happened here?"** is the provenance read (`UnitProvenance::of` → `base + facts`),
  which needs **the axes** — and is issued **rarely, at one voxel**, when a player inspects.

So the crisis is precisely as stated: **a cold/local query's data is being stored at the
residency of a hot/global one.** Every option below is a different answer to "where does
the axis data live", and the fold is common to all of them.

The constraint that rules out the naive fix: **compaction must not mean deletion.**
Collapsing to a per-slot scalar and discarding the facts makes a summary the authority —
`ARCHITECTURE.md` § *"A summary is not an authority"*, and `spines.md` **A-1** (a stand-in
becomes the definition). The facts are *not re-derivable*: material-behavior.md §1 says so
in as many words — they "are the outcome of stochastic, state-reading behaviors — not
re-derivable without re-running the compile". Deleting them is irreversible.

**The shape this already has a name for.** Option 3 (resident fold + paged facts) is not a
new mechanism; it is **S-9** — *derivable base + sparse committed facts + fallback query,
every answer carrying a resolution flag* — with the fact overlay placed on disk instead of
in RAM. That matters for how it should be judged: the question is not "is paging a good
idea", it is "may the S-9 overlay be non-resident, and what does the resolution flag say
when it cannot be reached".

---

## 1. Assumptions, stated up front

Every number below rests on these. They are the same ones journal/0106 used, re-derived
here rather than inherited.

1. **The fact count model.** A per-depth pass commits one fact per **(cell, slot, chapter,
   agent)**. The agent count is *measured*, not assumed: it is `today_facts /
   cell_chapter_firings` on the real record.
2. **The causal triangle.** A slot deposited in chapter `c` cannot weather before `c`. The
   record is appended bottom-up and `DepUnit::chapter` is non-decreasing, so the slots
   present at chapter `c` are an exact **prefix**. Visits are counted over that prefix.
3. **The visited-slot set is the fact-carrying set.** Every visited slot is assumed to
   commit facts. Slots that weather to below `EPS` commit nothing, so the real figure is
   **somewhat lower** — but a slot is visited *because* it is exposed to reactant, so the
   discount is not an order of magnitude.
4. **Eroded units are invisible.** Units removed before the end of the run are gone from
   the final record, so the reconstructed per-chapter depth is a **lower bound** on what a
   firing actually saw.
5. **The transport field is not modelled.** The arc's other half (reactant transport) is a
   *field* pass — per-cell-per-epoch, today's cost class. It changes the constant, not the
   exponent, and it adds no per-slot facts.
6. **The paged file is geometry-faithful, not content-faithful.** The prototype writes the
   projected per-cell run lengths with deterministic filler. Page-in latency is a function
   of offset, length and device — not of what the bytes say.
7. **"Cold" means the OS page cache is bypassed**, via `FILE_FLAG_NO_BUFFERING`. The
   device's own DRAM cache is not bypassed and cannot be from user space; on a file of
   ~1 GiB just written, some of it may still be in the drive's cache, so the cold figure is
   a **lower bound on true cold**.

---

## 2. Measured geometry

> NUMBERS PENDING

---

## 3. The four options

> NUMBERS PENDING

---

## 4. Engineering cost of paging, stated without flattery

*(prose below is independent of the measured latencies; the latencies decide whether it is
worth paying, not what it is.)*

**a. It is the first filesystem dependency in a headless crate.** `dc-core`, `dc-sim` and
`dc-worldgen` contain **zero** occurrences of `std::fs` / `std::io` / `File::` in `src/`
today. CLAUDE.md's rule is "headless crates never depend on rendering/OS"; a pager written
with `std::fs` inside `dc-worldgen` is a new capability class for that crate, and it is the
kind of thing that is easy to add and very hard to remove. The honest shape is an
**injected port** — a `FactStore` trait `dc-worldgen` defines and does not implement, with
the file-backed implementation living wherever the save layer lives. That is more design
than "write a file", and pretending otherwise is how a spike misleads.

**b. The file is a derived artifact of a world, and nothing today has a place to put one.**
It must be keyed by `(seed, extent, deep config, worldgen version)` and it must be
*validated*, not assumed: a stale file silently paired with a fresh world produces
confident wrong provenance, which is worse than no provenance. That means a header, a
version, and a mismatch path — and the mismatch path has to be a real branch someone
tested, not a comment.

**c. Regeneration.** Today `build_field_cfg` produces the `DeepField` in memory and the
collapse consumes it immediately, in one process. Paging splits that into a **writer at
compile end** and a **reader whose lifetime is longer than the generating process**. If the
world is re-generated the file must be rewritten; if generation is interrupted the file is
partial and must be detectable as such.

**d. Eviction.** The resident fold survives eviction because it is resident. The facts do
not need to. But note what this implies: **the fold must be persisted too**, or a
process that did not generate the world has no fold either — and rebuilding the fold from
the paged facts means reading the whole file, which is the thing paging exists to avoid.
So the artifact is really *two* artifacts: a small always-loaded fold, and a large
seek-only fact file.

**e. What breaks when the file is missing.** The hot query is unaffected — "what is this
rock?" reads the fold. The provenance query must return an honest **"unavailable"**, never
a fabricated or partially-reconstructed story. The project already has the right shape for
this (journal/0101, *an answer allowed to say nothing*; S-9's resolution flag), so this is
a cost that lands on an existing mechanism rather than a new one. It is still a new failure
mode in a user-facing answer.

**f. Concurrency.** Worldgen is rayon-parallel, but `LedgerField::from_accumulators`
already performs a single grid-wide serial pass at finalize, so the writer is naturally
single-threaded and append-only. No new concurrency problem. Readers are read-only.
This one is genuinely cheap.

**g. `mmap` is the obvious alternative and it is not free either.** A memory-mapped file
removes the seek/index code and gets OS-granularity paging for nothing, but it needs a
platform crate (`memmap2`), it puts address space (not RSS) back on the resident side, and
a page fault inside the render thread is a stall you cannot see in a profile as I/O. It is
worth pricing separately; this spike measured explicit reads because they are the version
whose cost is legible.

**h. Determinism is not threatened.** The file is a pure function of the seed and config;
no wall clock, no ambient randomness. The prototype's filler is LCG-seeded from a
caller-owned constant for the same reason.

---

## 5. What each axis-drop destroys

This is the price, and it is paid in the currency the arc exists to protect. Neither drop
is reversible: §1 states the facts are not re-derivable without re-running the compile.

**Drop the CHAPTER axis** (merge a slot's whole history into one fact per agent) — **you
lose *when*, and with it the entire derived-story layer.** material-behavior.md §1 is
explicit that the fact stores only the non-derivable core and that the **pass**, the
**epoch**, and the **environmental driver** ("because heat/pressure/water was thus") are
*re-derived at read time from the field state at that chapter*. Remove `chapter` and there
is no field state to read: the story layer does not degrade, it stops existing. Facts also
become unordered — §1 says they are "ordered by chapter; within a chapter a commutative
batch", and dropping the axis promotes the whole history to one commutative batch. The
surviving answer is "frost took 0.8 m out of this slot, in total, at some point".

**Drop the AGENT axis** (keep the sum) — **you lose *who*.** This retires, at the storage
layer, the thing that was ratified one day earlier: §4's DECIDED 2026-07-24 replaced the
product model with a sum *specifically because* only a sum makes each agent's share
well-defined, "hence **one fact per agent** (§1)", preserving "frost did 3, biotic did 2".
Dropping the axis makes `Σ share_a` the only survivor, which is exactly the quantity the
product model already gave — the ratified change becomes unobservable in the record. It
also forecloses the forward-note: the present-tier `Actor` cause ("player P deposited it")
is *the same axis*. Drop it here and the ledger cannot address present-tier provenance at
all.

**Drop BOTH** — what remains is one number per (slot, edge): "this much of this slot turned
into that". §1's *"the fact is the seed; the story is derived-and-displayed"* has no seed
left. This is the summary-as-authority defect wearing a fact's paperwork, and it is A-1.

**A note on which drop is cheaper in provenance terms, since the byte costs will not
decide it.** The chapter axis is the one the *story* is keyed off; the agent axis is the
one the *most recent ratification* is about. They are not interchangeable, and the byte
saving from each is measured below precisely so the trade is not made on intuition.

---

## 6. What this means for the design

> NUMBERS PENDING
