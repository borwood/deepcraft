# 0098 — the head field, and 232 artesian basins that were rounding error

*2026-07-25. FLOW continuation (a): the potential. Slice 1 recorded flux on 3D
faces and left the **vertical** ones structurally present and honestly zero,
because the drainage solve had no vertical term and there was no number to write.
This is the term that fills them — and the field that produces it had to be a
**potential**, not an elevation, or the whole exercise would have been `y + sat`
with a new name. The interesting part is not the solve; it is that the first
version reported 232 artesian columns and every single one of them was a
2.5×10⁻⁵ m priority-flood residue.*

> blogworthy (lenses: deepsim-reflexions; earth-respect; procgen-against-priors):
> *the number that agrees with your hypothesis is the one to distrust.* The brief
> asked for an artesian case. The first run produced 232 of them on a fixture with
> 269 land cells — 86 %, which should have read as a triumph and instead read as
> absurd. Chasing it down turned out to be the whole slice: it exposed a stale
> coarse-rate export, a lake test that compared two different moments, and the
> quiet fact that a seepage cap makes every capped cell a Dirichlet boundary for
> its neighbours.

## What "head" had to be, and why elevation would not do

flow.md § 2.4 is unusually blunt: *"flow goes downhill" forecloses artesian
basins, capillary rise, thermohaline circulation and density/turbidity currents.*
The existing bound-water proxy is `H = y + sat`, documented in the hydrology
priors as explicitly **unconfined, not a Darcy solver**. Head *is* the elevation
there. So a confined aquifer whose water stands **above** its own ground is not
"poorly modelled" — it is inexpressible, in the same way the Mississippi delta was
inexpressible in a receiver tree (journal/0095). Two slices, the same shape of
defect: the primitive, not the tuning.

The rule I wrote down before writing any code:

> **head = elevation + pressure head.** Where the water is unconfined and touching
> a free surface, the pressure term is zero and head *coincides* with an elevation
> — that is physics, not an assumption. Where a confining bed caps a permeable
> one, head is set by the recharge area up-dip, transmitted through the bed, and
> is free to exceed the local ground.

Everything else followed from taking that seriously.

## The solve reads the record, and the record already knew

The part I expected to be hard — "where do I get permeability at 460 m cells?" —
turned out to be already sitting there. Every `DepUnit` carries a measured tag;
`litho_of_tag` resolves it to a `Litho`; every `Litho` has a reference material;
every material's property sheet carries **`permeability`**. The same chain the
erodibility coupling reads for abrasion resistance. So the column's hydraulics are
**derived, never stored** (S-2), and erosion, the collapse tier and the head field
cannot disagree about what a bed is made of, because they all ask the same
function.

And the sheet's numbers straddle the question cleanly, which felt like a gift:

| lithology | reference material | k |
|---|---|---|
| coarse clastic | sandstone | **0.35** |
| fine clastic | mudstone | **0.02** |
| basement | granite | 0.02 |

A 17× contrast, with `deep_class` routing *subsea* deposition to fine clastic and
*subaerial high-energy* to coarse. Which means: **a marine transgression over a
fluvial bed is a confined aquifer**, and the sea-level sinusoid has been laying
them down for two hundred epochs without anyone asking it to. There is no
artesian-basin code path. There is a mud cap over a sand bed, which is what an
artesian basin *is*.

The field itself is the steady groundwater equation `∇·(T ∇h) = 0`, relaxed by
Gauss–Seidel with **alternating forward/reverse raster sweeps** — a fixed 24, not
a convergence check, because the sim logic must be deterministic and free of
data-dependent iteration. Alternating direction is what makes 24 enough: a reverse
sweep carries a boundary value across the whole grid in one pass, where Jacobi
would need O(w²) to diffuse it that far.

The boundary conditions are where the physics lives:

