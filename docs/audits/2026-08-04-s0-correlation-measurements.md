# S0 — the stratigraphic-correlation pre-slice measurement run (M0′)

**Status: COMPLETE.** All seven measurements ran. Two of them **refute a claim in the
documents that asked for them** — recorded here, banners owed and listed, none stamped:

- **⚠ THE UNCONFORMITY GAP IS NOT MEASURABLE, AND STRUCTURALLY CANNOT BE** (§ 5.2). The
  shipped world carries **0** interior unconformity-flagged contacts and **114,945** flagged
  at the record base. `DeepStrata::stripped` is set *only when `units.is_empty()`*
  (`recorder.rs:1178-1181`), so a flagged unit is **always `units[0]`** — its predecessor
  was erased by the very strip the flag records. journal/0154's illustration (*"a unit at
  epoch 41 under a flagged contact, the next at 88 — 47 epochs stripped"*) describes a
  record state the recorder cannot produce.
- **⚠ `column()` IS 1.62 ms/chunk, NOT ≈14 ms** (§ 2). The correlation design's F4 derived
  ≈14 ms by hand from M0's ratio and flagged it as arithmetic. Measured, it is **8.6×
  smaller** — which makes every *share-of-`column()`* claim in § 6 of that design 8.6× more
  expensive than it reads, including the one that concluded *"the blend itself cannot be the
  fight."*

**What this is.** `docs/audits/2026-08-03-stratigraphic-correlation-design.md` § 6 names
M0′ as the pre-slice measurement run the build owes before S1 wiring, and § 9b lists it as
integrator-settleable item **I-1**. This file is that run, **updated for two things that
landed after the design pass was written**:

1. **The deposition clock shipped** (journal/0154, `2026-08-04-deposition-clock-design.md`).
   Every `DepUnit` now carries a raw **epoch** byte. Correlation matches **epoch
   intervals**, not chapter fractions — so the design's measurement (c) (*union of
   fractional breakpoints under R-C*) is **superseded** by epoch-interval counts, measured
   here. R-C survives only as the within-interval interpolant.
2. **P-1 ruled M-C** (2026-08-04, user): mixture grading for **continuum** pairs, the
   octaves cut for **discrete** pairs, with the continuum predicate **derived** from the
   FS-A release spectra + property sheet, never hand-authored. So measurement (d) (the
   mixture-cap check) is live rather than conditional on M-A.

**Immutable body, mutable header** (CLAUDE.md read-first item 5). Anything that later
refutes or re-scopes a number here gets a banner **at the top of this file**, stamped by
the author of the correction.

---

## 0. Provenance

| field | value |
|---|---|
| base commit | `4ddb472` — *five roster dispositions ratified + folded* (worktree merged clean; `git log -1 main` identical) |
| branch | `worktree-agent-a5705acce6de2077d` |
| seed | **1337** |
| extent | **`Extent::Medium`** (545² = 297,025 deep cells, ~460 m pitch, 250.7 km world) |
| config | production (`Pregen::run` → `build_field_with(.., DeepOverrides::default())`) |
| instruments | `crates/dc-worldgen/examples/correlation_m0_probe.rs` (new, this run) · `crates/dc-worldgen/src/collapse.rs::m0_run_strata_share_of_column` (extended) · `crates/dc-worldgen/examples/member_diversity_probe.rs` (unchanged, re-run) |
| build rules | `CARGO_BUILD_JOBS=4`, shared `CARGO_TARGET_DIR`, `--release` |

**Commands.**

```text
cargo run  --release -p dc-worldgen --example correlation_m0_probe          # (a) (c) (d) (e) (g)
cargo test --release -p dc-worldgen --lib m0_run_strata -- --ignored --nocapture   # (b)
cargo test --release -p dc-worldgen --test contents_contract                # (f)
cargo run  --release -p dc-worldgen --example member_diversity_probe        # (g) cross-check
```

Run wall-clock, for whoever repeats this: probe **42.7 s** (42.24 s of it `Pregen::run`) ·
M0 harness **41.6 s** · `contents_contract` **137.3 s** · `member_diversity_probe` **~3 min**
(it pays `Pregen::run` *and* a second `build_field` *and* an identity-audit `run_cells`).
Console output was Tee'd to the session scratchpad, not committed — every figure quoted
below is reproduced verbatim in the tables and code blocks of its own section, per
*read the log the gate wrote*.

---

## 1. (a) Per-cell H histogram + max stack depth

**Method.** One pass over `DeepField::strata` (297,025 cells). *Stack depth* is
`DeepStrata::units.len()`; *H* is `Σ unit.thickness_m()`, which the journal/0053 finalize
invariant makes identical to `DeepField::regolith` — the probe reports the max absolute
disagreement as a self-check on that identity. *Events per cell* is a different quantity
(it is a per-**column** expression product, not a record product) and is measured in § 2
with the fill, where it belongs.

### 1.1 The numbers

`Pregen::run` **42.24 s** · deep grid 545×545 = **297,025 cells** · cell **460.0 m** ·
`DeepField` resident **147.80 MiB** · **248,322 cells (83.6 %) carry a record**.

