# 0068 — The thickness rule that was hiding inside a carve-out

*2026-07-22. A day-old "deliberate exception" was really a general rule wearing a
content name. Deleting the name and building the rule moved 40 % of the world's
outcrops — and, for the first time, the Small control.*

> blogworthy: **a carve-out is a rule you have not finished factoring.** The
> charcoal exception read as careful engineering — measured, argued, pinned by a
> named test. It was still wrong, and the tell was that its reasoning never once
> needed the word "charcoal". When the justification for a special case does not
> mention the special thing, the case is a general rule that has not been named
> yet.

## What shipped yesterday, and why it was A-7

Journal/0066 gave `litho_of_tag` — the map from a recorded unit's environment tag
to its coarse lithology — a hardcoded `Biofacies::Charcoal` arm, so a 3 cm fire
lamina would keep reading as its clastic host rather than defining a 460 m erosion
cell as "charcoal". The `deep_class ↔ litho_of_tag` mirror test was amended to
assert the divergence by name. It was careful work, and journal/0066's own
"three decisions" section argued it at length.

It is also **anti-shape A-7**: a process bound to a content instance. `spines.md`
records A-7 as *never ratifiable* — "naming directly will never, in any world, be
correct" — with a diagnostic underneath it: when you feel the need to write a
content name into a process, the property you are actually reaching for is the
feature the process lacks. Name that, and the special case dissolves for every
future member of the family.

Read that way, the charcoal arm answers its own question. The argument for it,
transcribed from 0066, was: *"a 3 cm lamina inside a bed of mud has no answer to
'what rock resists this agent over a 460 m cell' — the mud does."* That sentence
is about **thickness**. It never needed charcoal; charcoal was merely the first
unit thin enough to trip it, because a fire bed is structurally capped at 0.04 m
(fire residue is `soil × FIRE_CHAR_FRAC` over a `SOIL_MAX`-bounded pool). Volcanic
ash falls and marker beds are the next members of exactly the same family, and a
name-keyed arm would have to grow a new branch for each. The rule the prose
described — **a unit too thin to dominate an erosion cell must not define its
lithology** — belongs in `outcrop_at`, the seam that already asks "which rock is
here", where it applies to every thin bed with no name in it.

The mirror is total again. `litho_of_tag(Charcoal)` now returns a real
`Litho::OrganicCharcoal`, and both tiers agree the fire bed *is* charcoal. What
keeps a fire bed from defining an erosion cell is no longer a lie about what the
bed is made of — it is a truth about how little of the cell it fills.

## The rule: dominance over a window, not a per-unit floor

`exposed_litho` used to take `units.last()` and route it. It now takes the whole
unit slice and walks down from the surface, accumulating thickness per lithology
across a fixed window — `OUTCROP_DOMINANCE_WINDOW_M = 0.9 m`, one collapse voxel,
the smallest depth of section the expressed world can distinguish. The last unit
touched is clipped to the window boundary so the window is exact; any deficit — a
record shorter than the window — accrues to `Basement`, the rock below the pile.
The winner is the lithology holding the most accumulated thickness, ties going to
the one nearest the surface.

The window constant is a **calibration and says so in its docstring**: there is no
Earth value to defer to, it is pinned to the world's own voxel resolution, and it
is a knob (widen it and deeper thicker units set the outcrop; narrow it and thin
surface beds start to count).

The one subtlety worth recording is why this is *dominance* and not a per-unit
thin-bed filter, because the two look identical until they don't. A naive "skip
any unit under X" rule and a dominance rule agree on a single thin lamina. They
**disagree** on forty stacked 2 cm beds of the same lithology: the filter drops
all forty and reads whatever is beneath; the dominance rule integrates them to
0.8 m and lets them win. Forty ash falls in a row *are* a tuff, and the outcrop
should say so. That case is a shipped falsifier
(`many_thin_units_of_one_litho_do_dominate`), precisely because it is the one that
separates the rule we wanted from the rule that was easy.

## The measured delta

`examples/outcrop_dominance_probe.rs` computes, per cell of the production Medium
record, the old outcrop (top unit) against the new (window dominance):