| where | condition |
|---|---|
| submerged, or the domain border | **Dirichlet** at the sea stand / base level |
| **unconfined** with free water — a lake, or a stream carrying ≥ 25 cells of drainage | **Dirichlet** at that water's elevation: the table *outcrops* |
| **unconfined** elsewhere | free, **capped at its own ground** — a water table cannot stand above the land, it discharges |
| **confined** | free, and **uncapped** |

That last row is the entire artesian mechanism. **Artesian is not a special case
in this code — it is the absence of a cap.** I am fond of that, because it is
exactly the shape flow.md § 8 asks for: no landform-specific path.

## The 232 basins that were not there

First production-shaped run, small fixture: **269 subaerial columns, 232
artesian, 0 with the table below ground.** Which is nonsense — a landscape cannot
have 86 % of its cells discharging. But every one of them passed the "is it
confined" check I had written, so the invariant was not catching it.

The debug dump ended the argument in one line:

```
cell 22236: ground -4.386663  head -4.386638  confined false
```

Twenty-five *microns* of excess head, on an **unconfined** column. Three defects
stacked, and each is worth stating because each looks like working code:

**1. The exported field was from a different moment than the exported surface.**
The head pass is coarse-rate (period 20), so the plane the loop leaves behind was
relaxed at epoch 180 — twenty epochs of uplift and incision before the terrain the
field actually ships. A consumer reading `head` beside `surf` would be comparing
two different worlds. Fixed by re-relaxing once post-loop on the final state,
exactly as coal promotion already runs post-loop. This costs nothing already
recorded: the flux record's vertical faces were accumulated per epoch against
*contemporaneous* ground and the archive is closed before this runs.

**2. Standing water was tested against the wrong pair.** I had `free = filled[i]`,
where `filled` is the depression-filled surface. In-loop that is fine — the head
pass runs right after drainage, so `filled` and `R + H` are the same instant. At
finalize they are not, and `filled > ground` becomes true for **every cell erosion
has since lowered**. A world of fictitious ponds. The fix is to carry the
*difference*: `filled − routed` comes from one solve, so it is a self-consistent
lake depth that stays meaningful applied to a ground from a later moment. A small
thing that generalises — **when two quantities must be compared, carry their
difference, not one of them.**

**3. And then the residue.** Even self-consistent, priority-flood leaves 10⁻⁵ m
dust on cells that are not ponded at all. Pinning the table at `ground + 2.5e-5`
is artesian by the letter of the comparison and nonsense by the metre.
`PONDED_MIN_M = 0.01` floors it — one centimetre, below anything the world can
express (a voxel is 0.9 m) and far above the flood's residual.

After all three: **219 of 269 subaerial columns with the table below ground, 50 at
ground, 0 artesian on the fixture.** A real potential, and an honest zero.

**And then a fourth, found only because the invariant was written as a test over a
whole world rather than a constructed column.** A **lake** cell is pinned at its
water *surface*, which stands above the ground by construction — that is the lake,
and it is correct. So `head > ground` is not by itself artesian. On the production
world that distinction is not cosmetic: it is the difference between **308** and
**60**. The 248 lakes were being counted as aquifers by a comparison that could not
tell them apart, and the fix is to ask `DeepField::lake`, which the flux slice
already exports. The general lesson is the same one as defect 2 — *a comparison
between two quantities is only as honest as the third thing it forgot to ask about.*

## What the production world actually has

`examples/head_field_probe.rs`, seed 1337, `Extent::Medium`, production flags.
545 × 545 = 297,025 cells, 8 chapters, 200 epochs.

```
vertical entries now                         :   307,364
  DOWN (infiltration / recharge)             :   306,227
  UP   (artesian rise / a spring's last step):     1,137
columns carrying any vertical flux           :   131,586   (44.301 % of cells)
magnitude   min 0.000004   mean 0.501013   p95 0.720093   max 10.000000
  DOWN      min 0.100000   mean 0.502773   p95 0.720324   max 10.000000
  UP        min 0.000004   mean 0.027050   p95 0.084889   max  0.297483
```

**307,364.** Slice 1's column read zero — not "small", zero, because no term
existed that could write it honestly. Forty-four percent of the world's columns
now record what crosses their top face.