| distribution | n | mean | median | p95 | max | min |
|---|---:|---:|---:|---:|---:|---:|
| **stack depth**, all cells | 297,025 | **24.592** | 14 | 104 | **460** | 0 |
| **stack depth**, recorded cells | 248,322 | 29.416 | 17 | 117 | **460** | 1 |
| **H** = Σ unit thickness, all cells | 297,025 | **1.377 m** | 0.596 m | 5.116 m | **539.358 m** | 0 |
| **H**, recorded cells | 248,322 | 1.647 m | 0.727 m | 5.730 m | 539.358 m | 0.001 m |

- **max single unit thickness 62.936 m** (was 65.38 m at slice-3 M0 — the u24 fixed-point
  cap of 16,384 m keeps its ~260× headroom).
- **`max |DeepField::regolith − Σ units| = 4.8828 × 10⁻⁴ m`** — that is **exactly q/2** for
  `q = 2⁻¹⁰ m`. The journal/0053 finalize invariant and the slice-3 sub-quantum carry bound
  (`|carry| ≤ q/2`) are the *same* number, confirmed at world scale rather than in a unit
  test. **This is the strongest single evidence that F2's linearity argument is safe**: the
  record's thickness sum and `H` never disagree by more than one carry.
- **The stack-depth tail is much longer than the mean advertises.** The design reasoned from
  *"mean ~25 units/cell"*; the distribution is heavily right-skewed — median **14**, p95
  **104**, max **460**. Cells with 380–460 units exist (single-digit counts each). Empty
  stacks: **48,703 cells** (16.4 %).

**What the tail does to the rejected R-B.** Needleman–Wunsch is `O(m·n)`; at the mean that
is 25² ≈ 625, at p95 104² ≈ 10,816, and at the tail 460² = **211,600 cells per pair per
chapter** — a 340× spread. R-B was already rejected on correctness (§ 1.2 of the design);
this prices how badly its worst case would have behaved, and the answer is *much* worse
than the mean suggested.

## 2. (b) `ColumnFill::build` share of `column()` — the F4 decider

**Method.** The shipped 200-chunk M0 harness (`m0_run_strata_share_of_column`, added by
P11 slice 3), extended here with three rows the F4 question needs:

- `StrataRec::events.len()` per chunk (mean/median/p95/max) — the interval count the fill
  actually slices, today;
- `ColumnFill::depth_count()` and `heap_bytes()` per fill;
- **arm 3, the naive per-column projection**: rebuild the fill **1024×** from the same
  record over a 25-chunk subsample. Today one fill per `SubCell` serves all its columns
  (`subcell.rs:44-46`); a per-column thickness vector needs per-column fill state, so the
  naive cost is the *upper bound* mitigation (i)/(ii) must beat. Measuring it directly —
  rather than multiplying a ratio — is what makes the I-5 pick evidence.

### 2.1 The numbers

```
M0 run_strata share: 200 chunks | column() total 0.323 s (1.62 ms/chunk)
  | run_strata alone 0.001 s (0.00 ms/chunk, 0.3 % of column)
  | ColumnFill::build 0.001 s (0.2 %)
M0' (b) fill shape: 200 fills | events/chunk mean 90.61 median 86 p95 180 max 236
  | fill depth_count mean 104.84 median 104 p95 112 max 116 | fill heap mean 2912 B
M0' (b) per-column projection: ColumnFill::build 3.03 us/call
  | naive 1024 rebuilds 1.69 ms/chunk over 25 chunks (104.8 % of column()'s 1.62 ms/chunk)
```

| quantity | measured |
|---|---|
| **`column()` (cold `column_record`)** | **1.62 ms/chunk** |
| `run_strata` alone | ~5 µs/call, **0.3 %** of `column()` |
| `ColumnFill::build` | **3.03 µs/call**, **0.2 %** of `column()` (one fill per chunk today) |
| `StrataRec::events` per chunk | mean **90.61**, median 86, p95 **180**, max 236 |
| `ColumnFill::depth_count` | mean **104.84**, median 104, p95 112, max 116 voxels |
| `ColumnFill` heap | mean **2,912 B** per fill |
| **naive per-column fill (1024 rebuilds/chunk)** | **1.69 ms/chunk = +104.8 % of `column()`** |

### 2.2 ⚠ `column()` is 1.62 ms/chunk, not ≈14 ms

The design's F4 wrote *"≈14 ms/chunk by that ratio — hand arithmetic, flagged"*, and § 1.4
priced D-1 against it (*"1024 × 7 µs ≈ 7 ms/chunk added (≈ 50 % of `column()`)"*). Measured,
`column()` is **1.62 ms**. So:

- **D-1 (blend `DeepStrata`, then `run_strata` per column) is worse than it looked, not
  better**: 1024 × 5 µs ≈ 5.1 ms against a 1.62 ms budget — **+315 %**, not +50 %. The
  design's rejection of D-1 stands and hardens.
- Every other *share-of-`column()`* figure in that § 6 is likewise **8.6× larger** than it
  reads.

### 2.3 The I-5 RECOMMENDATION — **(ii) lazy per-column evaluation inside the voxel walk**