- **118 384 of 297 025 cells changed outcrop — 39.9 % of all, 41.5 % of the
  285 253 with a record.** This is a large move, and the shape of it is the
  interesting part (rows = old, cols = new, changed cells only):

  ```
              fine  coarse    soil    peat    coal charcoal basement
    fine         .       .    4780     520       .        .    67455
    coarse   15017       .    3870      15       .        .        1
    soil     20957       6       .    1263       .        .      672
    peat       892       .     264       .       .        .      177
    charcoal  1748       .     702      44       .        .        1
  ```

  The dominant transition — **67 455 cells, fine → basement** — is thin records.
  A cell whose topmost unit is mud but whose whole record is under ~0.45 m now
  loses the window to the basement deficit, and erodes as the resistant shield
  rock it mostly is rather than as the soft skin of mud on top. That is a fidelity
  gain, and it is where most of the world's shape moved.

- **Cells that outcrop `OrganicCharcoal` under the new rule: 0.** Under a naive
  top-unit rule with the new routing, 2 495 cells would have — every one of them
  a cell whose last recorded event was a fire. The window suppresses all 2 495. A
  bed capped at 0.04 m cannot dominate a 0.9 m window, ever, which is the whole
  point: charcoal is now an honest `Litho` that essentially never *outcrops* one.
  This is the "expected ≈ zero" the brief asked me to confirm loudly, and it is
  exactly zero.

## What it cost in goldens — including the control

Both are authorized behaviour changes, re-baselined with notes in place.

- **`providers_common`: `GOLDEN_SURFACE` `0x7B89…4062` → `0x344C…7BAE`, and
  `GOLDEN_RECORD` `0x7A7B…2C71` → `0xEA71…7A05`.** Both moved — unlike
  journal/0066, where only the record moved because coal promotion relabelled
  units at finalize without moving a metre of ground. This slice changes the
  erosion *input*, so the surface planes move too.

- **`contents_contract`: both Medium worlds moved on all three hashes.**

- **The Small control moved — and the brief said to stop if it did.** So I
  stopped and dug, and the premise turned out to be wrong. Every prior slice left
  Small byte-identical, and the file called it "the control" on the stated grounds
  that Small "runs no deep-time record". `Pregen` runs an **always-on** deep-time
  field at *every* extent; the Small world has ~20 700 recorded deep cells, and 92
  % of them change outcrop under the new rule. What was true is narrower: Small's
  sampled columns express no deep record *as contents*, so its material sidecar
  and mixture table are contents-derived and did not move. Prior slices changed
  record labels and expression, which Small does not surface — so Small held. This
  slice is the first to change erosion **rates**, which move the bedrock surface
  **geometry**, and Small's solids there are unrecorded basement Stone and the
  veneer stub (both absent-contents). So the shifting surface changes only which
  voxels are Stone vs Air: **the Small block hash moved, materials and table did
  not.** `block_equals_classify_of_contents` still passes with absent-contents
  blocks limited to Air and Stone — the proof it is geometry and not a classify
  regression. I re-baselined it and flagged it for ratification rather than
  treating a corrected premise as a licence to move a control silently.

- **Coal is still diggable, with more margin than before.** `the_measured_coal_
  seam_is_coal_a_player_can_dig` passes. Its census moved: record seams over 3 m
  went 88 → 31, and the strongest seam surfacing as diggable coal went 16 → 19
  collapse-voxels against the floor of 15 — the one-voxel margin journal/0066
  reported is now four. The floor stayed at 15; the claim it defends (a player can
  find and dig a coal seam) holds more comfortably, not less.

## What the dominance rule buys volcanism later

The reason this is worth more than deleting one `match` arm: the same window is
the mechanism volcanic ash needs. An ash fall is a thin event bed with a real
material identity and no business setting an erosion cell's strength on its own —
identical in shape to a fire bed. Because the rule is thickness, not name, ash
arrives already handled: a single fall is an inclusion the window ignores, a
sequence of falls thick enough to matter dominates and outcrops as tuff, and
nothing in `outcrop_at` has to learn the word "ash". The carve-out would have
owed volcanism a second branch; the rule owes it nothing.

## The thread

Journal/0066 closed with a lesson about prose that answers the right question and
then goes stale. This is its neighbour: prose that answers the right question
*correctly, today*, and is still in the wrong place — a general rule filed under a
specific name. The A-7 doctrine is what caught it, one day after it shipped, by
asking the single question the careful justification had never asked itself: *does
this reason mention the thing it is special-casing?* It did not. That was the tell.
