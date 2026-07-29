# A1 slice plan — block↔material collapse (present tier)

> ## ✅ EXECUTED 2026-07-23 — journal/0087. This is a historical plan, not a to-do list.
>
> Every step below shipped: `Block` is `{ Air, Material(MaterialId) }`, `block_twin` and
> the `block_layer` geology arms are gone, solidity checks ride the occupancy primitives.
> See `docs/spines.md` § compliance (2026-07-23) and `ROADMAP-history.md`. The branch and
> base commit named below are dead references. **Preserved as the record of what was
> planned and in what order; do not act on it.**
>
> *Banner added 2026-07-29 (baseline sweep S8/B4).*

Branch: worktree-agent-a63c85f041dedde50. Started from 606f17a.

## Steps
1. Redefine `Block = { Air, Material(MaterialId) }` in dc-core voxel.rs; niche
   MaterialId → 1-byte atom. Measure chunk size before/after (verify 64→32 KB).
2. `classify` returns `dominant_material()` (delete `block_twin` + `_ => Stone`);
   9 face-less materials now wear their own identity.
3. Migrate render path — delete `block_layer` geology re-translation arms.
4. Drain solidity checks onto occupancy primitives where the compiler forces it;
   list deferred sites.

Do NOT: build deep-cell inventory, touch dc-worldgen/deeptime/**, retire legacy S1.

Acceptance: byte-identical render/behavior where faces exist; 9 face-less
materials now show own identity (NEEDS-RATIFICATION appearance change).
