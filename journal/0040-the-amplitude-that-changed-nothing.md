# 0040 — The amplitude that raised a continent and changed nothing

> blogworthy: a knob that did exactly what it promised, moved a kilometre of
> rock, and was invisible from the ground — because it operated at a
> wavelength no player will ever stand at.

The deep-config plumbing (journal/0039) opened the sealed gen path so a world
could finally be booted with `tectonic_history` on. This is the first walk
through that door, and it was supposed to settle the amplitude call — user
decision U7, posed since the tectonics ratification as a straight choice
between `thickening_scale` 80 and 160.

It settled it, but not by picking one.

## The second seal

The plumbing was not the only thing sealed. On connecting to walk, the game's
MCP surface — the thing every walk since journal/0003 has been driven through
— refused the connection, and `/mcp` reported it could not authenticate.

Neither was true in the way it read. The deepcraft MCP server is hosted
*inside* `dc-client`; with no game running there is nothing on port 7777, and
`ConnectionRefused` renders in the client UI as an auth failure. But the
deeper finding was that **the server was registered nowhere at all** — not in
the repo, not in the project's local config. Every prior walk had connected
ad hoc and nothing carried forward. The fix was four lines of `.mcp.json`
committed to the repo, and it is the kind of thing that costs a session every
time it is rediscovered. The audit trail is a deliverable; so is the tooling
that produces it.

## Choosing an instrument that is not blind

The walk had a second problem. The far field cuts off at
`farmesh.rs::FAR_MAX_M = 1200.0` — a 1.2 km horizon. A mountain range is
5–20 km across, so the camera is always *inside* the landform and its macro
shape never enters frame. The instrument that would normally answer a shape
question (the lit pass, corrections #18) is blind at the scale the question
lives at.

But `pose_set { surface: true }` returns the true seated surface height at any
`(x, z)`, and **that is not limited by the render horizon**. So the walk
became a survey: teleport, read the height, move on. The visual register
stayed in the loop for what it *can* see — the ground underfoot — while the
numbers carried the landform scale. Corrections #18/#19 keep teaching the same
lesson from new angles: pick the control that can see your question. Sometimes
that control is not a camera.

## What the survey found

The first transect was a warning. Ten kilometres east of origin, the surface
climbed 912.8 → 951.4 m: **38.5 m over 10 km**, a 0.4 % grade. But a single
line through what might be craton interior proves nothing about orogens, so
the survey widened to ±100 km.

At that scale the world is *correct and impressive*: a continent margin
descending 912 → 143 m eastward, and northward a plunge to **−3523 m** of
abyssal plain — about 4.5 km of total range, with the right shape. The
tectonic model is producing continental structure.

The highest land in the sampled world sat at **1288.9 m**, at (10500, −15000).
And there the survey turned bleak. Across the whole belt: ~380 m of relief
over 40 km, ≈1 % grade. Down the steep axis, 231 m over 10 km — 2.3 %. Real
mountain fronts run 20–40 %.

Then the measurement that mattered. Sampling every 250 m across the summit —
the finest scale, the one a walking player actually occupies:

```
1287.1  1288.0  1288.9  1286.2  1281.7  1281.7  1285.3  1286.2
```

**7.2 m of variation over 1.75 km.** The fine scale was not hiding relief. It
was *flatter than the macro*. The photograph agrees without ambiguity: the
summit of the world's highest range photographs as a level green prairie with
scattered single-voxel ledges — which is exactly what a 0.4 % grade does under
0.9 m quantization, since terrace risers land ~200 m apart and read as isolated
steps rather than a slope.

## The A/B

Same seed, `--tectonics` on both, only `--amplitude` changed from 80 to 160.

The knob works, and works correctly. Continental elevation rose +714 to
+1079 m. Origin went 912.8 → 1659.8 m; the crest 1288.9 → 2249.2 m. The
abyssal plain moved **1.8 m** (−2492.1 → −2490.3), which is precisely right:
orogenic thickening should not drive ocean floor, and it didn't.

Then the same fine transect, on the same crest:

```
2248.3  2248.3  2249.2  2247.4  2242.9  2242.0  2246.5  2247.4
```

**7.2 m over 1.75 km.** Not similar — *identical*, to the decimetre. A full
kilometre of added rock bought exactly zero walking-scale relief. The 10 km
transect actually got **flatter** (38.6 → 22.4 m). Only at ~25 km did the knob
bite at all (380 → 626 m).

The two ground screenshots, same vantage, same height above surface, are
visually indistinguishable.

## Why (a hypothesis, and the real lead)

`DeepField::surface_at_voxel` is a **bilinear** sample of a **460 m** deep
grid. Every wavelength finer than that comes from the collapse elevation
lattice's own jitter, which is seeded from `provenance_roughness` and never
sees `thickening_scale`. That accounts for an exactly-identical sub-cell
number, and it is consistent with everything measured.

What it does *not* account for is the size of that number.
`provenance_roughness` runs 90 m (Craton) to 420 m (Orogeny) — and the
measured walking-scale relief is **7 m**. Something in the lattice's
per-refinement amplitude decay is attenuating roughness by one to two orders
of magnitude. **That is the lead**, and it is unmeasured; per corrections #19
this mechanism stays a hypothesis until someone puts numbers on it.

## What this does to the sequence

The standing claim — *"the model is not the bottleneck, the amplitude is"*
(journal/0029, /0030) — is falsified at the scale that matters, and filed as
corrections #23. `thickening_scale` is a **lift-the-continent** knob: it acts
at ~25 km and above and vanishes below. U7 as posed ("80 or 160") asked the
user to pick between two values neither of which fixes dismal mountains. The
honest answer to the amplitude call is **neither, and here is why.**

The legibility problem lives below the deep grid: in the collapse lattice's
roughness decay, and in erosion supply (already Sequenced from S12's
metre-scale exhumation finding). Both are now upstream of any amplitude
decision, and the amplitude decision itself can be deferred without blocking
anything — which is the opposite of what the roadmap assumed this morning.

Assets: `0040-tectonics-crest-lit-south.png` (amp 80),
`0040-tectonics-crest-lit-south-amp160.png` (amp 160).
