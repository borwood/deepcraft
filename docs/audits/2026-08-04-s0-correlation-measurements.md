# S0 — the stratigraphic-correlation pre-slice measurement run (M0′)

**Status: MEASUREMENTS IN FLIGHT** — every section carries its command and its
CANNOT-DETERMINE line; numbers land as runs complete.

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

---

## 1. (a) Per-cell H histogram + max stack depth

**Method.** One pass over `DeepField::strata` (297,025 cells). *Stack depth* is
`DeepStrata::units.len()`; *H* is `Σ unit.thickness_m()`, which the journal/0053 finalize
invariant makes identical to `DeepField::regolith` — the probe reports the max absolute
disagreement as a self-check on that identity. *Events per cell* is a different quantity
(it is a per-**column** expression product, not a record product) and is measured in § 2
with the fill, where it belongs.

*(numbers pending)*

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

*(numbers pending)*

## 3. (c) Shared-partition interval counts under EPOCH correlation

**Method — and why it supersedes the design's (c).** Under the clock, the shared partition
over a stencil is the **union of the epochs its parents carry**: each `DepUnit` carries one
epoch, epochs are monotone up-stack, so a parent's stack partitions into epoch-runs and a
parent contributes zero thickness to an epoch it does not hold. The union count therefore
*is* the per-column dot-product length. Measured as a 256-bit presence bitset per cell,
OR-ed over every 2×2 stencil (the interior case, 544² stencils) and every 3×3 stencil (the
straddling case, 543²).

*(numbers pending)*

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

*(numbers pending)*

## 5. (e) Epoch statistics

**Method.** Per cell: distinct epochs present (the same bitset as § 3), distinct chapters
for comparison, and the record-wide epoch range. At every unit whose `unconformity()` flag
is set and which has a predecessor in its own stack, the **gap** is
`epoch(k) − epoch(k−1)`. Flagged units at the record base have no predecessor and no
defined gap; they are counted separately rather than folded in as zero. The probe also
counts epoch **non-monotonicity** up-stack — which must be 0, and is the field-scale
version of the debug assert journal/0154 landed.

*(numbers pending)*

## 6. (f) `contents_contract` re-baseline

**Method.** `cargo test --release -p dc-worldgen --test contents_contract`, wall-clock of
the whole suite. The 102.43 s figure is journal/0129's machine-state (2026-07-26) and
predates P11 slices 1–3, the packed `DepUnit`, the near-path restructure and the
deposition clock.

*(numbers pending)*

## 7. (g) THE UNIT-COUNT HYPOTHESIS

**The question** (ROADMAP § Observed, *"ONE COUNT, TWO VALUES"*): journal/0136:280 says
**10,951,030** recorded units; `2026-08-03-stratigraphic-correlation-design.md` F1 says
**~7.4 M** over the same 297,025 cells — a 48 % gap with no pointer either way. The
dispatch hypothesis: the older count predates slice 3's sub-quantum carry.

**Method.** Two independent instruments on the current tree: the new probe's direct sum
over `pregen.deep.strata` (the shipped world the game loads), and `member_diversity_probe`
re-run unchanged — the very instrument that produced 10,951,030.

*(numbers pending)*

## 8. What each number decides

*(pending)*

## 9. CANNOT-DETERMINE

*(pending)*
