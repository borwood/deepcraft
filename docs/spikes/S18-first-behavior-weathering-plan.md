# S18 — first real cellular behavior: sum-agent subaerial weathering on the working inventory (PLAN / in-progress)

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
