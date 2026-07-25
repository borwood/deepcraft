---
name: tour-map
description: Find the walk stations BEFORE spending the user's game time — a headless probe that locates the strongest exemplar of each signature, prints coordinates and a ready pose, and honestly reports a null. Use whenever an appearance walk is owed, before launching the game.
---

# tour-map — never launch a walk you haven't scouted

**A walk with nothing to look at costs the user's session. A tour map costs one background
agent.** Run this first, every time.

*Earned 2026-07-25: the geotherm's coal walk was cancelled **before launch** because the tour
map found **zero coal on the shipped world** — and that null exposed a guard running on a
world nobody ships (corrections #51). It was the most valuable result of the day, and it cost
no game time at all.*

## When

Any time an appearance walk is owed (ROADMAP § APPEARANCE WALKS OWED), **before** launching
`dc-client`. Also before any "go look at X" request where you do not already have coordinates.

## What the probe must produce

Dispatch a background agent (**read-only on `src/`**; its writes are one `examples/` probe and
optionally a note under `docs/audits/`). Require:

1. **The station** — world **metres** `(x, z)`, the surface elevation, and a **ready-to-paste
   pose** (feet x/y/z in metres, `yaw`, `pitch` — **radians**). Frame it: state the altitude or
   standoff that actually fits the signature in view.
2. **What makes it the strongest exemplar** — the measured quantity that ranks it (band
   thickness, class count, seam depth), not an impression.
3. **A contrast station where one exists** — the pair that makes a *change* legible rather
   than merely present. If no such pair exists within a walkable distance, **say so**.
4. **The distribution** — how many cells carry the signature, and its spread. This is what
   turns "found one" into "here is what the world looks like".
5. **Distance between stations**, so one launch can be planned. Teleports are free; knowing
   costs nothing.
6. **A NULL IS A RESULT.** If the signature does not occur, say so plainly **and prove the
   instrument works** by running the identical census on a world where it *does* occur.
   *A zero from an unproven census is not evidence.* That control is what made the coal null
   trustworthy rather than a suspected probe bug.

## Rules

- **Use the world the player actually boots** — `BENCH_SEED` (1337) at the shipped extent.
  A tour of a convenient fixture world is worse than no tour (corrections #51).
- **Pick the instrument that can see the question** (CLAUDE.md § Agent walks): material
  questions → `--fullbright` + `world_get_contents`; shape → the lit pass; **never**
  `scan_region` for material.
- **Do not tune anything.** The probe's job is coordinates and counts. A tour map that
  "improves" the thing it was sent to look at has destroyed the walk's evidence.
- Gate the probe if it carries assertions (`[[example]] test = true`, journal/0103).

## Then

Hand the stations to the walk loop (CLAUDE.md § Agent walks — **Claude launches, teleports,
measures, screenshots, briefs, and pauses**). If the tour returned a null, **brief the null
and cancel the walk** — do not launch anyway to have something to show. Convert the
appearance question into whatever desk question the null actually raises.
