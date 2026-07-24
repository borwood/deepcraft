# 0091 — The ladder that ends the standoff

STUB / in-progress (agent-a2976). Far-field LOD band-inversion + constant-coupling fix.

Four coupled changes (user-directed):
1. Warm geometry-safe — floor/clamp the warm-reduce surface top the way cold synth floors, so warm can't poke through the near ground.
2. Drop the 176 m reduction standoff — warm-where-resident (A-5 guard kept), cold-only-where-not.
3. One LOD ladder — nearfield border + each step's range-past-border in one place; ring assignment, warm/cold selection, near/unload radii all derive from it.
4. Ranges as named knobs (future per-player perf setting; no UI now).

Render-only: goldens must stay unmoved. Fix (b) — the S-9 material reconciliation (cold dither vs warm dominant-subsurface) — is NOT this slice.
