# S0 — the stratigraphic-correlation pre-slice measurement run (M0′)

**Status: SKELETON — measurements in flight.** Every number below is filled in from a
run whose exact command, seed, extent and commit are recorded beside it. Anything not
measured stays **CANNOT-DETERMINE** and says why.

**What this is.** `docs/audits/2026-08-03-stratigraphic-correlation-design.md` § 6 names
M0′ as the pre-slice measurement run the build owes before S1 wiring. This file is that
run, **updated for two things that landed after the design pass was written**:

1. **The deposition clock shipped** (journal/0154, `2026-08-04-deposition-clock-design.md`).
   Every `DepUnit` now carries a raw **epoch** byte. Correlation matches **epoch
   intervals**, not chapter fractions — so the design's measurement (c) (*fraction
   breakpoints under R-C*) is **superseded** by epoch-interval counts, measured here.
2. **P-1 ruled M-C** (2026-08-04, user): mixture grading for **continuum** pairs, the
   octaves cut for **discrete** pairs, with the continuum predicate **derived** from the
   FS-A release spectra + property sheet, never hand-authored. So measurement (d) (the
   mixture-cap check) is live rather than conditional, and it is measured against the
   pairs the shipped world actually adjoins.

**Immutable body, mutable header** (CLAUDE.md read-first item 5). Anything that later
refutes or re-scopes a number here gets a banner **at the top of this file**, stamped by
the author of the correction.

---

## 0. Provenance

| field | value |
|---|---|
| base commit | *(filled in)* |
| branch | `worktree-agent-a5705acce6de2077d` |
| seed | 1337 |
| extent | `Extent::Medium` |
| machine | the project machine (`CARGO_BUILD_JOBS=4`, shared `CARGO_TARGET_DIR`) |

---

## 1. (a) Per-cell H histogram + max stack depth

*(pending)*

## 2. (b) `ColumnFill::build` share of `column()` — the F4 decider

*(pending)*

## 3. (c) Shared-partition interval counts under EPOCH correlation

*(pending)*

## 4. (d) Distinct blended mixtures per continuum-pair transition zone

*(pending)*

## 5. (e) Epoch statistics

*(pending)*

## 6. (f) `contents_contract` re-baseline

*(pending)*

## 7. (g) THE UNIT-COUNT HYPOTHESIS

*(pending)*

## 8. What each number decides

*(pending)*

## 9. CANNOT-DETERMINE

*(pending)*
