# S20 — what should the fact ledger COST, and where should it LIVE?

*Measured 2026-07-25. Probe: `crates/dc-worldgen/examples/s20_ledger_residency_probe.rs`
(gated, `[[example]] test = true`). World: seed **1337**, `Extent::Medium` — the world
`dc-client` boots. Machine: NVMe system volume (`C:`, WD SN750) and SATA SSD data volume
(`B:`, SK hynix), both measured.*

**This spike does not make the call.** It prices four options and states, for each, what it
costs and which of the arc's commitments it keeps or breaks. The decision is the user's.

**It builds no part of the arc.** `Fact`, `FactLedger`, `LedgerField` and every weathering
pass are untouched. The alternative encodings are local candidate structs measured with
`size_of`; the paged prototype writes a geometry-faithful synthetic file to a temp
directory and deletes it.

---

## 0. The question, and why it has a clean answer shape

journal/0106 closed R3 and moved the blocker. Gen time is affordable (25.7 s → 46.4 s;
*gen time is not a constraint*). Residency is not: the `LedgerField` sidecar projects
**17.45 MiB → 973 MiB**, and it is **resident** — it lives in `Pregen.deep.ledgers`, which
`WorldGenerator` holds for the process lifetime.

The design conversation named the discriminator, and the code agrees with it exactly:

- **"What is this rock?"** is `collapse.rs:1509` —
  `ledger_at_voxel(...).weathering_product_m(...)`, read **once per chunk column, for every
  chunk, forever**. It consumes **the fold**: one scalar. It never looks at `chapter`, at
  `cause`, or at the endpoints.
- **"What happened here?"** is the provenance read (`UnitProvenance::of` → `base + facts`),
  which needs **the axes**, and is issued **rarely, at one voxel**, when a player inspects.

So the crisis is precisely as stated: **a cold/local query's data is stored at the residency
of a hot/global one.** Every option below is a different answer to *where the axis data
lives*; the fold is common to all of them.

The constraint that rules out the naive fix: **compaction must not mean deletion.**
Collapsing to a per-slot scalar and discarding the facts makes a summary the authority —
`ARCHITECTURE.md` § *"A summary is not an authority"*, `spines.md` **A-1**. The facts are
*not re-derivable*: material-behavior.md §1 says so in as many words — they "are the outcome
of stochastic, state-reading behaviors — not re-derivable without re-running the compile".
Deleting them is irreversible.

**The shape this already has a name for.** Option 3 (resident fold + paged facts) is not a
new mechanism; it is **S-9** — *derivable base + sparse committed facts + fallback query,
every answer carrying a resolution flag* — with the fact overlay placed on disk instead of
in RAM. The question is therefore not "is paging a good idea" but "may the S-9 overlay be
non-resident, and what does the resolution flag say when it cannot be reached".

---

## 1. Assumptions, stated up front

Every number below rests on these. They are journal/0106's assumptions, re-derived here
rather than inherited.

1. **The fact count model.** A per-depth pass commits one fact per **(cell, slot, chapter,
   agent)**. The agent count is *measured*: `today_facts / cell_chapter_firings` = **2.9999**
   on the real record (three agents, as expected — printed rather than hardcoded).
2. **The causal triangle.** A slot deposited in chapter `c` cannot weather before `c`. The
   record is appended bottom-up and `DepUnit::chapter` is non-decreasing, so the slots
   present at chapter `c` are an exact **prefix**. Visits are counted over that prefix.
3. **The visited-slot set is the fact-carrying set.** Slots that weather to below `EPS`
   commit nothing, so the true figure is **somewhat lower** — but a slot is visited
   *because* it is exposed to reactant, so the discount is not an order of magnitude.
4. **Eroded units are invisible.** Units removed before the end of the run are gone from the
   final record, so the reconstructed per-chapter depth is a **lower bound** on what a
   firing actually saw.
