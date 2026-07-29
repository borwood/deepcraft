# S18 — first real cellular behavior: sum-agent subaerial weathering on the working inventory (PLAN / in-progress)

> ## ⚠ BOTH OF THIS PLAN'S HEADLINE CLAIMS WERE FALSIFIED AFTER IT SHIPPED
>
> The slice below shipped (`journal/0089`). Two of its claims did not survive:
>
> - **Stage E — "a basal weathering-front band expresses" — FALSE.**
>   `journal/corrections.md` #46: at production scale the strongest band anywhere is
>   **0.04 m** against **0.9 m** voxels, which quantizes to **zero eighths everywhere**.
>   The collapsed world is effectively byte-identical. There is nothing to walk to.
> - **The title — "first real cellular behavior" — FALSE.**
>   `journal/corrections.md` #47: weathering is *continuous*; this runs `weather_column`
>   **once**, after the run, over the finished record. A snapshot of a continuous process
>   is a **category error, not a simplification**. What shipped is
>   keystone-consumption **plumbing** with a one-shot behavior stub.
>
> **What stands:** stages A–D and F are real plumbing and are consumed today —
> `Fact::cause`, the applied-edge log, the bedrock `Structure` seam, the `FactLedger`
> sidecar, the empty-ledger identity floor.
>
> **The heir** is weathering run as a per-epoch pass riding the `H` process (the R/H
> unification) — `docs/design/stubs.md` #17.
>
> *Banner added 2026-07-29 under the immutable body / mutable header policy (CLAUDE.md
> read-first item 5). Nothing below is edited.*

The slice that makes the material-behavior machinery real end-to-end and
**consumes the S17 keystone** (`WorkingInventory`/`FactLedger`, tested-only until now).

## Staged work
- **A. Facts carry `cause`; the fact source is the applied-edge LOG (not the diff).**
  `Fact::InPlace` gains `cause: Cause` (`Chemical`/`Biotic`/`Frost`/`Dissolution`;
  room left for a future `Actor`). `InvCtx` becomes cause+chapter-scoped and logs
  every `apply_edge`/`move_form`. `commit_chapter` drains the log (coalescing
  identical successive edges). `diff_facts` demoted to a validation check.
- **B. Bedrock Structure seam (stubs.md #16).** `build_working` materializes a flat
  basement `Structure` span at the base so `Structure→Loose` has a source. Loud marker.
- **C. Sum-agent weathering pass** (`weather_inventory.rs`):
  `rate = cover_taper × Σ_a (driver_a × susceptibility_{m,a})`, one fact per agent.
- **D. Run over the real record**, store a `FactLedger` sidecar per deep cell on
  `DeepField`, behind a `DeepConfig::weather_inventory` flag (default off = byte-identical).
- **E. Collapse folds `base + facts`** — a basal weathering-front band expresses.
- **F. Invariant tests** + the empty-ledger identity floor.
- **G. Walk-prep exemplar coords.**

Full write-up lands in `journal/0089`.