And the artesian question, which is the one I was most prepared to answer with a
disappointment:

```
subaerial columns                                            44,496
  CONFINED (>= 5 m low-k cap over a permeable bed)            1,044   (2.3 % of land)
  water table BELOW ground                                   24,935
  holding a LAKE (above ground, and correct)                    248
  ARTESIAN (head ABOVE the local ground)                         60
excess head above ground (m)  min 0.002  mean 0.654  p95 2.105  max 2.935
```

**It occurs naturally.** 60 columns, up to 2.94 m of excess head. The strongest:

```
cell (368,317) ~(169.3 km, 145.8 km)  ground  -1.6 m  head   1.3 m  EXCESS 2.94 m  cap  6.7 m
cell (332,303) ~(152.7 km, 139.4 km)  ground 120.2 m  head 122.3 m  EXCESS 2.16 m  cap 24.5 m
cell (300,299) ~(138.0 km, 137.5 km)  ground 105.8 m  head 107.9 m  EXCESS 2.10 m  cap 14.5 m
```

Not one of those is expressible under `H = y + sat`. The second is the textbook
picture: a hundred and twenty metres up, **twenty-four metres of mud over a sand
bed**, and the water in that bed standing two metres above the hillside. Drill it
and it flows.

And the counterpart the same field gives for free — the deepest water table on the
world, at cell (249, 239), 1169 m up: **the table sits 15.9 m below the ground**,
in an unconfined 52-unit column, recharging downward at ~0.65 units every chapter
for all eight. A depth to water that is *derived from the rock*, not from
present-day precipitation. That is the number `y + sat` never had.

The excesses are **metres, not the hundreds of metres of a Great Artesian Basin**,
and I want to be precise about why rather than let the number imply more than it
earns — see the seams below.

## The thing I got to not invent

The vertical magnitudes had to be in *some* unit, and the obvious move was to give
them their own — metres of water — and let the record carry two currencies with the
face key as the discriminator. I disliked it: it makes `magnitude` mean different
things in different rows, which is one short step from the § 11.3 failure where a
consumer must know how a world was generated in order to read it.

The escape was noticing that **gravity drainage through the vadose zone runs at a
unit hydraulic gradient**. Darcy with `∂h/∂z = 1` gives `q = k`, the property
sheet's `permeability` is already a relative 0..1 number, and the drainage solve
already seeds exactly `1.0` unit per cell per epoch. So:

> **one unit = one cell-epoch of the seeded source**, and a unit gradient through
> unit relative permeability moves one of them.

One normalisation, stated once, no fabricated depth scale, and the vertical faces
land in the same currency as the lateral ones. The record needs no mode flag,
because there is no mode. That is the kind of thing that only shows up if you
refuse the first answer.

## What is a seam, and who inherits it

- **Recharge.** The solve is the **R = 0 steady limit** — the potentiometric
  surface interpolated through the material between fixed free-water boundaries.
  `∇·(T∇h) = −R` needs `R/T` in real units: a real hydraulic conductivity *and* a
  real precipitation depth. `grid.precip` is normalized 0..1 with no depth scale,
  and inventing one would have made the acceptance number a knob. **This is why
  the excess heads are metres rather than hundreds of metres** — with no recharge
  mounding the table under interfluves, the potential a confined cell inherits is
  its neighbours', not a distant highland's. Heir: a real water-balance climate
  (the same missing scale that keeps the lateral source uniform).
- **Precipitation weighting.** journal/0096 flagged that a precip-weighted source
  "arrives with the head field". It does not, and deliberately: weighting only the
  *vertical* half would make the record internally inconsistent, and weighting the
  lateral half changes `area`, which feeds stream power, which moves terrain. It
  belongs with the MFD slice, which is already changing routing.
- **Buoyancy / density.** `head = z + p/(ρg)` with ρ constant (`FLUID_DENSITY_REL`).
  Named and *multiplied in* rather than folded away — `x * 1.0 == x` is provably
  inert, and the term now has somewhere to land. Heir: continuation (d), fluid
  identity (brine, thermohaline, turbidity).