5. **The transport field is not modelled.** The arc's other half (reactant transport) is a
   *field* pass — per-cell-per-epoch, today's cost class. It changes the constant, not the
   exponent, and it adds no per-slot facts.
6. **The paged file is geometry-faithful, not content-faithful.** The prototype writes the
   projected per-cell run lengths with deterministic LCG filler. Page-in latency is a
   function of offset, length and device, not of what the bytes say.
7. **"Cold" means the OS page cache is bypassed** (`FILE_FLAG_NO_BUFFERING`). The device's
   own DRAM cache is not bypassed and cannot be from user space, so the cold figure is a
   **lower bound on true cold**.
8. **Row count is the same in every option.** All four keep one CSR row per non-empty
   `(cell, slot)`; only the fact payload moves. `rows = 4 516 541` throughout.

---

## 2. Measured geometry

```
deep cells 297 025, epochs 200, chapters 8
cells that ever weather                72 006  (24.2 %)

TODAY      facts    1 033 189   rows      72 006  =   17.45 MiB
           (measured footprint 17.45 MiB — itemisation reconstructs it EXACTLY)
(cell, chapter) firings 344 410 ; facts per firing 2.9999 (measured agent count)

PER-DEPTH  (cell, slot, chapter) visits, triangular   20 487 597
PER-DEPTH  facts   61 460 352   rows   4 516 541  =  973.40 MiB
```

**journal/0106's 973 MiB reproduces exactly**, from an independently written itemisation
checked byte-for-byte against `LedgerField::footprint_bytes()`. The itemisation is
`facts × 16 B + rows × 8 B + (cells + 1) × 4 B`; at per-depth that is **937.81 + 34.46 +
1.13 MiB**. Note where the money is: **96.3 % of the projected residency is fact payload**,
which is why the levers that matter are fact *width* and fact *count*, and why the CSR index
is not worth attacking.

Other measured facts used below:

- **mean chapters fired per weathering slot: 4.54** (= visits / rows) — the chapter axis's
  own multiplicity, measured rather than the ~4.8 estimated in journal/0106.
- **distinct `(from → to)` edges inhabited on this record: 1.** Every weathering fact is
  `(GRANITE, Structure) → (GRANITE, Loose)`.
- **stored `fraction_m` magnitudes span 1.0014e-9 m … 0.7038 m**; the smallest sits right on
  the `EPS = 1e-9` gate. Deepest fold: **24 summands** (3 agents × 8 chapters).

---

## 3. The four options

Resident = what ships in the `DeepField`. Disk = what a pager would hold. Page-in is the
cold p50–p95 range across both devices and three runs.

| # | Option | Resident | Disk | Page-in (cold) | Provenance retained | Engineering cost |
|---|---|---:|---:|---|---|---|
| 1 | **Baseline** — full resolution, fully resident (16 B `Fact`) | **973.40 MiB** | — | n/a | **everything** (chapter · agent · edge · fraction) | **zero** — today's code at per-depth |
| 2a | edge id only (u16 + f64) | **973.40 MiB** | — | n/a | everything | small: declare the edge table, map at commit |
| 2b | f32 only (endpoints + f32) | **738.95 MiB** | — | n/a | everything (fraction to 24 bits) | small: narrow at persist, widen on read |
| 2c | **edge id + f32, AoS (8 B)** | **504.50 MiB** | — | n/a | everything (fraction to 24 bits) | both of the above |
| 2d | edge id + f32, SoA (6 B) | **387.27 MiB** | — | n/a | everything (fraction to 24 bits) | 2c + a fact is no longer one contiguous object |
| 3 | **Resident fold + PAGED facts** (16 B on disk) | **37.86 MiB** | 937.85 MiB | **p50 102–216 µs, p95 151–344 µs** | **everything** | **large — see § 5** |
| 3′ | paged + 8 B compact facts | **37.86 MiB** | 468.91 MiB | same | everything (fraction to 24 bits) | as 3, plus 2c |
| 4a | drop CHAPTER (one fact per slot per agent) | **242.34 MiB** | — | n/a | agent · edge · fraction. **No *when*.** | small, and **irreversible** |
| 4b | drop AGENT (one fact per slot per chapter) | **348.21 MiB** | — | n/a | chapter · edge · fraction. **No *who*.** | small, and **irreversible** |
| 4c | drop BOTH (one fact per slot) | **104.51 MiB** | — | n/a | edge · total fraction only | small, and **irreversible** |
| 4a′ | drop CHAPTER + 8 B compact | 138.97 MiB | — | n/a | as 4a | 4a + 2c |
| 4b′ | drop AGENT + 8 B compact | 191.90 MiB | — | n/a | as 4b | 4b + 2c |
| 4c′ | drop BOTH + 8 B compact | **70.05 MiB** | — | n/a | as 4c | 4c + 2c |