Two independent arguments, both from the numbers above.

**Wall clock.** The naive materialization costs **+104.8 %** — it roughly *doubles* the
chunk collapse. That is not noise and it is not a shrug; against *runtime is sacred* it
disqualifies (iii)-free naivety and makes the mitigation pick load-bearing rather than
cosmetic. Mitigation **(i)** (shared interval list + per-column thickness vector, prefix
sums) still *materializes* a `Plan` vector per column and so keeps most of that cost;
mitigation **(ii)** does the same arithmetic inside a walk that already visits spans in
depth order and never materializes the vector at all.

**Memory, and this is the sharper argument.** Today the record-side transient is **one**
`ColumnFill` per `SubCell` — mean **2,912 B**, ≤ 9 per chunk. Per-column state means 1024 of
them: **≈ 2.98 MB per chunk** of transient — and `column_cache` is **runtime-resident**
(`subcell.rs` heap-bytes doc, the design's own RETURN spec). Add the correlation thickness
vector itself at § 3's measured widths (mean 44.9 f64 = 359 B/column, p95 145 = 1,160 B,
max 200 = 1,600 B) and materializing per column adds a further **368 KB mean / 1.19 MB p95
/ 1.64 MB max per chunk**. Lazy evaluation carries **one** interval vector for the chunk and
a running dot product per column — it removes both.

**(iii) — the weight-bucket cache — is NOT needed and should not be reached for.** It buys
back a cost that (ii) does not pay, and it reintroduces exactly the quantized steps this
whole arc exists to kill (the design flags it *"only as a fallback, loudly"*). Nothing
measured here creates a case for it.

*Caveat, stated rather than smoothed:* (ii)'s own cost cannot be measured before it exists
(§ 9.3). What is measured is the **ceiling** it must beat (+104.8 %) and the allocation it
avoids (≈3 MB/chunk). Both point the same way.

## 3. (c) Shared-partition interval counts under EPOCH correlation

**Method — and why it supersedes the design's (c).** Under the clock, the shared partition
over a stencil is the **union of the epochs its parents carry**: each `DepUnit` carries one
epoch, epochs are monotone up-stack, so a parent's stack partitions into epoch-runs and a
parent contributes zero thickness to an epoch it does not hold. The union count therefore
*is* the per-column dot-product length. Measured as a 256-bit presence bitset per cell
(`DEEP_ITERATIONS = 200`, so 256 bits covers the axis exactly), OR-ed over every 2×2
stencil (the interior case, 544² stencils) and every 3×3 stencil (the straddling case,
543²).

**What an "interval" is, precisely.** A `DepUnit` is a **merged run**, and merge keeps the
**bottom** epoch (`recorder.rs::add_quanta`, journal/0154). So the epochs a cell *stamps*
are the run-start ticks, not every tick it deposited in — which is exactly the boundary set
correlation needs, and the reason the count below is the partition's true width rather than
a proxy for it. A stable environment holding epochs 37..41 in one unit contributes **one**
boundary, at 37, in every parent that shares that stability.

### 3.1 The numbers

| stencil | n | mean | median | p95 | max | min |
|---|---:|---:|---:|---:|---:|---:|
| **2×2** (interior), all | 295,936 | **44.936** | 36 | **145** | 200 | 0 |
| **2×2**, non-empty | 248,454 | 53.524 | 42 | 153 | 200 | 1 |
| **3×3** (straddling), all | 294,849 | **61.731** | 52 | **183** | 200 | 0 |
| **3×3**, non-empty | 248,551 | 73.230 | 60 | 188 | 200 | 1 |

Single-cell floor, for scale (§ 5): distinct epochs per **recorded cell** mean **22.968**,
median 17, p95 70, max 200. A 2×2 stencil roughly **doubles** the partition width (23 → 54);
a 3×3 roughly triples it (23 → 73). The world's epoch axis is 200 ticks
(`DEEP_ITERATIONS = 200`) and the union saturates at 200 in the deep basins.

### 3.2 The per-column dot-product constant, and what it costs

The design's § 6 estimated *"a union of ~10² intervals … ~10⁵ multiply-adds per 1024-column
chunk, microseconds"*. The estimate's **width** was right (mean 45, p95 145 — the same order
as 10²). Its **conclusion** does not survive § 2.2's correction to `column()`:

| | intervals | multiply-adds per chunk (× 4 parents × 1024 columns) |
|---|---:|---:|
| mean | 44.9 | **184,000** |
| p95 | 145 | **594,000** |
| max | 200 | **819,000** |

At a realistic 1–4 GFLOP/s for a strided FMA over f64 vectors, that is **≈0.05–0.6 ms per
chunk against a 1.62 ms budget — 3 % to 37 %.** *(Arithmetic, flagged as such: no
correlation kernel exists to time. The FLOP count is measured; the rate is assumed.)*

**So "the blend itself cannot be the fight" is too strong.** It is not the *dominant* cost —
the per-column fill state is, at +104.8 % — but at the p95 tail it is a double-digit
percentage of the chunk budget and it belongs in the S1 slice's RETURN spec as a measured
line, not as a dismissal. **This is the number that makes I-5's (ii) a requirement rather
than a preference**: the dot product is affordable, materializing 1024 fills around it is
not.

