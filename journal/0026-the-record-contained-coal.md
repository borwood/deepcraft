# 0026 — The record contained coal

*2026-07-20. Milestone: organic materials + the biotic production flip.
DECIDED work — ecology.md § DECIDED 2026-07-20 (user): S10 GO, biology ships.*

> blogworthy: **a simulation that recorded coal for a whole session before the
> world could contain it.** S10 ran a real ecology across 500 Myr, watched a
> swamp hold a low-lying site long enough to lay 24 metres of peat, buried it,
> cooked it, wrote `Biofacies::Coal` into the strata record — and then the
> collapse tier turned it into sandstone, because the function that picks a rock
> read three fields of a four-field struct. The fix is nine lines. The
> interesting part is everything the fix made me measure.

## The gap, as S10 left it

S10's own results doc named it, which is the best kind of handoff:

> **The honest gap — and the reason this is GO-with-a-follow-on:** the record now
> contains coal, but **the world does not.**

The mechanism was small and completely legible. The deep-time recorder tags
every unit with a `DepTag`: `env` (subaerial/subsea), `aridity`, `energy`, and —
new in S10 — `biota`, the `Biofacies` axis carrying `Soil`/`Peat`/`Coal`/
`Charcoal`/`Retro`. The collapse tier turns each recorded unit into a material
through `geology.rs::deep_class`, which chose a content class like this:

```rust
match tag.env {
    DepEnv::Subsea => CLASS_CLASTIC_FINE,
    DepEnv::Subaerial => match tag.energy { High | Medium => COARSE, Low => FINE },
}
```

Three of the four axes. The fourth — the one S10 had just spent a milestone
learning to write — was never read. So an organic unit collapsed as whatever the
transporting flow happened to be doing, and there was no coal material in the
roster to collapse *to* even if it had been asked for.

## What the routing actually required

The nine-line version is: consult `tag.biota` first, and let it win where it is
inhabited.

```rust
match tag.biota {
    Biofacies::Coal => CLASS_ORGANIC_COAL,
    Biofacies::Peat => CLASS_ORGANIC_PEAT,
    Biofacies::Soil | Biofacies::Retro => CLASS_ORGANIC_SOIL,
    Biofacies::Charcoal | Biofacies::Mineral => /* the old env/energy rule */,
}
```

"Let it win" rather than "blend it in" is the load-bearing choice, and the
measured seam is what argues for it. Pull the S10 site's record up (world voxel
`(107338, 58787)`, seed `0x0D5E_ED57_2026`) and the 24 m seam's own tag is
`Sa/A/L·Co` — subaerial, **arid**, **low** energy. Under the old rule that is
`CLASS_CLASTIC_FINE`: mudstone. Not because the rule was sloppy, but because it
was answering a different question. Flow energy tells you how coarse the grains
were in whatever was being carried past. It says nothing whatever about the fact
that plant matter was accumulating faster than it decayed, which is the entire
reason there is a rock there at all. An organic unit is a *different rock*, not a
clastic one wearing a flag.

The arid tag is worth pausing on, because it looks wrong and is right. S10's
design choice 8 models waterlogging separately from rainfall: a wet mountainside
sheds water and grows forest; a low flat site collects upslope drainage and grows
peat. So coal swamps land on lowlands, not in the rain belt, and a seam can
carry an arid climate-at-deposition tag honestly. That in turn decided the
organic classes' formation windows — **precip is left open and depth does the
work**, because rainfall is genuinely not the control.

Everything else followed the existing pattern with no argument: three new class
constants, three new vanilla members, three property sheets, three block-tier
stand-ins, three placeholder texture packs. Classes-as-contracts held — the
class-share invariant test passes unchanged, and adding a second coal member
(the test does exactly this) diversifies the coal class without changing how much
coal exists anywhere.

## The measurement that chose the roster