- **Darcy, the parts of it that are missing.** The lateral solve *is* steady
  confined Darcy and the vertical term *is* Darcy's 1-D law; what is absent is
  **transient storage, unsaturated (Richards) flow, and anisotropy**. Capillary
  rise specifically needs the unsaturated half.
- **Compaction.** flow.md § 10.2 arms this trap in advance. The field stores **no
  elevation** — it stores a potential, recomputed from the live surface and the
  live record at its own cadence — so when thickness becomes a function of burial
  depth, nothing here goes stale.
- **MFD.** Head is what makes a multi-flow-direction partition possible, and
  therefore *simultaneous* divergence (flow.md § 2.6 — today's divergence is all
  temporal avulsion). Explicitly the next slice. Lateral routing here is untouched
  and the terrain is byte-identical, proven by name.
- **The bound occupancy.** Every entry this slice writes is still `FlowForm::Free`.
  The exchange crosses a column's top face; turning it into a genuine free↔bound
  *edge* with void intervals is continuation (c), which also owns the conduit
  pairing rule.

## A rule I did not have to invent, and one place flow.md was silent

Vertical faces pair **within** a column, so the slot-pairing question that § 2.2
leaves open — and that § 11.5 answered in two modes with a third assigned to (c) —
simply does not arise here. The entry binds to the chapter's slot exactly as every
other entry does, and that is not a convenience: **during chapter K, chapter K's
unit *was* the contemporaneous land surface**, so flux crossing its face is
precisely that moment's recharge or discharge. The atom fit without being bent.

Where flow.md is silent: it names head as *"elevation + pressure, and for density
flows a buoyancy term"* and stops. It does not say what supplies the boundary
conditions, and that is where all the modelling judgement actually lives — the
choice to pin at streams and lakes, the seepage cap, and the decision to leave
confined columns uncapped are mine, argued in the module docs, not ratified. They
are not deviations from the document; they sit underneath it. **No PLEA.**

## Cost, measured

```
head plane (297,025 x 8 B)     2.27 MiB
new vertical entries x 16 B    4.69 MiB   (307,364 entries)
DeepField without the head   149.21 MiB
DeepField with    the head   156.16 MiB   = 1.0466x   (+6.96 MiB)
flow record total             45.35 MiB   (was 40.66)
face sparsity                  9.3806 %   (was 8.386 %)
gen time                       +0.5 s on a 19.6 s deep-time run
```

The denominator is exactly the 149.21 MiB the integrator re-measured on merged
main, so the ratio is honest against today's world rather than a forked one. No
`Vec<Vec<_>>` anywhere: the vertical entries ride the existing flat-array + CSR
record, and the field is one exact-sized plane.

## Shapes

Rides the **field/cellular split** (material-behavior.md §5 — this computes a
field, plants it, and runs **no edges**), **§14** (a condition-field with an opaque
id, `dc:field/head`, joining `dc:field/temperature`), **S-2** (the column's
hydraulics are derived from the record every time, never stored), **S-9**, and the
north-star **pass-runner** (`dc:deep/head` declares `{reads: [Routed], reads_prev:
[Recorded], writes: [Head]}` over plain data, opaque ids and a bare `fn` body; no
closures cross the seam).

Discharges **A-4**: the vertical zeros were a declared, honest hole and this is the
term that fills them — and the edge is *declared*, not inherited from a tie-break
(`dc:deep/flow_record` reads `Head`, pinned by a test, after the spine-audit lesson
that an undeclared order is a comment rather than a guarantee).

Guards **A-3** (the acceptance is 307,364 crossings and 60 artesian columns on a
production world, not a green unit test) and **A-1** — the honest empties that
remain are asserted *as* empty by name: vertical faces carry **no load**, because
the bound phase carries solute and dissolution is dormant until (c). The slice that
populates it must delete that assertion on purpose, which is the discipline slice 1
set and this slice was the first to be held to.
