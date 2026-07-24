# Movement 2a — R/H unification (plan)

**Goal.** Make the deep-cell working material inventory the single *authority* for
surface material; the scalar `R`/`H` planes become **derived views**. Byte-identical
representation change (production goldens unmoved).

## The honest, byte-identical landing (scratch-first reconcile)

The erosion loop is a tightly-coupled scalar `R`/`H` f64 relaxation; rewriting it to
run on the inventory would move goldens. Per material-behavior.md §13.6 the ratified
shape is **scratch-first reconcile**: the hot per-epoch work stays on the materialized
`R`/`H` planes (byte-identical), and plane deltas reconcile into the inventory **as
facts at pass/chapter boundaries**.

- **The strata record IS the persistent, reconciled per-cell surface-Loose inventory.**
  The recorder (deposition pass) already reconciles each epoch's net ΔH into the record
  as deposit (`void→Loose`) / erode (`Loose→void`) facts — the inventory's own edge
  primitives — keyed by `DepTag` (→ material via the current `deep_class`/`litho_of_tag`
  rule). journal/0053's finalize invariant: `Σ unit.thickness == H` exactly. Ungated
  (`record:true` in production). No loop change → goldens unmoved.
- **`H` derives positionally:** `H = Σ Loose above the topmost Structure` (cave fill /
  buried Loose below the first Structure is excluded). In production the only Structure
  is the basal bedrock seam, so `H_derived = Σ record Loose = grid.h`.
- **`R` is a bedrock-top ELEVATION datum, not a Structure stock** (finding: §13.6's
  "R = Σ Structure" is a fully-materialized-column idealization; the two-plane engine's
  bedrock is a semi-infinite basement whose *top* sits at elevation `R`). The
  byte-identical derivation is `R = surf − H_derived` (the elevation of the topmost
  Structure contact), which recovers `grid.r` within the recorder residual.

## Write-set (dc-worldgen only)
- `deeptime/inventory.rs`: positional derivation `surface_regolith_m` (H) +
  `structure_stock_m`, and a `WorkingInventory` physical-column view.
- `deeptime/field.rs`: `DeepField::derive_regolith_at` / `derive_bedrock_at`
  (materialize-at-boundary views); keep `surf`/`regolith` planes as the exact cache.
- `deeptime/mod.rs`: re-exports.
- `tests/`: derived-vs-scalar agreement over the real production field + cave test.

## Scope held
NO material-aware transport / sorting (2b). NO weathering-rate change (3). NO new
materials / material-rule change. Material stays the current `deep_class` rule.