The brief asked for coal "plus whatever the recorder can already distinguish and
the geology model can honestly express," and explicitly warned against inventing
discriminators the data cannot back. So before writing any material I wrote a
probe (`examples/organic_probe.rs`) to ask the only question that matters at the
collapse tier: **which facies can a 0.9 m voxel column actually hold?**

`deposit_deep_history` drops any recorded unit thinner than one voxel — a
condensed couplet the record keeps but blocks cannot resolve. That rule predates
this milestone and is roster-independent, which makes it a clean sieve. Running
it over the whole Medium deep grid:

| facies | units recorded | thickness | units surviving | thickness kept |
|---|---:|---:|---:|---:|
| Mineral | 328 668 | 1 006 252 m | 216 646 (65.9 %) | 989 664 m |
| **Soil** | 132 836 | 195 929 m | 64 411 (48.5 %) | 184 997 m |
| **Peat** | 3 140 | 575 m | 72 (2.3 %) | 85 m |
| **Coal** | 1 310 | 1 398 m | **1 174 (89.6 %)** | 1 340 m |
| **Charcoal** | 158 310 | 5 519 m | **0 (0.0 %)** | 0 m |
| **Retro** | 48 838 | 6 406 m | 2 930 (6.0 %) | 3 579 m |

That table wrote the roster, and it killed two materials I had already sketched.

**Coal ships, obviously** — 89.6 % of seams survive, and 96 % of the recorded
coal *thickness*. Coal is the one facies the deep sim makes thick enough to be
architecture rather than a lamina, which is exactly why it is the one you can
mine. **Carbonaceous mudstone ships** on the strength of the Soil row: nearly
half the horizons and 185 km of accumulated thickness survive, making organic
soil the second most abundant facies in the world after ordinary clastic. That
is a real appearance change and it is flagged for the user's eye. **Peat ships
too, but barely** — 72 units in the entire world. Peat that is both thick and
unburied is genuinely rare, because thickness plus burial is the definition of
coal; the rarity is the model being right, not the model being broken.

## What the record could not honestly justify

**Charcoal gets no material.** This is the finding I did not expect. 158 310 fire
beds in the record — the third most numerous facies by unit count — and **not one
of them survives the voxel quantization.** Mean bed thickness is about 3.5 cm.
The voxel is 90 cm. The fire record is two orders of magnitude below the
resolution of the world it is recorded in, and a charcoal *band* is therefore not
a thing that can exist in a voxel column. Shipping a charcoal material would have
been dead content: a class no unit ever reaches, a texture pack no player ever
sees, a line in the roster that lies about what the world contains. A
charcoal-tagged unit now reads as the mineral host it is a streak within.

The honest representation for a 3 cm bed already exists in this codebase and is
the *inclusion*: pore or debris eighths riding inside a host stratum, which is
how the placer puts gold in gravel and how 3d puts olivine in basalt. Charcoal
wants to be a few dark eighths in the mudstone above the burn, not a layer. I did
not build it, because doing so means changing the sub-voxel drop rule to
redistribute dropped units into their host — a mechanism change that touches
every dropped unit, mineral ones included, and needs its own invariant work. It
is filed to Sequenced with the measurement attached, which is a much better brief
than it would have had yesterday.

**Coal rank gets no members.** Lignite / sub-bituminous / bituminous / anthracite
is a real, well-understood progression, and it is genuinely controlled by burial
depth and temperature — so the temptation to ship a rank ladder discriminated on
the `depth_m` axis was strong, and the machinery would have accepted it happily.
It would have been fiction. Rank transitions live at roughly 1–2 km of burial.
The deepest overburden in our record is about 100 m. A depth window spanning our
actual data would put every seam in the same rank no matter how it was drawn, and
a window spanning the *real* range would be decoration on an axis the data never
visits. So vanilla ships one coal member and the class documents that **its depth
axis is the rank axis**, ready for the day the record carries kilometres. The
class-share invariant means a pack can add anthracite later without moving a
single existing seam.

