# A1 slice plan — block↔material collapse (present tier)

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