Today's ledger is **17.45 MiB**. As multiples of it: option 1 is **55.8×**, option 2c
**28.9×**, option 3 **2.17×**, option 4c **5.99×**, option 4c′ **4.01×**.

### 3.1 The encoding finding: an edge id ALONE saves nothing

`Fact::InPlace` today is `{ chapter: u8, cause: u8, from: (u8,u8), to: (u8,u8),
fraction_m: f64 }` — 6 bytes of payload, 2 of padding, 8 of `f64` = **16 B**. Replacing the
four endpoint bytes with a `u16` edge id leaves 4 bytes of payload — and the `f64`'s 8-byte
alignment pads it straight back to **16 B**. Measured, not reasoned:
`size_of::<EdgeIdShape>() == size_of::<ShippedShape>()`, and the gate asserts it so the
finding cannot quietly stop being true.

**The two levers are only worth anything together.** f32 alone gets 16 → 12 B (the endpoints
still cost 6 bytes and align to 4). Both together get 16 → **8 B** with zero padding, which
is the point at which either lever pays. In struct-of-arrays form — a `u16` meta array
(3 bits chapter, 2 bits cause, 11 bits edge = 2 048 declared edges) beside an `f32` array —
it is **6 B**, at the cost that a fact is no longer one contiguous object.

The edge id's *real* value is not bytes. It makes the declared transition graph (S-8,
material-behavior §3) the authority for what edges exist, so a fact structurally cannot name
an undeclared transition. That is a validation gain, independent of size.

### 3.2 Does the edge id have to be world-stable?

**Today: no requirement at all.** There is **no `Serialize` on `LedgerField`, `FactLedger`,
`Fact` or `DeepField` anywhere in the tree** — the ledger is regenerated in-process on every
boot. Any id stable within one compiled binary suffices.

**The moment the ledger is persisted, yes** — and "ready-made worlds are the sanctioned
answer" (CLAUDE.md) means it will be. A positional id into a registry-ordered edge list
breaks on any registry change.

**But the shipped `Fact` already has exactly this exposure.** It stores `MaterialId`, which
is a positional `MatRepr` enum guarded by a compile-time `MATERIAL_COUNT == 26` assert. An
edge id does not create the instability; it inherits it, unchanged. The fix is the same for
both and it is cheap: **ship a per-world edge dictionary** beside the facts — `id →
(from-material-name, from-form, to-material-name, to-form)`. On this record that dictionary
has **one entry**; in any plausible future, a few dozen. Its cost is a rounding error against
937 MiB, and it turns a registry change from *silently reinterpreted* into *detected*.

### 3.3 Paging, measured

The prototype writes the projected per-cell runs to a real file, then reads a **scattered
deterministic sample** of 400 cells (a stride coprime with the population, so neither the OS
nor the device sees a sequential pattern to prefetch).