**Retrogressive horizons get no distinct material.** `Retro` is one of S10's four
headline signals and it survives quantization tolerably (56 % of its thickness),
so this one is a judgement rather than an impossibility. A retrogressive horizon
is an organic soil horizon whose *community* is phosphorus-starved sclerophyll.
That is an ecological fact. Our property sheet has axes for density, grain size,
cohesion, extraction resistance, permeability, insulation, albedo — and none for
nutrient status. Real retrogressive surfaces do eventually diverge
mineralogically (bleached podzol E horizons, laterite), but the record does not
carry a weathering-product axis to justify picking one. So `Soil` and `Retro`
both fill `dc:stratum/organic-soil`, and the day we track weathering products is
the day that class gains a second member.

## A correction fell out of the flip

Flipping `production_config`'s `biotic` to true is one bool, and the ratified
cost was S10's headline: the world-creation ritual grows **15.17 s → 25.19 s**.
The user ratified that number explicitly, with the generous framing that world-gen
time is not a constraint we optimize against.

Measuring the shipped path, it is **10.82 s → 13.46 s** — and the full
`Pregen::run` ritual lands at 13.79 s. Same seed, same 297 025 cells, same 460 m
A tier, same 200 iterations, same config values.

The difference is not the machine and not a regression: S10's cost table was
produced by `examples/biotic_spike.rs`, which calls `deeptime::run` — and
`deeptime::run` is the **scalar** path. Production calls `build_field`, which
uses the byte-identical **parallel** path S9b built and proved. The spike measured
a worst case and reported it as the ritual; the shipped ritual is a little over
half of it. Nobody was wrong, but the number the user ratified is not the number
they will experience, so it goes in corrections.md (#12) rather than quietly into
this entry. Biology's marginal cost on the shipped path is **+2.6 s**, not +10 s.

I have deliberately not amended docs/spikes/S10-results.md: a spike result is a
dated record of what was measured, and its table is correctly labelled as the
harness's. The correction points at it.

## What a player can now do

Stand at world voxel `(107338, 58787)` on seed `0x0D5E_ED57_2026` and dig down.
The collapsed column reads, top first:

```
   3 vox  dc:geo/mudstone       [stratum/clastic-fine]
   2 vox  dc:geo/sandstone      [stratum/clastic-coarse]
   1 vox  dc:geo/conglomerate   [stratum/clastic-coarse]
   5 vox  dc:geo/sandstone      [stratum/clastic-coarse]
  27 vox  dc:geo/coal           [stratum/organic-coal]   <-- the 24 m seam
   7 vox  dc:geo/conglomerate   [stratum/clastic-coarse]
   2 vox  dc:geo/coal           [stratum/organic-coal]   <-- the lower seam
   1 vox  dc:geo/mudstone       [stratum/clastic-fine]
  ... marine mudstone and siltstone below
```

Twenty-seven voxels of coal, as `Block::Coal` and as `MaterialId::COAL` in the
voxel contents, under eleven voxels of alluvial cap, over a second seam and then
the marine section. Yesterday every one of those sixteen events was clastic. The
seam is soft for a rock, too — coal's smash resistance is 2.2 against granite's
5.5, straight off the real property sheet — so it yields to a tool that would
barely scratch the sandstone above it. That is the payoff the spike's ten seconds
were bought for, and it is the first time the deep-time sim's *biology* has
produced something a player's hands can reach.

## Files

`materials/mod.rs` (3 materials + property sheets), `materials/geology.rs`
(3 classes + 3 vanilla members, roster-agnostic order test), `voxel.rs`
(3 blocks), `worldgen/geology.rs` (`deep_class` — the routing), `pipeline.rs`
(the clastic pass now declares the organic classes it selects),
`collapse.rs` (block tier), `deeptime/field.rs` (**the flip**), `dc-api/host.rs`,
`dc-client/meshing.rs` + `terrain_material.rs` + `terrain_fullbright.wgsl`
(atlas 26 → 29 layers), `tools/gen_placeholder_textures.py` + 3 new packs,
`tests/organic.rs` (new), `examples/organic_probe.rs` (new).
