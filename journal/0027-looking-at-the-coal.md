# 0027 — Looking at the coal

*2026-07-20. Photo walk on the 0026 organic materials. No code changed —
screenshots and this entry only.*

0026 was headless. It shipped three materials — coal, peat, carbonaceous
mudstone — flipped the biotic layer on in production, and proved every claim
with assertions. Nobody had looked at any of it. Carbonaceous mudstone is now
the second most abundant facies in the world, so this walk existed to answer one
question: **does the new dark end of the palette ruin the look of a cut face?**

> blogworthy: **a material whose data is correct, whose fullbright control is
> correct, and which still renders as a hole in the screen.** The coal shot is
> the cleanest case yet for why the fullbright pass is mandatory — it is the only
> reason we can say "the renderer crushed it" instead of "the material is wrong."

## The world I could not open

The 0026 sites are on seed `0x0D5E_ED57_2026`. The client cannot open that world.
`dc-client` takes no `--seed`; it boots `Authority::new(BENCH_SEED, ..)` with
`BENCH_SEED: i32 = 1337` and `WORLDGEN_EXTENT = Extent::Medium`. The seed is an
`i32` on that path, and `0x0D5E_ED57_2026` does not fit in one — so world voxel
`(107338, 58787)` and its 24 m seam are not reachable from the game at all. They
are a headless-test address, not a place.

That is worth saying plainly because the milestone entry says "stand at world
voxel (107338, 58787) and dig down," and you cannot. So I photographed **the
world a player actually gets**: seed 1337, Medium, and I re-sited every shot in
it with a throwaway probe over the deep grid. Seed 1337 turns out to be a
perfectly good stand-in — 609 coal-bearing cells, 17 peat-bearing cells, and
140 728 of 297 025 cells carrying an organic soil horizon.

The best coal site in the player's world is world voxel **(-76133, -80221)**,
and its column is *better* framing than the measured one:

```
  y83   grass
  y82   mudstone
  y81   mudstone
  y80   carbonaceous mudstone
  y62–79  COAL  (18 voxels)
  y59–61  basalt
  y40–58  granite
```

Eighteen voxels of coal four voxels under the turf, on flat grassland at 75 m.
A player would hit it with a shovel.

## The first cut showed nothing, and that was the finding

I dug a road-cut: a 25 × 22 × 60 voxel box of air with the section left standing
as the far wall, camera inside, level, twenty metres back. The screenshot was
black. Not dark — black, with a crosshair in it.

The first guess was the obvious one: coal is a dark rock, and I had shot it in a
pit with no sky. So I re-shot the *opposite* wall in case the sun was on the
other side. Also black. Then I opened the pit up from above
(`0027-pit-interior-unlit-lit.png`) and got the actual mechanism: from the rim
the world is a bright green plain with a clean brown band of mudstone around the
lip of the hole — and then the interior falls off a cliff into nothing. Vertical
faces deep in an excavation receive essentially no light. The mudstone rim reads
because it is at the top; four voxels lower the same rock is unreadable.

So I stopped digging pits and cut a **bench** instead: a wide, shallow, open
quarry floor with the section standing as one short face under open sky. That is
`0027-coal-seam-cut-lit.png`, and it is the ratification shot. It reads, top
down: green turf, a thick red-brown mudstone band, a thin grey-brown band you
have to be told is carbonaceous mudstone — and then the entire lower two-thirds
of the frame is black. Ten voxels of coal in the face, and the bench floor is
coal too, and the floor is an *up-facing, fully sunlit surface*, and it is also
black. The coal is not shadowed. The coal is just black.

## The control says the data is fine

`0027-coal-seam-cut-fullbright.png` is the same framing, same excavation, same
pose, launched with `--fullbright`. Coal renders as a perfectly ordinary
mid-dark grey — a rock, plainly distinct from the mudstone above it. Nothing is
wrong with the block, the atlas, or the palette.

The number is in `meshing.rs`:

```rust
(Block::Coal, _)                 => [0.07, 0.065, 0.06, 1.0],
(Block::Peat, _)                 => [0.24, 0.17,  0.11, 1.0],
(Block::CarbonaceousMudstone, _) => [0.21, 0.18,  0.15, 1.0],
```

0.07 linear is about 0.29 in sRGB — which is exactly the grey the fullbright
pass shows, so the fullbright pass is telling the truth. Then the lit path
multiplies that 0.07 by a directional term and tonemaps it, and 7 % albedo has
nowhere to go but zero. Real coal has an albedo around 0.04–0.08, so the
*number* is defensible; what is not defensible is that our lighting has no floor
under it, so a physically-correct dark material becomes an absence of image.
This is not a material bug and it must not be fixed by brightening coal — filed
to Observed as a lighting/tonemap floor question, with the pair of photos that
isolates it.

## Carbonaceous mudstone, the thing we were actually worried about

The abundance claim was the ratification risk, and it is the good news.
`0027-carbonaceous-mudstone-ordinary-lit.png` is an ordinary hillside at
(4096, 2048) at 988 m — no special site, just a place — cut to a bench. The
profile is grass / one voxel of mudstone / **three voxels of carbonaceous
mudstone** / basalt / granite. The new rock is the single thickest unit in the
soil profile, exactly as the census predicted.

It reads **well**. It is a dark chocolate brown, and against the red-brown
mudstone above it and the blue-black speckled basalt below it, it is its own
colour and its own layer. It makes the profile *more* legible, not less: before,
that band was more mudstone. The fullbright control shows why — it sits at a
genuinely different hue and value from mudstone's salmon, not just a darker
version of it.

The world is not muddier for it. `0027-vista-lit.png` is the honest wide check:
a flat green horizon to the fog line, and not one pixel of new rock anywhere in
it. Nothing organic reaches the surface, because the surface is grass. That is
reassuring for the "did we make the world dark" question and slightly damning
for a different one — the vista is *relentlessly* uniform green, which is the
grass/dirt frontier item already in Observed wearing a different face.

## Peat exists, and it is nearly the same colour as its neighbour

0026 expected peat to be unfindable — 72 surviving units on the measured seed.
Seed 1337 has 17 peat-bearing cells and one of them is spectacular: world voxel
**(-22983, 24546)**, sixteen voxels of peat under two of mudstone, with a single
voxel of coal beneath it and carbonaceous mudstone under that. All three new
materials in one twenty-four-voxel section, and the stratigraphy tells the story
in order: soil, buried and cooked to coal, then a younger unburied peat bog on
top of it. `0027-peat-coal-mudstone-section-lit.png`.

Appraisal is mixed. Peat reads better than coal — it is a very dark warm brown
with a visible surface, not a void. But 0.24/0.17/0.11 against carbonaceous
mudstone's 0.21/0.18/0.15 is a difference you can measure and not really one you
can see, and the fullbright control makes it worse rather than better: side by
side and unlit, peat and carbonaceous mudstone are nearly the same taupe. Two of
the three new materials are competing for one slot in the palette. Peat is rare
enough that this may never matter, but the pair that *does* matter — organic
soil against ordinary mudstone — is well separated, so the priority is right by
accident.

## What I did not do

No code changed. No gates run — this commit is `.md` and `.png` only. The
excavations were runtime `dc:world/fill` edits and died with the process; both
passes replayed the identical fills and poses so the lit/fullbright pairs are
the same framing, not two similar framings.

## Files

`journal/assets/0027-*.png` (nine), this entry, ROADMAP Observed.