```
file          937.85 MiB   written in 0.7 s (NVMe) / 2.5 s (SATA)
RESIDENT fold-only projection   37.86 MiB
   = rows 4 516 541 × 8 B  +  cell row offsets 1.13 MiB  +  cell disk offsets 2.27 MiB
one cell's run: mean 12 966 B, max 84 288 B

                       mean      p50      p95        max
NVMe  COLD (run 1)    347.6    114.8    174.2   93 494.7
NVMe  COLD (run 2)    344.0    101.6    150.9   95 669.3
NVMe  COLD (run 3)    108.1    102.4    155.8      305.0   (max at read #356; first 170.4)
SATA  COLD (run 1)    179.4    176.5    300.7      370.6
SATA  COLD (run 2)    184.4    187.1    305.5      434.9
SATA  COLD (run 3)    211.4    216.1    343.7      438.3   (max at read #265; first 120.8)
NVMe  WARM              5.4      4.5     11.4    18.3 – 46.4
SATA  WARM              4.6      3.9     10.0    15.9 – 26.6
                                                     (microseconds, n = 400 each)
```

**The fold rides free inside the index.** A CSR row is `{ slot: u32, start: u32 }` = 8 B
today; under paging it becomes `{ slot: u32, product: f32 }` — the same 8 B. So the entire
37.86 MiB resident cost is *addressing*, not payload: 34.46 MiB of rows, 1.13 MiB of
per-cell row offsets, 2.27 MiB of per-cell disk offsets. There is no further residency lever
here short of coarsening the per-slot fold itself.

