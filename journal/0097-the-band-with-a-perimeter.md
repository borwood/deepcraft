# 0097 — The band with a perimeter

*(The flag-ON appearance walk owed by journal/0094 — Movement 3's
`--weather-inventory` band, walked and blessed. The band is real, ~7× the
magnitude the ROADMAP claims for it, and **wrong in a way only a walk could
show**: it has a hard edge. The user's finding, and the follow-up it opens.)*

> blogworthy: **respect for earth and anthropological processes** (lens 4) — a
> simulation can be right about *how much* and still be wrong about *what a
> thing is*. The weathering front got its magnitude honestly, from a real time
> integral, and then expressed it as a sharply-bounded slab of one material
> abutting pristine bedrock — which is not what a weathering front is. Also
> lens 3: the most realistic-looking part of the picture turned out to be a
> **quantization artifact**, doing aesthetic work the physical model wasn't.

## What we set out to bless

ROADMAP § APPEARANCE WALKS OWED item (2): Movement 3 made the saprolite band
≥1 voxel at production scale, so the flag-ON walk was ready. Default off is
byte-identical; the band is a walk-gated appearance flip, the user's to bless.

`examples/s18_weathering_tour` already existed and printed ranked stations, so
the tour map cost nothing:

```
banded cells:  71748 / 297025  (24.2%)
band thickness: max 6.09 m, mean 1.61 m over banded cells
max in voxels:  6.77 voxels @ 0.9 m/voxel
STATION 1 — deep cell (439,276) | world (84185 m, 9212 m)
           surface 273.1 m, regolith cover H 1.32 m
```

Worth recording plainly: **the ROADMAP undersells this by most of an order of
magnitude.** It states the M3 result as "≥1 voxel at production scale" — the
A-3 guard's threshold, which is what the test asserts. The measured argmax is
6.77 voxels and a quarter of all land carries *some* band. The guard's floor got
written down as if it were the finding.

## The control is the whole argument

A screenshot of a band proves a band exists, not that the flag made it. Since
flag-off is byte-identical by construction, the honest instrument is the **same
column, same seed, flag flipped**:

| voxel | `--weather-inventory` ON | OFF |
|---|---|---|
| y=300 (veneer) | mudstone 2/8 + sandstone 1/8 + siltstone 1/8 + carb-mudstone 2/8 | **identical** |
| y=299 | mudstone 6/8 + peat 2/8 | air |
| y=298–293 | **loose `dc:mudstone` 8/8** | air |
| y=287 ↓ | `dc:stone`, contents-free basement | **identical** |

Seven voxels ≈ 6.3 m against a predicted 6.09 m, in the right place (basement
contact, under the veneer), with the veneer above and the basement below
unmoved. The flag's product, and only the flag's product.

Screenshots: `journal/assets/0094-weathering-band-flag-{on,off}.png` (named for
0094, the entry whose walk this is) and `-pit-topdown-on.png`. Instrument
`--fullbright` — a material question, so flat albedo, per journal/0030.

## The user's finding: a front is not a slab

Then the user looked at the ON frame and named what is wrong with it:

> the band of mudstone/sandstone appears as if using an in-place
> structure→structure edge, instead of a more appropriate proxy like
> structure→pore_fill. The layer of degraded bedrock has a hard perimeter and
> then pure bedrock, which does not make sense for the natural process it
> claims to model. I would expect to see some mix of bedrock: some speckle,
> because nature does not in-place degrade a bulk unit of rock to another via
> weathering.

This is right, and the code says exactly why. The inventory edge is honest at
its own tier — `weather_inventory.rs` moves `(BEDROCK_SEAM_MATERIAL,
Structure) → (BEDROCK_SEAM_MATERIAL, Loose)`, a **form** change on one material,
one fact per agent, mass-conserving. But the collapse-tier consumer
(`geology.rs::emplace_weathering_front`) folds the whole ledger into a **single
stratum of one class** — `CLASS_CLASTIC_FINE`, thickness `deep_weathering_m` —
pushed at the base of the pile. The record→voxel path then expresses a stratum
whose span wholly contains a voxel as `Single`: **8/8 of one member**. So:

- band interior → pure `dc:mudstone` debris, 8/8, no bedrock anywhere in it
- band bottom → against **contents-free** basement (unrecorded `dc:stone`),
  which cannot mix with anything, because it has no contents to mix

A hard perimeter top and bottom. What the model computed was a *rate integrated
over depth and time*; what got emplaced was a **slab**. The weathering front —
the thing that makes saprolite legible as saprolite, a downward gradient from
intact rock through corestones and grus to clay — is precisely the part the fold
discards. `weathering_product_m` is a scalar; a scalar cannot carry a profile.

The user's proposed proxy is expressible in **today's** vocabulary, which is
what makes it a follow-up and not a research project. `VoxelContents` already
carries `structure[]`, `pore_fill[]`, `open_pores`, `debris[]` and eighths —
the walk read them straight off `world_get_contents`. Degraded bedrock as
*retained structure with weathering product in its pores*, the structure share
falling with height above the front, is a thing the contents model can already
say. Today's band says `structure: []`, `debris: [mudstone 8/8]` — the parent
rock is simply gone.

## The speckle that looks right is an artifact

The user also noticed, honestly and with the right hedge — *"I can see that the
mudstone/sandstone speckle/mix with the veneer one layer down, which looks
realistic though I am unsure the mechanism"* — that the band's **top** contact
does mix, and reads well.

It does, and the mechanism is not weathering. `collapse.rs::mixed_voxel_contents`
is "the addressed stochastic allocation of `n` eighths among the events
overlapping its span" (`allocate_partial` over `fill_draw`, journal/0055). A
voxel that **straddles** two strata events gets eighths of both, position-
addressed and stable. That is why y=299 came back `mudstone 6/8 + peat 2/8`: it
straddles the band top and the veneer above it. It is **boundary quantization**,
one voxel thick wherever any two units meet — nothing to do with weathering, and
it would look identical at the contact of two units that never interacted.

So the one part of the picture that reads as a natural gradational contact is an
artifact of the 0.9 m grid, and it is exactly one voxel deep. This is the
uncomfortable version of the lesson: the artifact was flattering, and flattering
artifacts are how a wrong model survives a walk. The user caught it by asking
what the mechanism *was* rather than accepting that it looked right.

## Wrong turns worth keeping

**`world_scan_region` is blind to stratigraphy.** The first instinct — scan the
column, find the band — returned `palette: ["dc:stone"]` for every voxel from
the veneer to the basement. `scan_region` and `get_block` answer with the stored
1-byte `Block` summary, which collapses mudstone, sandstone, siltstone,
carbonaceous mudstone and granite alike into `dc:stone`. The walk was reading a
summary and would have concluded "no band" from an instrument that structurally
cannot see one. `world_get_contents` is the honest surface and shows all of it.

This is ARCHITECTURE's *"a summary is not an authority"* caught **in the field,
by an agent using the tool wrong**, and it is a live argument for the
`identify(pos)` arc (ROADMAP: the honest identity surface): the tier-flagged
answer is what a walk needs, and the untiered `Block` token is a trap for
exactly the reader who does not already know it is one.

**The contents record and the physics disagree.** At an untouched column in the
**flag-OFF** control, `world_get_contents` reports `dc:air` for voxels 288–299
while `client_player_pose_set` reports `eye_in_solid: true` at 296 and 291 — ~11
voxels of column where the record says empty and the world is solid. It is in
the control run, so **Movement 3 did not cause it**; the ON band then fills that
same range with real recorded material, which is how it nearly got credited to
the flag. It is adjacent to the known bare-cell fallback (*"a walker stood on
paint over nothing"*, `record_hole_probe.rs`, ROADMAP Observed) but not the same
shape — that one is paint over *stone*; this is the record reading empty over
*solid*. Reported, not diagnosed: the walk reports, the diagnosis measures.

**Tooling friction, all avoidable.** `yaw`/`pitch` are **radians** (CLAUDE.md
says "negative pitch looks down" without the unit; `-10.0` clamps to −1.55 rad,
straight down). Voxel y = metres / 0.9, and pose speaks metres while
fill/scan/contents speak voxels — mixing them probes 30 m off. A small pit
frames badly for a cross-section; `world_fill` of a **bench** (~20k voxels of
`dc:air`) gives a road-cut face that reads at a glance. Screenshot names must be
a bare lowercase slug.

## Status

**ACCEPTED FOR MERGE** (user, 2026-07-25). The band is blessed as a real,
visible, correctly-magnituded appearance flip; the hard perimeter is a
**follow-up, not a blocker**. Stub #16 (the flat granite basement stand-in) is
untouched and still names the material identity heir.

### What this owes the ROADMAP

*(Written here rather than applied, because the main session holds the merge and
was editing ROADMAP concurrently — see § Numbering below.)*

1. **Shipped / APPEARANCE WALKS OWED item (2)** → **DONE, walk-confirmed
   2026-07-25** (user). Cite the measured numbers (max 6.09 m / 6.77 voxels,
   24.2 % of land banded) and the asset pair; correct the "≥1 voxel" phrasing,
   which quotes the guard's floor rather than the result.
2. **Sequenced — the weathering front needs a PROFILE, not a slab.** The fold
   `weathering_product_m` → one `CLASS_CLASTIC_FINE` stratum discards the
   depth gradient that makes a front a front. Heir shape (user-proposed):
   `structure → pore_fill`, retained parent structure with product in the
   pores, share falling upward through the front — expressible in today's
   `VoxelContents` vocabulary. Couples to stub #16 (a rind's material identity)
   and to the deep-cell inventory's form vocabulary.
3. **Observed — the contents record reads empty over solid ground** (flag-OFF
   control, untouched column, ~11 voxels; `get_contents` air vs `eye_in_solid`
   true). Adjacent to but distinct from the bare-cell fallback.
4. **Observed — `world_scan_region` / `world_get_block` answer with the `Block`
   summary** and are structurally blind to stratigraphy; an agent walk read
   `dc:stone` for the whole column. Feeds the `identify(pos)` arc.
5. **CLAUDE.md doc gaps** (§ Agent walks): pitch/yaw are **radians**; and pick
   `world_get_contents`, never `scan_region`, for any material question.

### Numbering

Written as **0097** because the main session shipped **0096** (flux on faces)
while this walk was running. Two sessions appending to one append-only journal
is a numbering hazard the wrap ritual already knows about; the walk session
checked at write time rather than at start time.
