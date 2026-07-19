# 0004 — The walker learns where its eyes are

*2026-07-18 · fix + verification walk (Claude; user review drove the diagnosis)*

> blogworthy: the first AI field report was mostly wrong — and the process
> caught it. The user read the screenshots better than the walker did (the
> camera was inside a block); three of walk 3's four "renderer defects"
> dissolved under that one fact, and the real fix turned out to be
> self-awareness instrumentation for the walker, not shader code.

## What walk 3's findings became

| Walk-3 claim | Verdict | Actual mechanism |
|---|---|---|
| Inverted near haze | **false** | flat shading + near-vertical sun blows out pale top faces; distance haze is normal (user diagnosis). Art calibration, queued with visuals work. |
| MCP edits invisible; receipts-vs-events seam | **false** | `tick_authority` applies *all* receipts' changes; the pillar rendered fine — the photographer was buried. Proven by `0004-pillar-verified.png`. |
| Floating LOD shards + horizon seam | **false** | camera inside a block: backfaces, reverse-side face outline (user diagnosis). |
| Spawn inside chasm | true | fixed: spawn spirals to open ground. |
| Pitch sign undocumented | true | fixed: schema says negative = down. |

Full mechanisms in corrections.md #3. Lesson worth engraving: **do not
diagnose renderer defects from a viewpoint you haven't verified is in air** —
and the user reading the walker's own screenshots is part of the loop, not a
courtesy.

## Shipped (`0ba292f`)

- `eye_in_solid` in every pose response — the walker's proprioception.
- `pose_set { surface: true }` — walker-safe teleport, feet clamped to
  terrain.
- Pitch sign documented; spawn relocated to open ground.

## Verification walk (this entry's assets)

`0004-open-spawn.png`: new spawn on rolling open hills — and `eye_in_solid:
true` immediately caught that even the *fixed* spawn buried the camera,
because **`surface_height_m` under-reports the actual voxel surface** (by
~7 m at the spawn site; location-dependent — likely a detail octave present
in voxel generation but missing from the heightmap helper). New, real,
instrument-caught finding → Observed.
`0004-pillar-verified.png`: surface-clamped teleport (`eye_in_solid: false`
confirmed), three stones placed over MCP, pillar photographed standing on
the hillside. The edit pipeline is vindicated end-to-end.

## Status

- [x] walker proprioception shipped; spawn + docs fixed
- [x] walk-3 misdiagnoses corrected in corrections.md
- [ ] `surface_height_m` vs voxel-field discrepancy (Observed; affects
      spawn, surface teleport, and anything else trusting the helper)
- [ ] near-field lighting calibration (art pass, with visuals work)