**The 95 ms tail, stated honestly.** Two of six device-runs showed a single ~95 ms sample on
the NVMe *system* volume. The third run, on a quiet machine, did not reproduce it (max
305 µs, at read #356 — not the first read, so it is not handle warm-up). The SATA volume
never showed it. p50 and p95 were stable across all six runs; only the max moved. Read:
**machine contention on the system volume, not device physics** — but it is exactly the kind
of tail a design must budget for, which argues for issuing the page-in off the render thread,
not for or against paging as such.

**The measured run is pessimistic.** 12 966 B is a *whole cell's column* of facts. A
provenance query at one voxel wants one slot, which is ≈ 3 agents × 4.54 chapters × 16 B
≈ **218 B** — a single 4 KiB sector. The numbers above are the cost of fetching everything a
column knows.

### 3.4 Axis-drops, and the multiplicity they remove

The chapter axis's measured multiplicity is **4.54** (visits / rows); the agent axis's is
**3.0**. Dropping chapter is therefore the *larger* byte saving (973 → 242 MiB) and dropping
agent the smaller (973 → 348 MiB) — the opposite of what the axis names suggest, because a
slot fires in more chapters than it has agents.

---

## 4. f32: store narrow, widen on read

The proposal is a **serialization** choice: compute in f64, narrow once at persist, widen on
read, keep every sum in f64. The error therefore happens once and never compounds.

### 4.1 Measured fold error

```
folds compared 72 006          deepest fold 24 summands
stored fraction_m magnitudes   min 1.0014e-9 m   max 0.70379 m
fold error                     max 1.2305e-7 m   mean 8.1530e-9 m   max relative 5.766e-8
largest fold                   6.0938 m
f32 relative resolution        2^-24 = 5.960e-8
cross-check |Σ − weathering_product_m|   0.000e0   (the simple sum IS the consumer's fold)
```

**Against the scale that matters:** one eighth of a voxel is **0.1125 m**. The worst fold
error is **1.094e-6 of an eighth — one part in 914 000**. The measured max relative error
(5.766e-8) sits *at* f32's own resolution (5.960e-8), which is the signature of a single
rounding rather than an accumulating one: the "widen on read, sum in f64" discipline is doing
what it claims.

The smallest stored fraction (1.0014e-9 m) sits on the `EPS = 1e-9` gate. f32 represents 1e-9
with full relative precision (nowhere near denormal), so the gate behaves identically.

### 4.2 Which tests fail, and whose fault it is

Every stored-value assertion in the tree falls into one of two classes, and the class is the
whole answer.

**Class A — stored vs stored: unaffected, still pass at 1e-12.** Both sides ride the same
rounded values (and `2×` is exact in binary), so narrowing does not move the comparison.

| site | assertion | verdict |
|---|---|---|
| `weather_inventory.rs:416` | `Σ facts` vs `weathering_product_m` | **passes** |
| `weather_inventory.rs:430` | composed total vs `BEDROCK_SEAM_THICKNESS_M` | **passes** — a form change subtracts and adds the same `q`, so mass conserves exactly at any width |
| `weather_inventory.rs:483` | `moved` vs `weather_rate` | **passes** — neither side is stored |
| `weather_inventory.rs:518` | `m2` vs `2 × m1` | **passes** — neither side is stored |
| `weather_inventory.rs:519` | `acc_band(a2)` vs `2 × acc_band(a1)` | **passes** — both stored, and `2×` is exact |
| `inventory.rs:1450, 1463, 1464, 1497` | `0.5`, `1.5`, `0.5`, `1.25` | **passes** — all four are exactly representable in f32 (measured error 0.000e0) |
| `s18_first_behavior_weathering.rs:127` | `Σ shares` vs `product`, 1e-9 | **passes** — both stored |

**Class B — stored vs a freshly-computed f64 (or an f64 literal): fails.** The tolerance is
being asked to certify agreement between two *different representations*, at a bound below
the resolution of one of them.

| site | assertion | measured error | verdict |
|---|---|---:|---|
| `weather_inventory.rs:417` | `Σ stored shares` vs `weather_rate(...)`, 1e-12 | 3.619e-10 | **FAILS 1e-12** (would pass 1e-9) |
| `weather_inventory.rs:482` | `acc_band` vs the f64 `moved`, 1e-12 | 3.619e-10 | **FAILS 1e-12** (would pass 1e-9) |
| `weather_inventory.rs:504` | 10-firing band vs `single × 10`, **1e-9** | **1.038e-9** | **FAILS 1e-9 — by 4 %** |
| `inventory.rs:1643` | stored `0.6`, 1e-12 | 2.384e-8 | **FAILS** |
| `inventory.rs:1647` | stored `0.3`, 1e-12 | 1.192e-8 | **FAILS** |
| `inventory.rs:1652` | stored `0.2`, 1e-12 | 2.980e-9 | **FAILS** |
| `inventory.rs:1654` | stored `0.4`, 1e-12 | 5.960e-9 | **FAILS** |
| `s17_deep_cell_inventory.rs:91` | stored `fraction_m` vs the f64 `moved`, 1e-12 | ≈ 6e-8 relative | **FAILS** |
| `s17_deep_cell_inventory.rs:100` | composed `sink_qty` vs the f64 `moved`, 1e-12 | ≈ 6e-8 relative | **FAILS** |

**Nine sites — and it is the tolerance's fault, but not *only* the tolerance's fault.** Both
halves, plainly:

- **The tolerance's fault.** Every failing bound is tighter than the storage's resolution.
  1e-12 on a 0.6 m quantity demands 2e-12 relative — four orders below f32. *No* f32-storing
  pipeline can meet it, and asking it to is a category error, not a defect report. Eight of
  the nine were written at 1e-12, which is an f64 round-trip bound.
- **Not only.** The stored value genuinely differs from the computed one, by up to 6e-8
  relative. The failing assertions assert something real — *"the fact records exactly what
  the edge moved"* — and under f32 that becomes *"records what the edge moved, to 24 bits"*.
  That is a true, tiny loss of information, and the honest response is to write the weaker
  claim down, not to pretend the stronger one survived.
- The one genuinely interesting case is **`weather_inventory.rs:504`**. Its author chose
  **1e-9** — a deliberately physical bound, not a round-trip one — and f32 storage misses it
  by 4 % (1.038e-9 against 1e-9). That is the only site where the old tolerance was even
  close to defensible, and it is the one to point at when deciding whether narrowing is worth
  it.

### 4.3 The tolerance a f32-stored / f64-summed pipeline deserves

The error is **relative by construction**, so the bound should be too:

```
|a − b|  ≤  8 · 2^-24 · max(|a|, |b|)  +  1e-12
        =   4.8e-7 relative            +  an absolute floor for near-zero comparisons
```

The factor 8 is the next power of two above √24 (the deepest measured fold), so the bound
survives deeper folds without re-tuning. If a single **absolute** constant is wanted instead:
the largest fold on the production world is 6.09 m, giving `8 · 2^-24 · 6.09 = 2.9e-6 m`, so
**1e-6 m** covers today's measured worst case (1.23e-7 m) with 8× headroom and is still
**five orders of magnitude below the 0.1125 m eighth** that could change what a player sees.

**Why this is not "loosening a tolerance until it goes green"** (journal/0106's third refused
option): the new bound is *derived from the storage's stated resolution*, published as a
formula, and independently checked against the physical scale it must stay under. A tolerance
with a derivation is evidence; a tolerance chosen until green is not. Whichever number is
adopted, the derivation belongs beside it in the code.

**One design consequence worth naming before it bites:** *where* the narrowing happens
decides which tests move class. If the gen-time `FactLedger` accumulator stays f64 and only
the resident `LedgerField` narrows at `from_accumulators`, then `weather_inventory.rs:549`
(`band_after_finalize` vs `band_in_accumulator`) moves from Class A to Class B, because the
two sides stop sharing a representation. That is the correct place to narrow — it is the
"once at persist" the proposal promises — but it is one more assertion to re-derive.

---

## 5. Engineering cost of paging, stated without flattery

**a. It is the first filesystem dependency in a headless crate.** `dc-core`, `dc-sim` and
`dc-worldgen` contain **zero** occurrences of `std::fs` / `std::io` / `File::` in `src/`
today (grepped). CLAUDE.md's rule is "headless crates never depend on rendering/OS"; a pager
written with `std::fs` inside `dc-worldgen` is a new capability class for that crate, and it
is easy to add and very hard to remove. The honest shape is an **injected port** — a
`FactStore` trait `dc-worldgen` defines and does not implement, with the file-backed
implementation living wherever the save layer lives. That is more design than "write a file",
and pretending otherwise is how a spike misleads.

**b. The file is a derived artifact of a world, and nothing today has a place to put one.**
It must be keyed by `(seed, extent, deep config, worldgen version)` and it must be
*validated*, not assumed: a stale file paired with a fresh world produces confident wrong
provenance, which is worse than none. That means a header, a version, and a mismatch path —
and the mismatch path has to be a branch someone tested, not a comment.

**c. Regeneration splits one process into two.** Today `build_field_cfg` produces the
`DeepField` in memory and the collapse consumes it immediately. Paging adds a **writer at
compile end** and a **reader whose lifetime is longer than the generating process**. A re-gen
must rewrite; an interrupted gen leaves a partial file that must be detectable.

**d. It is really *two* artifacts, not one.** The resident fold survives eviction because it
is resident — but a process that did not generate the world has no fold either, and
rebuilding the fold from the paged facts means reading the whole 937 MiB, which is the thing
paging exists to avoid. So the fold must be persisted too: a small always-loaded file and a
large seek-only one.

**e. What breaks when the file is missing.** The hot query is unaffected — "what is this
rock?" reads the fold. The provenance query must return an honest **"unavailable"**, never a
fabricated or partially-reconstructed story. The project already has the right shape for this
(journal/0101, *an answer allowed to say nothing*; S-9's resolution flag), so the cost lands
on an existing mechanism rather than a new one. It is still a new failure mode in a
user-facing answer.

**f. Concurrency is genuinely cheap.** `LedgerField::from_accumulators` already performs a
single grid-wide serial pass at finalize, so the writer is naturally single-threaded and
append-only. Readers are read-only. No new concurrency problem.

**g. `mmap` is the obvious alternative and it is not free either.** It removes the seek/index
code and gets OS-granularity paging for nothing, but it needs a platform crate (`memmap2`),
it puts address space back on the resident side, and a page fault inside the render thread is
a stall that does not look like I/O in a profile. Worth pricing separately; this spike
measured explicit reads because that is the version whose cost is legible.

**h. Determinism is not threatened.** The file is a pure function of seed and config — no
wall clock, no ambient randomness. (The prototype's filler is LCG-seeded from a caller-owned
constant for the same reason.)

---

## 6. What each axis-drop destroys

This is the price, and it is paid in the currency the arc exists to protect. **Neither drop
is reversible** — §1: the facts are not re-derivable without re-running the compile.

**Drop the CHAPTER axis** (−731 MiB, the biggest byte saving) — **you lose *when*, and with
it the entire derived-story layer.** material-behavior.md §1 is explicit that a fact stores
only the non-derivable core, and that the **pass**, the **epoch**, and the **environmental
driver** ("because heat/pressure/water was thus") are *re-derived at read time from the field
state at that chapter*. Remove `chapter` and there is no field state to look up: the story
layer does not degrade, it stops existing. Ordering goes too — §1 says facts are "ordered by
chapter; within a chapter a commutative batch", and dropping the axis promotes the whole
history to one commutative batch. What survives is "frost took 0.8 m out of this slot, in
total, at some point".

**Drop the AGENT axis** (−625 MiB) — **you lose *who*.** This retires, at the storage layer,
what was ratified one day earlier: §4's DECIDED 2026-07-24 replaced the product model with a
sum **specifically because** only a sum makes each agent's share well-defined, "hence **one
fact per agent** (§1)", preserving *"frost did 3, biotic did 2"*. Drop the axis and
`Σ share_a` is the only survivor — which is exactly what the product model already gave, so
the ratified change becomes unobservable in the record. It also forecloses the forward-note:
the present-tier `Actor` cause ("player P deposited it") is *the same axis*. Drop it and the
ledger cannot address present-tier provenance at all.

**Drop BOTH** (−869 MiB) — what remains is one number per (slot, edge): "this much of this
slot turned into that". §1's *"the fact is the seed; the story is derived-and-displayed"* has
no seed left. This is the summary-as-authority defect wearing a fact's paperwork; **A-1**.

**Which drop is cheaper in provenance terms will not be decided by bytes.** The chapter axis
is what the *story* is keyed off; the agent axis is what the *most recent ratification* is
about. They are not interchangeable, which is why both byte figures are given separately.

---

## 7. What this means for the design

**This section does not make the call.** For each option: what it costs, and which of the
arc's commitments it keeps or breaks.

**The measurement changed the shape of the problem in one specific way.** The two independent
encoding levers together take the baseline from 973 → **504 MiB** while keeping *every* axis
— but an edge id **alone buys literally nothing** (alignment padding), so "cheap encoding
wins" is a single combined decision, not two. And the f32 half costs one part in 914 000 of
an eighth of a voxel, five orders below anything a player can see. If narrowing is rejected,
it will be rejected for the nine test tolerances and the "a stored fact is exactly what
moved" property — not for accuracy the world can feel.

**Option 1 (baseline, 973 MiB)** keeps every commitment and breaks none. It is also the only
option with zero engineering cost. It costs a gigabyte of resident memory on a Medium world,
in a project whose standing doctrine is that **runtime is sacred** — and residency is a
runtime cost.

**Option 2 (encoding, 505–739 MiB)** keeps every commitment intact — chapter, agent, edge,
and a fraction accurate to 24 bits. It breaks nothing in the fact *shape*; the only thing it
gives up is the exactness of "a stored fact is bit-identical to what the edge moved", which
today nine assertions rely on. It is a **~2× reduction, not an order of magnitude** — it
makes the number smaller without changing its class. It composes with every other option.

**Option 3 (resident fold + paged facts, 37.86 MiB resident)** is the only option that
**keeps every commitment and changes the class of the number** — 55.8× becomes 2.17× of
today, while *nothing is deleted*, so "compaction must not mean deletion" is honoured
literally. It is also S-9 with the overlay on disk, which means it is the shape the project
already ratified rather than a new mechanism. Page-in is **p50 ~100–220 µs, p95 ~150–345 µs
cold** and **~4–5 µs warm** — for an inspect-one-voxel query that is 1–2 % of a 60 Hz frame
and faster than the chunk generation the same click may trigger; comfortably sub-frame and
sub-10 ms. **The price is § 5**, and § 5 is not small: a first filesystem dependency in a
headless crate (properly, an injected port), a versioned derived artifact with a validated
staleness path, a writer/reader split across process lifetimes, two artifacts rather than
one, and a new "provenance unavailable" answer. Each is ordinary engineering; together they
are a slice, not a tweak.

**Option 4 (axis-drops, 70–348 MiB)** is the only family that **breaks a commitment the arc
was built to keep**, and it breaks it irreversibly. Dropping chapter deletes the derived
story layer §1 specifies; dropping agent deletes the distinction §4 was ratified to create
one day earlier; dropping both produces a per-slot scalar, which is A-1 and the exact defect
this arc exists to kill. The byte savings are also *worse than option 3 in every case*: the
cheapest axis-drop (4c′, 70.05 MiB) is still nearly twice option 3's resident footprint, and
it has thrown away the provenance option 3 keeps. **On the numbers, no axis-drop dominates
paging.** If an axis is dropped it will be to avoid § 5's engineering cost, not to save
memory — that is the trade, stated plainly.

**Compositions worth having in view.** Option 3 + option 2c: **37.86 MiB resident, 468.91 MiB
on disk**, every axis retained, fraction to 24 bits. Option 4c′ (drop both + compact) is the
floor of the no-disk family at **70.05 MiB**, and it gives up the most.

**One residual the numbers do not settle.** All of option 3's 37.86 MiB is *addressing*, not
payload — 4.5 M CSR rows at 8 B. If the per-depth fold itself were coarsened (fewer, thicker
slots for the fold than for the facts), that number would fall further while the facts on
disk kept full resolution. Nobody asked for that and this spike did not price it; it is noted
so it is not lost.

---

## Appendix — reproducing

```
cargo run --release -p dc-worldgen --example s20_ledger_residency_probe
```

Prints all five sections at seed 1337 / `Extent::Medium`. The run takes ~90 s (26 s deep run,
then ~938 MiB written and re-read on each of two volumes; both files are deleted at the end).

The gate (`gate::the_s20_residency_model_is_self_consistent`) runs at `Extent::Small` and
asserts only **scale-free** properties: the residency itemisation reconstructs the record's
own measured `footprint_bytes()` byte-for-byte; the local `ShippedShape` still models `Fact`;
`EdgeIdShape` is still padded back to the shipped width; every axis-drop is a strict
reduction; the probe's fold agrees with `weathering_product_m`; the f32 error is non-zero and
more than three orders below an eighth; the paged prototype writes and reads back a file of
exactly the projected geometry. **No assertion is pinned to a MiB figure or a microsecond
count** — those are what `main` is for.

**Measured gate cost: +6.6 s** (`gate::the_s20_residency_model_is_self_consistent`), against
the 35.0 s the seven earlier converted probes add and the 8.8 s
`perdepth_weathering_cost_probe` adds. The gate's paged prototype writes a Small-extent file
(a few MiB) to the temp directory and deletes it.

Full `dc-worldgen` gate after `cargo clean -p dc-worldgen --release`: `fmt --check` clean,
`clippy --all-targets --release -D warnings` clean, **44 test binaries, all `ok`, 0 failed**.
