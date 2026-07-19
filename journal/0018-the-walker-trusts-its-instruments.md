# 0018 — the walker trusts its instruments (walk 13)

*2026-07-19 · main session walk, immediately after integrating the S1-fallback
sweep (journal/0017, merge `b1a3054`). Seed 1337, N=2 worldgen authority,
`--fullbright`. The owed items: walk 12's deep-time cut-face photograph, live
verification of the hardened instruments (journal/0016), and first live
contact with the sweep's collision fix.*

This was the first walk conducted under corrections #10's discipline: every
cross-check between a pose (meters) and a block query (voxels) was converted
explicitly, on paper, before being believed. It changes the texture of the
walk — slower, and much harder to fool.

## The instruments, exercised where they used to lie

Teleported `surface:true` to (2000, 2000) — a place nothing had ever
streamed. Feet seated at 982.85 m; converted to voxels (÷0.9 → column
(2222, 2222), feet voxel 1092), the block queries said: 1092 **air**, 1091
**grass**. Instrument and world agree at an unstreamed location — the exact
configuration where `eye_in_solid` used to answer from the S1 planet and the
walk-12 report was born.

Then the sweep's headline fix, tested the honest way: spawned `scout13` in
mid-air at (−2000, 1100, −2000) — another never-streamed column — and
watched the pose stream: falling at −80 m/s, then `on_ground: true` at
967.5 m, velocity zero. Under pre-sweep code that walker's collision query
would have fallen back to the legacy S1 terrain a kilometer below; it would
have kept falling. It stood.

One instrument gap found: the `surface:true` teleport reply carries no
`surface_snapped` field **on the success path** — the snap is only inferable
from the y actually moving. Journal/0016 added the field for the miss case;
absence-means-success is exactly the kind of ambiguity corrections #10 warns
about (silence is not a reading). Filed to Observed alongside the owed
voxel-coordinate echo.

## The quarry, and the character who rode it down

The cut-face photograph took two attempts, both instructive. From the rim at
a shallow angle the pit hides itself — the near edge occludes everything but
the top bands of the far wall (the first exposure shows soil-over-mudstone
and nothing else). The real photograph required walking INTO the ground:
extend the cut to a 20×28-voxel trench, then `surface:true` teleport to its
center — **and the scan seated the feet at 945.05 m, the carved trench
floor, not the pre-edit surface.** `true_surface_m` includes edits; verified
live, incidentally, by trusting it with the photographer's body.

`0018-deep-time-cut-face.png` is the picture walk 12 owed: from the trench
floor, a 22-m wall reading top to bottom — grass rim, red-brown mudstone,
a dark basalt band, then massive pale granite densely speckled green with
olivine pore-partial inclusions (the 3d accessory machinery, photographed at
outcrop scale for the first time). A 500-m column's upper story, told by a
hole in the ground.

And standing at the bottom of the frame, for scale: **scout13**. It had
landed at 967.5 m on what was then the surface; the trench was carved
underneath it; it fell 22 m with its floor and re-grounded, upright,
velocity zero. Nobody planned the shot. The character surviving the
excavation of its own footing is the collider-invalidation path and the
authority-routed ground-finding demonstrating themselves in one image.

> blogworthy: the accidental self-portrait of a physics stack — carve the
> ground out from under a walker to photograph geology, and the walker
> riding the excavation down IS the verification of the system you actually
> came to test.

## The empty horizon

The far-field assessment walk 12 owed (`0018-empty-horizon-player-view.png`,
`0018-far-field-vista.png`): from a walker's eye at the worldgen surface,
the near terrain rolls out to the load radius and then the horizon is
**sky**. Nothing. The world ends like a floating island. The S1 phantom
old-world — user-sighted in walk 8 as terrain ~500 m below — is still there,
but from ~1150 m up it registers only as a scatter of haze-bleached
fragments a kilometer down at a steep viewing angle; at eye level it is
simply invisible behind the near silhouette. So under the worldgen
authority the far mesh contributes *nothing a player can see from the
ground* except, occasionally, a wrong world glimpsed off cliff edges.
This sharpens the far-field milestone's framing: it isn't "fix the phantom,"
it is "there is no horizon at all" — the summary-pyramid far field
(journal/0017 § far mesh) is building the horizon for the first time.

## Ledger

Verified live this walk: `eye_in_solid` authoritative at unstreamed
locations · `surface:true` snap correct (voxel-converted cross-check) ·
streaming-edge character collision (mid-air spawn → landing) · re-grounding
through edit-driven collider invalidation · `true_surface_m` honoring edits ·
deep-time strata legible at outcrop scale. Filed: `surface_snapped` absent
on success; the empty horizon as the far-field milestone's true shape.
