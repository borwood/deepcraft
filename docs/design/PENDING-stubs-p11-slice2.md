# PENDING stubs.md edits — P11 slice 2 (the conversion)

*Ordinals are the integrator's. These are the entries this slice moves, with the
text to fold in.*

---

## RESOLVE (by construction) — the `SPECIES = Litho::COUNT` budget width

Not a numbered stub today, but it was the live instance of *"a summary is not an
authority"* on the transport side: `Litho::of_material` bucketed a recorded
`MaterialId` back to a class **because the tables downstream were class-keyed**,
and `lithology.rs`'s own comment named slice 2 as what deletes the call. It did.
The comment is rewritten in this slice (A-2 sweep), and the call survives only for
consumers that genuinely want a class — the far tier's `ShareVec<7>`, the outcrop
verdict, the transformation edges, and the degenerate no-content door. **Heir:
slice 4**, unchanged.

## NARROW — #36 `the-deep-species-mask-is-one-word-wide`

Added by the foundation. Unchanged in substance, but now **live rather than
prospective**: the one-word mask is what `SpeciesLayout` addresses in production,
and the ceiling is `MAX_DEEP_SPECIES = 64` against the registry's stricter cap of
51 (`EdgeId`'s mixed-radix packing, stubs #21). No action; note that the entry is
now guarding a shipped structure.

## NARROW — #25 `the-record-does-not-say-which-mover-delivered-it`

The arriving-identity argmax is now over **materials**, so its doc comment moved,
but #25's arithmetic survives the re-grade unchanged: 26 materials need 5 bits,
leaving 3 for a mover, still exactly one byte. Restate the pointer at
`erosion/record.rs::arriving_material`.

## NEW — the degenerate no-content door is now a **two-armed rate path**

`SusTable::{Class, Member}` (erosion/weathering.rs) and
`Providers::outcrop_at`'s empty-axis branch answer the same question two ways,
selected by whether a content set was supplied. That is legitimate (it is the S-5
identity default, and the member arm is a strict generalisation — every material
that *is* its class's reference member blends to the same number bit for bit), but
it is a **stand-in with a named heir**: slice 4 dissolves `Litho` and with it the
class arm, at which point `SpeciesAxis` is mandatory and the door closes.

- **Blast radius if it hardens:** every rate consumer (`expose`, `periglacial`,
  `wind`, `wave`) carries a branch that a reader must resolve before they can say
  what the world's erosion rate *is*.
- **Heir:** P11 slice 4.
- **Today's identity:** the class arm is byte-identical to the pre-slice-2 rate
  path, so a no-content harness reproduces exactly what it used to.

## NEW — `refine.rs` runs a **different tier configuration** from production

`measure_decay` now sets the species axis and mirrors `material_transport`, so the
decay experiment's rates are member-grade like production's. It does **not**
mirror `material_creep`, and its `record` is off, so the composition the rates
blend is the one the record would have held had it been recording. That is honest
for a relaxation-decay measurement (the experiment perturbs bedrock and measures
surface penetration) and it is not the same solve production runs.

- **Blast radius:** a future halo-sizing decision read off `S*-results` would be
  read off a solve whose record is empty.
- **Heir:** whoever re-runs the decay experiment for a real refinement slice
  (architecture C).
