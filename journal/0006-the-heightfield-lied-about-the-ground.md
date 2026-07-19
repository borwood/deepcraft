# 0006 — The heightfield lied about the ground

*2026-07-19 · surface-query fix + attach placement guard (background agent,
integrated + walk-verified by the main session)*

> blogworthy: three field reports blamed one number — `surface_height_m` "under-
> reports by ~7 m." The real story was better and worse: an analytic heightfield
> and the voxels it generates are *not the same surface*, and the gap is up to
> **14.5 m** on a chasm wall — but also up to half a voxel on flat ground, so it
> quietly buried more than half of all near-surface placements. The fix was to
> stop asking the heightfield where the ground is and ask the voxels.

## The problem as encountered

Since walk 3 the same ghost kept surfacing (journal/0004, 0005): bodies placed
"on the surface" ended up *inside* it. The player spawned with its camera in
solid; `pose_set { surface: true }` teleported feet into a hill; a character
attached with the default position spawned embedded to the chest and — because
swept collision won't move an interpenetrating box and the character surface has
no despawn verb by design — became a permanent statue. Every one of these
trusted `TerrainGen::surface_height_m`, and the standing hypothesis (journal/0004)
was that the analytic helper "under-reports the actual voxel surface by ~7 m,
likely a detail octave present in voxel generation but missing from the heightmap
helper."

## The wrong turn in that hypothesis

There is no missing octave. `surface_height_m` is not a *different* field from the
one voxelization uses — `block_in_column` calls it directly. Generation and the
helper sample the identical noise. So a same-column disagreement of several
meters is impossible *by construction*: `block_in_column` makes a voxel solid
only when its center is below the analytic height, so within one column the top
solid face can never rise more than half a voxel *above* the analytic value. The
"detail octave" explanation is falsified (→ corrections).

## The mechanism, measured

The gap is real; it is just not where the hypothesis pointed. Two effects
compound, both quantified by a new deterministic test
(`worldgen::tests::surface_height_m_underreports_true_voxel_surface`, seed 1337,
scale 3, 0.6 m voxels, a 0.6 m-wide player footprint, ~12,500 near-surface
columns):

1. **Half-voxel top-face offset (flat ground).** A voxel is solid iff its
   *center* sits below the analytic height, so the solid *top face* sits up to
   half a voxel (0.3 m at scale 3) *above* that height. Feet clamped to
   `analytic + 0.05 m` are already below the true top face about half the time.

2. **Footprint over a slope (the spectacular case).** The helper is one point
   sample per column; a body's footprint spans neighbouring columns, and the
   resting surface is the *highest* voxel under any of them. On the chasm walls
   — a 90 m smoothstep carve over a narrow band — neighbouring columns differ by
   many meters. Worst measured gap between the analytic center height and the
   true footprint-max surface: **14.49 m** at (−182.6, −189.4).

Together: **7,669 of 12,468** sampled near-surface columns (≈61%) would embed a
standing body if placed at `analytic + 0.05 m`. The ~7 m from the field was a
real mid-slope case; the tail is twice that. This is why `eye_in_solid` caught
buried spawns *intermittently* — a sub-voxel bury is invisible until the camera
crosses a voxel boundary, a chasm-wall bury is total.

## The fix: ask the voxels

The correct surface for placing a body is the actual voxel data — edits and all
— not an analytic field. New `dc_core::column_top_solid_y` scans a column
downward for the topmost solid voxel; `worldgen::true_surface_m` takes the
**max over the footprint columns** of that top face, each column scanned from a
ceiling seeded by *its own* analytic height (a per-column upper bound — a single
center-column ceiling starts below a steeper edge column's real surface and
misses it, which the test caught mid-development). The analytic height keeps its
one honest job: a cheap upper bound to start the scan safely in air.

Everything that places a body now routes through it: open-ground spawn
(`find_open_spawn`), the `surface: true` player teleport (via `ChunkMap`, so
loaded edits count), and the character attach snap (via the host world, edits
included). `surface_height_m` remains for *generation* — nothing that seats a
body or a camera trusts it anymore.

## The attach guard (walk 5)

Attach was unguarded: an embedded spawn was a permanent statue. Now
`handle_character_attach` builds the body AABB at the target feet and refuses the
attach if it overlaps solid (`dc_core::aabb_overlaps_solid`, using the sweep's
own EPS cell rule so *resting flush on the ground is not "embedded"*). The
refusal is a machine-readable receipt in the attach flow's existing shape —
`{ ok: false, code: "obstructed", character, error }` — and **no character is
spawned**. Opt-in `surface: true` on `character_attach` first snaps the feet to
the true surface at the requested x/z (mirroring the player teleport), then the
guard confirms the landing is clear. The raw dev `spawn_character` is
deliberately left unguarded (the dev surface keeps full reach).

## Walk 6: the proof photograph

Integration walk (main session, merged main, `--fullbright`), aimed at the
worst spot the test found. First lesson re-learned before the walk even
started: the running exe predated the merge — `character_attach` had no
`surface` field in its live schema. `cargo test` does not relink the main
binary; **check the exe timestamp against the merge before walking**.

On the rebuilt client, at the exact worst-case coordinates (−182.6, −189.4):

- Attach at y = −200 (deep in the wall) → refused: `{ ok: false, code:
  "obstructed" }`, no statue, and the error text says what to do instead.
- Attach with `surface: true` → spawned, proprioception reports
  `on_ground: true`, `eye_in_solid: false`, feet at y = −4.8 — standing on a
  ledge of the near-vertical chasm wall, terrain that would have buried the
  old placement 14 m deep
  (`assets/0006-walker-snapped-to-chasm-wall.png`).
- Player `surface: true` teleport 15 m away landed feet at y = −67.15 — that
  column is the chasm *floor*. The per-column scan is doing real work:
  neighbouring columns 60 m apart in height each get their own honest answer.

Side observation for the record: the photo is the first clean-air look at the
chasm-wall "speckle" (Observed since walk 3) — from this angle it reads as
genuine steep-slope terracing, single-voxel ledges with gaps, not a mesh
defect. Diagnosis still owed; the asset is now evidence.

## Status

- [x] mechanism diagnosed + quantified (14.49 m worst gap; 61% of columns embed)
- [x] `true_surface_m` (footprint-max, per-column ceiling, edits-included);
      spawn + teleport + attach routed through it
- [x] attach embed guard + opt-in surface-snap; obstructed receipt
- [x] gates green on merged main; character replay still bit-identical
- [x] walk-verified photographically at the measured worst case
- [x] falsified "missing octave" claim → corrections.md #5
- [x] ratified 2026-07-19: `surface: true` attach semantics; dev
      `spawn_character` staying unguarded; the JSON `obstructed` receipt shape