## 4. (d) Distinct blended mixtures per continuum-pair transition zone

**Method.** Per 2×2 stencil and per correlated epoch interval, the union of the species the
four parents present, as a bitmask over `MaterialId`. **k = |set|** is the mixture cap's k
(the combinatorial doctrine's `C(k+8,8) − 1` bound over a locale mixing k materials); the
number of distinct mixtures a k-set can express **at the fill's own quantum** — eight
eighths, `fill.rs::fixed_weights` — is the count of compositions of 8 into k positive
parts, `C(7, k−1)`: **7 for a pair, 21 for a triple, 35 for a quadruple**.

**The continuum predicate is a PROXY here, and that is flagged.** P-1 ruled the real
predicate is *derived from the FS-A release spectra + property sheet* — which is S1/S2
work. This probe uses **same-`Litho`-class** as the measurable stand-in (clastic-fine =
{mudstone, siltstone}; clastic-coarse = {sandstone, conglomerate}), which is the set the
brief names. A derived predicate can only ever *narrow* the continuum set relative to
same-class, so the mixture count below is an **upper bound** under M-C.

### 4.1 The numbers

**13,298,244 stencil-intervals scanned** over 295,936 2×2 stencils; **6,231,820 (46.86 %)
carry more than one species.**

| k = distinct species in a stencil-interval | count | share | eighth-states `C(7, k−1)` | combinatorial cap `C(k+8,8) − 1` |
|---:|---:|---:|---:|---:|
| 1 | 7,066,424 | 53.138 % | 1 | 8 |
| **2** | **4,695,703** | **35.311 %** | 7 | 44 |
| 3 | 1,216,817 | 9.150 % | 21 | 164 |
| 4 | 288,101 | 2.166 % | 35 | 494 |
| 5 | 30,992 | 0.233 % | 35 | 1,286 |
| **6 (max observed)** | 207 | 0.002 % | 21 | **3,002** |

**Distinct species SETS with k > 1: 98.** Split by the same-class proxy:

| | sets | mintable mixtures at the eighths quantum |
|---|---:|---:|
| **CONTINUUM** (M-C's mixture branch) | **2** | **14** |
| **DISCRETE** (M-C's octaves cut — no mixture minted) | **96** | — |

The two continuum sets are exactly the roster's two multi-member depositional classes:

| set | class | occurrences | states |
|---|---|---:|---:|
| `dc:sandstone + dc:conglomerate` | clastic-coarse | **2,762,133** | 7 |
| `dc:mudstone + dc:siltstone` | clastic-fine | **111,112** | 7 |

By occurrence, continuum intervals are **2,873,245 of 6,231,820 multi-species intervals
(46.1 %)**; the octaves cut owns the other **53.9 %**.

### 4.2 THE MIXTURE-CAP CHECK PASSES, WITH ENORMOUS MARGIN

**14 distinct blended mixtures, world-wide, upper bound.** Not 14 per transition zone — 14
in total, because the mixture branch can only ever fire on the **two** same-class pairs the
vanilla roster contains, and each pair admits 7 interior states at the fill's own eighths
quantum. Against the combinatorial doctrine's `C(k+8,8) − 1` = **3,002** at the observed
max k = 6, and against a mixture table the world already populates from every other source,
**M-C's mixture branch is a rounding error on the palette.**

Three consequences the build should carry:

1. **P-1's M-A worry about palette growth is empirically dead.** The design flagged it as
   *"⚠ unmeasured how many distinct blended mixtures a transition zone mints"* (§ 3.2,
   § 10.6). Measured: at most 14. **Discharged.**
2. **M-C's *discrete* branch is the one carrying the load** — 96 of 98 sets, 53.9 % of
   multi-species intervals. **This sizes I-4** (the new registered draw domain): it is
   exercised at over half of all multi-species correlated intervals, on 96 distinct species
   combinations up to k = 6. The design's note that *"vanilla exercises [the discrete branch]
   at every clastic-igneous and clastic-organic contact"* is confirmed and understated — the
   single most common **discrete** set is `dc:sandstone + dc:carbonaceous-mudstone` at
   **747,413** occurrences, a clastic-against-soil contact.
3. **A caution the numbers force.** The commonest set of all is
   `sandstone + conglomerate` (2.76 M) — which the same-class proxy calls continuum. If the
   derived FS-A predicate later says sandstone↔conglomerate is **not** a continuum (it is a
   clast-size step, not a smooth silt/clay ratio), the mixture branch collapses to
   `mudstone + siltstone` alone — **7 mixtures, 111,112 intervals, 1.8 % of the multi-species
   population** — and M-C becomes, in practice, "the octaves cut, with one small exception".
   That is a real possibility this run cannot settle (§ 9.1) and the S2 slice should not be
   surprised by it.

## 5. (e) Epoch statistics

**Method.** Per cell: distinct epochs present (the same bitset as § 3), distinct chapters
for comparison, and the record-wide epoch range. At every unit whose `unconformity()` flag
is set and which has a predecessor in its own stack, the **gap** is
`epoch(k) − epoch(k−1)`. Flagged units at the record base have no predecessor and no
defined gap; they are counted separately rather than folded in as zero. The probe also
counts epoch **non-monotonicity** up-stack — which must be 0, and is the field-scale
version of the debug assert journal/0154 landed.

### 5.1 The numbers

- **Epoch range present in the record: 0 ..= 199.** Every one of the 200 iterations
  (`DEEP_ITERATIONS`) has surviving rock somewhere on the world.
- **Epoch NON-monotone up-stack: 0.** Across all 7,304,581 units. The correlation design's
  § 10.4 owed assert and journal/0154's debug assert are both confirmed at world scale, in
  release, where `debug_assert!` does not run.

| distribution | n | mean | median | p95 | max | min |
|---|---:|---:|---:|---:|---:|---:|
| distinct **epochs** / cell, all | 297,025 | 19.202 | 14 | 64 | 200 | 0 |
| distinct **epochs** / recorded cell | 248,322 | **22.968** | 17 | 70 | 200 | 1 |
| distinct **chapters** / recorded cell | 248,322 | **5.442** | 6 | 8 | **8** | 1 |

**The clock is ~4.2× finer than the chapters it replaced** (22.97 epochs against 5.44
chapters per recorded cell). That ratio is the concrete value of O-2b: correlation now has
23 anchors per borehole where it had 5, and R-C's proportional guessing shrinks to the gap
*between* them (§ 8).

### 5.2 ⚠ THE UNCONFORMITY GAP HAS ZERO INSTANCES, AND CANNOT HAVE ANY

```
unconformity-flagged contacts .... 0 interior + 114945 at the record base
```

**114,945 flagged units — every single one is `units[0]`.** Not one flagged contact in the
world has a predecessor in its own record, so **no epoch gap is defined anywhere**, and the
"measurable gap" figure this section was asked to produce **does not exist**.

**This is structural, not a property of seed 1337.** `DeepStrata::stripped` is set in
exactly one place — `erode`, and only `if self.units.is_empty()` (`recorder.rs:1178-1181`).
The flag therefore means *"the record was stripped to bedrock"*, and the unit that carries
it is by construction the first unit of a record that was empty a moment earlier. **The
strata whose deposition time would define the gap were deleted by the very event the flag
records.** A flagged unit can never migrate off index 0: later deposits push above it, and
merge-down (`:1142-1148`) pops the *upper* unit into the survivor.

**What this refutes, and what it does not.**

- It **refutes** journal/0154's illustration — *"a unit at epoch 41 under a flagged contact,
  the next at 88 — 47 epochs stripped"* — as a description of the shipped recorder. If the
  column was stripped to bedrock, the epoch-41 unit is gone.
- It **refutes** the correlation design's R-C′ premise (§ 1.2) that *"where both parents
  carry [a flag] at the same chapter transition"* an anchor can be built. There are no
  interior flags to pair. **R-C′ is moot on two independent grounds now** — the clock
  superseded it, and its input never existed.
- It does **not** refute the clock. Epochs are real, dense (§ 5.1) and monotone; correlation
  matching true epoch intervals is unaffected.
- It does **not** mean the world has no unconformities. **46.3 % of recorded cells
  (114,945 / 248,322) sit on one** — the record-to-basement contact is an erosional surface
  almost half the time. That is a large, real, expressible fact; what is missing is only the
  *duration*, because the record above bedrock is the only record there is.

**What would make the gap measurable** (recorded, not proposed as work): the strip event
would have to leave a trace — the epoch at which the column was last stripped, or the epoch
of the youngest unit it destroyed. `DeepStrata::strips` counts strips (`:886-888`) but
records no time. That is a recorder-side widening and therefore outside this arc, which is
read-side only (design F3). **Flagged for whoever owns the gap-semantics claim.**

## 6. (f) `contents_contract` re-baseline

**Method.** `cargo test --release -p dc-worldgen --test contents_contract`, wall-clock of
the whole suite. The 102.43 s figure is journal/0129's machine-state (2026-07-26) and
predates P11 slices 1–3, the packed `DepUnit`, the near-path restructure and the
deposition clock.

### 6.1 The number

```
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 137.31s
```

| | wall clock | source |
|---|---:|---|
| 2026-07-26 (journal/0129, post-U3) | 102.43 s | journal/0129:202 |
| **2026-08-04 (this run)** | **137.31 s** | `contents_contract-199b58089a8d7d05.exe` |

**+34.88 s, +34.1 %** over nine days. The suite's shape did not change (3 tests, same
sample); what grew is the world it generates — P11 slices 1–3 (member identity in the record,
the CSR-sparse budget planes, the near-path restructure with `run_strata` per touched cell)
and the deposition clock. **This 137.31 s is the denominator the correlation build's RETURN
spec reports against**; the design's citation of 102.43 s is retired.

## 7. (g) THE UNIT-COUNT HYPOTHESIS

**The question** (ROADMAP § Observed, *"ONE COUNT, TWO VALUES"*): journal/0136:280 says
**10,951,030** recorded units; `2026-08-03-stratigraphic-correlation-design.md` F1 says
**~7.4 M** over the same 297,025 cells — a 48 % gap with no pointer either way. The
dispatch hypothesis: the older count predates slice 3's sub-quantum carry.

**Method.** Two independent instruments on the current tree: the new probe's direct sum
over `pregen.deep.strata` (the shipped world the game loads), and `member_diversity_probe`
re-run unchanged — the very instrument that produced 10,951,030.

### 7.1 The corpus archaeology (desk work, no cargo — done before the run)

The two disputed figures are not the only ones. **There is a third, and it dates the drop.**

| count | artifact | instrument | landed |
|---:|---|---|---|
| 10,951,030 | journal/0136:280, :307 · corrections #88 (`:3621`) | `member_diversity_probe` | commit `3cab931`, **2026-08-02** (P11 **slice 1**) |
| **7,622,541** | **journal/0141:188** · `2026-07-29-fluvial-record-terms-priors.md:46` | `member_diversity_probe` | commit `81d8d3b`, **2026-08-02** (P11 **slice 2b**) |
| 7,363,947 | journal/0145:9 · `2026-08-02-p11-slice3-design.md:60` | the recorder's M0 mover-split counter | commit `ee2f800`, **2026-08-03** (P11 **slice 3**) |

**This refines the hypothesis both the dispatching brief and the clock audit's F6 state.**
F6 (`2026-08-04-deposition-clock-design.md:198-210`) attributes the whole 48 % gap to
**slice 3's sub-quantum carry** and offers exactly the falsifier this section runs. The
intermediate figure says the carry cannot be the bulk of it: the count had already fallen
to 7,622,541 **one slice earlier**, before the carry existed. The split is

- **slice 2 (journal/0141, ruling 6 — identity comes from the arriving composition rather
  than a per-deposit draw): 10,951,030 → 7,622,541, −30.4 %.** Transported identity is
  *more coherent bed-to-bed* than an independent draw at every deposit, so `deposit_as`'s
  merge key splits **less** often. The same entry records the split factor moving
  2.4053× → 1.9064× against a pre-P11 baseline that itself moved (4,552,847 → 3,998,428).
- **slice 3 (journal/0145 — the sub-quantum carry): 7,622,541 → 7,363,947, −3.4 %.** Real,
  and the direction F6 predicts, but ~1/9th of the gap.

F6's **conclusion** stands (the two docs were out of order, not in conflict, and ~7.4 M is
current); its **mechanism** is only the small half. Both are recorded so the correction is
a refinement, not a reversal.

### 7.2 The owed-banner list — LISTED, NOT STAMPED

Per read-first item 5, the obligation lands on the **writer of the correction**, in the
same commit. This run is a measurement agent in a worktree; **the integrator stamps at
merge**. Nothing below has been edited by this run.

| artifact | the stale claim | what the banner should say |
|---|---|---|
| **journal/0136:280, :307** | *"10,951,030 recorded units"*; *"the merge-key split factor is 2.4053×"* | superseded at P11 slice 2b (journal/0141) and again at slice 3; the count is now § 7.3's figure. The 2.4053× split factor and the derived **+97.6 MiB** both move with it. **The ROADMAP entry names this artifact by name as owed a banner.** |
| **journal/corrections.md #88** (`:3621`) | restates *"2.4053× (10,951,030 vs 4,552,847)"* as the priced correction | same supersession; the *mechanism* of #88 (a per-entry cost statement is not an aggregate one when the change touches the merge key) is **unaffected and still right** — only its magnitude moved |
| **`docs/audits/2026-08-03-stratigraphic-correlation-design.md` F1** (`:136`) | *"~7.4 M units … mean ~25 units/cell, mean unit 0.053 m"* | the current measured count, and F1's *"⚠ max stack depth and the per-cell H distribution are unmeasured"* is **discharged by this file** |
| **`docs/audits/2026-08-04-deposition-clock-design.md` F6** (`:194-210`) | the hypothesis that slice 3's sub-quantum carry explains the gap | **conclusion confirmed, mechanism refined**: the carry is −3.4 %, slice 2's identity-from-arriving-composition is −30.4 % (§ 7.1). Its falsifier ran; this file is the answer |
| **journal/0141:188** | *"7,622,541 units … split factor 1.2775× → 1.9064×"* | superseded by slice 3 + this run; listed because it is the figure that dates the drop and neither disputing doc cites it |
| **`docs/audits/2026-07-29-fluvial-record-terms-priors.md:46`** | *"the unit split 1.9064× (3,998,428 → 7,622,541 units)"* | same |
| **`ROADMAP.md` § Observed, *"ONE COUNT, TWO VALUES"*** | the open question | **RESOLVED** — move to the resolved/close-block treatment with a pointer here (and per the close-block rule, the resolution owes the asking documents their banners in the same commit, which is this table) |

### 7.3 The measured answer — **7,304,581 units**

**Two independent instruments, byte-for-byte agreement.**

| instrument | reads | figure |
|---|---|---:|
| `correlation_m0_probe` (new) | `pregen.deep.strata` — the field `Pregen::run` distils and the game loads | **7,304,581** |
| `member_diversity_probe` (unchanged — *the instrument that produced 10,951,030*) | its own `build_field(cells, SEED)` run | **7,304,581** |

`member_diversity_probe` also reports **409,070.4 m** recorded (⇒ mean unit **0.056 m**),
split factor **1.9314×** against a 3,782,091-unit class-only counterfactual, and max unit
**62.936 m** — all consistent with the new probe's independent scan.

**VERDICT.** The 10,951,030 figure is **stale, not wrong** — it was correct for P11 slice 1
and has been superseded twice. The current count is **7,304,581**, a further **−0.8 %**
below slice-3's 7,363,947 (the FS-A wire-up and the deposition clock landed in between). The
full history:

| stage | units | Δ |
|---|---:|---:|
| P11 slice 1 (journal/0136) | 10,951,030 | — |
| P11 slice 2b (journal/0141) — identity from the arriving composition | 7,622,541 | **−30.4 %** |
| P11 slice 3 (journal/0145) — the sub-quantum carry | 7,363,947 | −3.4 % |
| **today (this run)** | **7,304,581** | −0.8 % |

**ROADMAP § Observed *"ONE COUNT, TWO VALUES"* is RESOLVED.** The two values were never in
conflict; they were fourteen commits apart, and the dominant mechanism was slice 2's, not
slice 3's (§ 7.1).

## 8. What each number decides

### 8.0 The headline answers

| question | answer |
|---|---|
| **I-5 — fill-state mitigation** | **(ii), lazy per-column evaluation inside the voxel walk.** Naive materialization costs **+104.8 %** of `column()` and ≈**3 MB/chunk** of transient in a runtime-resident cache; (i) keeps most of both; (iii) is unnecessary (§ 2.3) |
| **the dot-product constant** | **mean 44.9 intervals per 2×2 stencil** (p95 145, max 200) ⇒ **184 k–819 k multiply-adds/chunk**, ≈3–37 % of a 1.62 ms budget (§ 3.2) |
| **the mixture cap under M-C** | **PASSES with enormous margin — 14 mixtures world-wide, upper bound**, against a cap of 3,002 at the observed max k = 6. The discrete branch carries 96 of 98 sets (§ 4.2) |
| **the unit count** | **7,304,581**, two instruments agreeing exactly; the dispute is closed (§ 7.3) |
| **`contents_contract` baseline** | **137.31 s** (was 102.43 s) (§ 6.1) |
| **the unconformity gap** | **⚠ CANNOT-DETERMINE, structurally — 0 interior flagged contacts** (§ 5.2) |

### 8.1 Banners owed by THIS file's two refutations — LISTED, NOT STAMPED

Same discipline as § 7.2: the integrator stamps at merge.

| artifact | the claim | what the banner should say |
|---|---|---|
| `2026-08-03-stratigraphic-correlation-design.md` **F4** (`:171-179`) and **§ 1.4 D-1** | *"≈14 ms/chunk by that ratio — hand arithmetic, flagged"*; *"1024 × 7 µs ≈ 7 ms/chunk added (≈ 50 % of `column()`)"* | measured: `column()` is **1.62 ms/chunk**; D-1's added cost is **+315 %**, not +50 %; every share-of-`column()` figure in its § 6 reads 8.6× low. The *rejection* of D-1 is unaffected and strengthened |
| `2026-08-03-stratigraphic-correlation-design.md` **§ 1.2 R-C′** (`:271-281`) | the unconformity-anchor refinement | its input does not exist: **0 interior flagged contacts, structurally** (§ 5.2). R-C′ is moot on two grounds |
| `2026-08-03-stratigraphic-correlation-design.md` **§ 3.2 / § 10.6** | *"⚠ unmeasured how many distinct blended mixtures a transition zone mints"* | **discharged: 14, world-wide** (§ 4.2) |
| `2026-08-03-stratigraphic-correlation-design.md` **F1 / § 10.2** | *"⚠ max stack depth and the per-cell H distribution are unmeasured"* | **discharged** (§ 1.1); the tail is far longer than the mean advertised (median 14, p95 104, max 460) |
| **journal/0154** (the deposition-clock entry) | *"the unconformity flag returns to GAP SEMANTICS with the gap now measurable (a unit at epoch 41 under a flagged contact, the next at 88 — 47 epochs stripped)"* | the recorder cannot produce that state: `stripped` is set only on a **full** strip, so a flagged unit is always `units[0]` and its predecessor was deleted (§ 5.2). The clock itself is unaffected |
| `2026-08-04-deposition-clock-design.md` (header + the P-3 resolution it drove) | the same gap-semantics claim, restated | same |
| `2026-08-03-stratigraphic-correlation-design.md` **§ 6 RETURN spec** | *"the 102.43 s figure is nine days old"* | **re-baselined: 137.31 s** (§ 6.1) |

| measurement | the decision it feeds | where the decision lives |
|---|---|---|
| (a) stack depth p95/max | R-B's rejection cost (`O(m·n)` alignment at the tail, not the mean) — already rejected on correctness, now also priced; and the transient size of the per-column thickness vector | design § 1.2, § 6.2 |
| (a) H distribution | how many voxels a correlated interval can actually express (mean H ≈ 1.5 voxels; the tails are what read at a bench cut) | design § 6.2, § 2.2.4 |
| (b) `ColumnFill::build` µs/call + the naive 1024× projection | **I-5 — the fill-state mitigation pick**: (i) shared interval list + per-column thickness vector, (ii) lazy per-column evaluation inside the voxel walk, (iii) weight-bucket cache *(fallback only, loudly — it reintroduces steps)* | design § 6.1, § 9b I-5 |
| (c) 2×2 / 3×3 union interval count | **the per-column dot-product constant** — the design estimated "~10² intervals"; this replaces the estimate. Also sets the per-column vector width and hence the transient memory line in the build slice's RETURN spec | design § 6 (the ~10⁵ multiply-add estimate), § 6.2 |
| (d) k-histogram + continuum species-set inventory | **the mixture-cap check under P-1's M-C**: how many distinct blended mixtures the mixture branch can mint, and how many *discrete* sets the octaves cut must dress (which sizes **I-4**, the new draw domain) | design § 3.2 M-A/M-C, § 9b I-4, § 10.6 |
| (e) distinct epochs/cell | the same quantity as (c) at stencil size 1 — the floor the union can never go below, and the first evidence that epoch intervals are a *finer* clock than the 8 chapters | clock audit § 5 |
| (e) epoch gap at unconformities | the **measurable-gap claim's first numbers** (journal/0154: *"a unit at epoch 41 under a flagged contact, the next at 88 — 47 epochs stripped"*) — and whether a flagged contact reliably carries a gap at all | clock audit header, design § 1.2 R-C′ |
| (f) `contents_contract` wall-clock | the **denominator of the build slice's RETURN spec** — the before/after the correlation wiring is measured against | design § 6 RETURN spec |
| (g) unit count | closes ROADMAP § Observed *"ONE COUNT, TWO VALUES"*; the banner list is § 7.2 | ROADMAP § Observed |

## 9. CANNOT-DETERMINE

First-class, not smoothed. Each says what would answer it.

1. **The real continuum predicate.** P-1 ruled it **derived** from the FS-A release spectra
   + property sheet. FS-A has not written the grain bits yet (`GRAIN_UNSET`,
   `recorder.rs`), so the predicate does not exist to evaluate. § 4 measures with
   **same-`Litho`-class** as a stand-in, which can only over-count the continuum branch.
   *Answered by:* the first slice that lands the derived predicate re-running § 4's probe.
2. **The realized mixture count, as opposed to its bound.** § 4 counts the distinct species
   *sets* a correlated interval presents and the eighth-states each set can express. Which
   of those states the blend actually visits depends on the weight field the S1 kernel
   produces, which does not exist yet. The figure is therefore an **upper bound**.
   *Answered by:* re-running the count against the S1 kernel's realized weights.
3. **Mitigation (i)'s and (ii)'s actual cost.** § 2 arm 3 measures the **naive** per-column
   rebuild — the ceiling. A prefix-sum fill over a shared interval list (i) and a lazy
   in-walk evaluation (ii) are *cheaper than* that by construction, but by how much cannot
   be measured before they are written. The naive number is what makes "(i) or (ii)" a
   pick rather than a hope, and it is what bounds the risk if both disappoint.
   *Answered by:* S1 reporting its own before/after on this same harness.
4. **Whether the epoch partition holds up per-column rather than per-cell.** Everything in
   § 3 is measured over deep cells. The expression tier interpolates a per-column stencil;
   whether the union widens once weights enter is an S1 property, not a record property.
5. **Nothing here touches the P-1 *identity* question at a bench cut.** These are desk
   numbers about counts and costs. The design's own § 10.5 — *is R-C's position-not-identity
   weakness visible at 460 m spacing* — is now largely moot (epochs replaced fractions), but
   the M-A-vs-M-B *appearance* question is a walk question and stays one (S3).
6. **⚠ THE UNCONFORMITY GAP — CANNOT-DETERMINE, AND THE REASON IS STRUCTURAL** (§ 5.2). The
   measurement (e) asked for is not merely absent from this world; the recorder cannot
   produce it. 0 interior flagged contacts against 114,945 basal ones.
   *Answered by:* a recorder-side widening that logs *when* a strip happened (`strips` counts
   them and records no time) — out of this arc's read-side scope (design F3).
7. **The FLOP-to-milliseconds conversion in § 3.2 is arithmetic, not measurement.** The
   multiply-add **count** is measured; the rate (1–4 GFLOP/s for a strided f64 FMA) is
   assumed, which is why the answer is a 3–37 % range rather than a number.
   *Answered by:* S1 timing its own kernel on this same 200-chunk harness.
8. **`collapse.rs` is 3,006 lines against the 700-line threshold** (4.3×, flagged by the
   write hook on every edit this run made). The M0 harness lives inside it. Not this run's
   to split — flagged for the integrator, per the hook's own *"propose the extraction rather
   than doing it silently mid-task"*.
