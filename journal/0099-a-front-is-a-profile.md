# 0099 — A front is a profile

*(The follow-up journal/0097's walk opened, ratified 2026-07-25. Movement 3's
saprolite band was correctly **magnituded** and wrongly **shaped**: a slab of one
material with a hard perimeter, abutting contents-free basement. This slice
re-expresses the same mass as a graded profile — retained parent structure with
weathering product in its pores, the product share rising with height.)*

> blogworthy: **respect for earth and anthropological processes** (lens 4) — the
> difference between *how much* rock weathered and *what a weathering front is*.
> The model had the first right and expressed the second as a slab; the fix is
> not more simulation, it is refusing to let a scalar be a shape. Also lens 3:
> the mechanism the fix needed was **already in the codebase**, wearing an
> igneous name (`accessory`, the pore-slot rider). The instinct to add a third
> rider field next to the two that existed is exactly this project's
> characteristic failure, and the honest read of "what shape is this?" avoided
> it.

## The defect, stated precisely

The inventory edge is honest at its own tier: `(GRANITE, Structure) → (GRANITE,
Loose)`, a **form** change on one material, one fact per agent, mass-conserving.
The loss was entirely at the **fold**. `geology.rs::emplace_weathering_front`
read the scalar `FactLedger::weathering_product_m` and pushed **one stratum of
one class** at that thickness. `ColumnFill` then expresses a span that wholly
contains a voxel as `Single` — 8/8 of one member. So the band read:

- interior: pure `dc:mudstone` debris, 8/8, **no bedrock anywhere in it**;
- bottom: against contents-free basement, which cannot mix, because it has no
  contents to mix.

A hard perimeter top and bottom. The user's finding, and the sentence the slice
is built around:

> nature does not in-place degrade a bulk unit of rock to another via weathering

What the deep model computed is a *rate integrated over depth and time*. What got
emplaced is a slab. **A scalar cannot carry a profile** — so the profile has to be
imposed by the consumer that knows the geometry, which is the collapse tier.

## The mechanism was already there

The user's proposed proxy — `structure → pore_fill` instead of
`structure → structure` — is expressible in today's vocabulary, and that is what
made this a follow-up rather than a research project. But it is better than that:
`StrataEvent` **already had the field**. `accessory` is `(member, eighths per
voxel)` carried in the host rock's pore slots, built for the sparse igneous
inclusion (olivine in granite, 1/8, journal's 3d pore partials). Degraded bedrock
is the same shape at a different magnitude: parent structure, second material in
its pores, 1/8 … 7/8.

So the front is **eight bands of the parent rock, each carrying the weathering
product as a pore rider**, and the whole slice reduces to: choose the shares,
choose the parent, and make the record→voxel path carry a rider it was dropping.
No new field, no new expression path. The `accessory` doc comment now names both
riders, because a field with one inhabited use case is how a mechanism gets
re-invented next to itself.

## Why the shape is a *shape*, and the ledger sets only its size

The profile is a constant, in eighths, top-down:

```
WEATHERING_PROFILE = [7, 5, 4, 3, 2, 1, 1, 1]   // round(7 · exp(−j/3))
```

An exponential because a weathering front is a **reaction front**: the reactant
(oxygenated, acidic meteoric water) arrives from above and is consumed going
down, so the degree of alteration decays with depth — first-order kinetics, the
form every saprolite profile shows, reading upward as intact rock → corestones →
grus → clay. The array is a *cache* of that function, not a hand-drawn curve;
`the_front_profile_is_the_exponential_decay` asserts the two agree, so the shape
stays auditable and the generation path stays a constant.

The top is **7/8, never 8/8**, deliberately. Saprolite is defined by retaining
the parent's fabric; a voxel of pure product with no relict rock in it is mobile
regolith, not a front. "No bedrock anywhere in the band" was the defect.

The ledger sets the *size*, and only the size. One line does it:

```
band_m · Σ(PROFILE / 8) = deep_weathering_m
```

so the product integrates back to exactly the metres the deep tier committed, and
the **front** is `8·8/24 ≈ 2.67 ×` thicker than the old product band. That is not
inflation — it is the physical statement that a front of that thickness has
converted that much rock. It costs nothing above ground, because `ColumnFill`
slices the record downward from the surface: a longer record grows **down** into
what was unrecorded basement, and nothing above the front moves.

## The parent is the rock underneath, not a second draw

The first version selected the parent from `CLASS_IGNEOUS_INTRUSIVE` with its own
draw address — a fresh opinion about what the basement is made of, next to the
basement the column had already recorded. The probe showed why that is wrong: at
a station with an andesite flow under the front, the front would have been made
of granite sitting on andesite. A second perimeter, one tier down.

So the front **inherits** — member *and* selection address (`sel_salt`,
`sel_tag`, `depth_m`) — from the last structural event laid before it, which
(the igneous pass runs first) is the top of this column's own basement body.
Inheriting the address matters as much as the member: `dithered_member`
re-resolves the member per voxel-column off exactly those fields, so the front's
retained structure is the *same rock as the basement immediately below it, per
voxel column*, not merely the same class. At station B the bottom contact reads:

```
1049   andesite 7/8   mudstone 1/8    <- deepest front voxel
1048   andesite 7/8   olivine  1/8    <- the basement body itself
```

There is no contact there any more. It is one rock with a changing pore fill,
which is what a weathering front is.

Where no basement body was recorded (the column's province gets none), the parent
falls back to a basement-class selection — the deep tier asserts one flat granite
basement everywhere anyway, which is stub #16 and stays stub #16.

## The mixed path was silently dropping the rider

Two record→voxel changes were needed, and both are the kind that only show up
when you look at the numbers.

**`ColumnFill::build` merged spans by member.** The front is eight bands of *one*
member whose pore share differs per band — the entire gradient lives in the
difference the merge was erasing, attributing the topmost band's share to every
band's eighths. The key is now `(member, accessory)`. Widening it moves no
existing world: only the two igneous passes set an accessory, and they select
from different classes, so no pre-0099 column can hold two same-member events
that differ in it.

**`mixed_at` ignored `accessory` entirely** — documented, on the grounds that
accessories ride thick voxel-aligned bodies. With a front, roughly half the
voxels are contacts, and dropping the rider there would express them as pure
parent rock and lose the mass the ledger conserves. A **loose** rider is now
carried through, with its share **proportional** to the eighths its host actually
won — `cnt · k/8`, stochastically rounded on a low digit of the voxel's own fill
draw (the high digits are already spent by `allocate_partial`; reusing them would
correlate "this band won an extra eighth" with "the product won an extra eighth
of it" into a pattern). The existing `ore` rider keeps its historical `min`
semantics untouched: changing them would move every placer voxel in the world.

A *structural* accessory is still dropped at a contact. That residue is filed on
stubs.md #19 rather than fixed, because fixing it moves the goldens for a mineral
speck.

## What it looks like

`examples/weathering_profile_probe` ships in the slice and reads the production
world (seed 1337, `Extent::Medium`, `weather_inventory` ON) at the argmax cell —
the same station 0097 walked. Voxel by voxel, top to bottom:

```
   voxel     plan  structure        pore_fill        debris
     299    Mixed  -                -                mudstone 4/8 + granite 1/8 + siltstone 1/8 + peat 2/8
     298   Single  -                -                mudstone 7/8 + diorite 1/8
     297    Mixed  -                -                mudstone 6/8 + granite 2/8
     296   Single  -                -                mudstone 5/8 + diorite 3/8
     295    Mixed  -                -                mudstone 5/8 + granite 3/8
     294   Single  diorite 4/8      mudstone 4/8     -
     293   Single  diorite 4/8      mudstone 4/8     -
     292    Mixed  granite 4/8      mudstone 4/8     -
     291   Single  diorite 5/8      mudstone 3/8     -
     290    Mixed  granite 6/8      mudstone 2/8     -
     289   Single  diorite 6/8      mudstone 2/8     -
     288    Mixed  granite 6/8      mudstone 2/8     -
     287   Single  diorite 7/8      mudstone 1/8     -
     286…281 Single  diorite 7/8    mudstone 1/8     -
```

Product eighths, top → bottom: `[4, 7, 6, 5, 5, 4, 4, 4, 3, 2, 2, 2, 1, 1, 1, 1,
1, 1, 1]` over **19 voxels** (17.1 m), against a 6.09 m ledger band that the slab
expressed as 6.77 voxels of pure mudstone. Product expressed: 55 eighths = 6.19 m
against the ledger's 6.094 m, **+1.5 %**.

Note the form flip at 295/294. Above it the product exceeds half a voxel and the
rock has no skeleton left to call structure — it expresses as **debris with
parent clasts in it**, which is what grus and corestones in clay are. Below it
the rock still stands and the product is **pore fill**. The same ≥4/8 rule
`fill::mixed_contents` already applied at contacts, now applied by
`contents_for_event` too, so the two expression paths cannot disagree about the
form of the same band.

## Telling the gradient from the artifact

0097's uncomfortable lesson was that the most convincing part of the picture — the
mixed contact at the band top — was **boundary quantization**, one voxel deep,
present wherever any two units meet, and identical for two units that never
interacted. A gradient that is really that artifact would be indistinguishable in
a screenshot, so the probe prints the **fill plan** beside every row and three
things separate them:

1. **It is not one voxel.** Nineteen at the argmax, four at the median station.
2. **`Single` rows carry the composition too.** A `Single` plan is a voxel wholly
   inside one recorded band — no straddle, nothing to quantize — and the ladder
   `7/8 → 5/8 → 4/8 → 3/8 → 2/8 → 1/8` is read off `Single` rows.
3. **It scales with the model's own magnitude.** The argmax cell (6.09 m) grades
   over 19 voxels; the median cell (1.16 m) grades over 4, with the same shape
   compressed: `[5, 4, 2, 1]`. A decorative constant gradient would look the same
   at both. Quantization would be one voxel at both.

## Mass, honestly

At the **record** tier conservation is exact by construction and asserted for
five magnitudes spanning three orders (`the_weathering_front_conserves_the_
ledger_product_mass`). At the **voxel** tier the eighths are a *draw* — the same
unbiased-estimator doctrine journal/0055 installed — so one column carries
quantization noise: +1.5 % at the argmax, +16 % at the median (whose whole front
is 12 eighths). Over 21 columns sampled across the band distribution the totals
are **56.14 m expressed against 54.12 m owed, +3.7 %**, with the worst single
column off by less than two eighths. The probe prints this population line
precisely so "conserved" is a claim about the estimator and not about one lucky
voxel column.

## Wrong turns worth keeping

**`accessory.is_some()` is not "this is the front."** The first probe run
reported a 103-voxel front grading from 1/8 to 1/8, and a mass error of +850 %.
It had found the **igneous inclusion** — the 96-voxel intrusive basement body
carrying olivine in exactly the same pore slot the front now uses. Reusing a
mechanism means reusing its ambiguity: the front's identity is not "carries a
pore rider" but "carries a **loose** pore rider", and the probe now tests the
rider's class. Worth keeping because the same trap is waiting for every future
consumer that wants to ask "is this weathered?" of a voxel — which is an argument
for the `identify(pos)` arc, and against reading intent off a field's presence.

**The parent flips between `Single` and `Mixed` rows** — `diorite` on one line,
`granite` on the next. That is not this slice: `mixed_at` deliberately uses the
canonical (undithered) member, documented, because re-dithering up to eight
candidates per voxel would cost more than the rest of the fill and could flip a
`classify` tie-break. Before 0099 the basement was one thick `Single` body with a
single contact voxel, so the disagreement had nowhere to show; a front makes ~half
its voxels contacts and puts the two answers side by side. Left as-is (fixing it
moves every mixed voxel in the world) and filed, but it is the visible edge of a
real inconsistency: **two paths answer "which member?" differently for the same
record.**

## What it costs

Gen-time only, and only with the flag on: eight extra events per banded column
and a record `2.67 ×` the product deeper, which is more voxels carrying contents
in the chunks *below* the old band. Nothing on a per-frame or per-tick path, and
the default world (flag off ⇒ no ledger ⇒ no front) is byte-identical — proven by
`generated_world_is_byte_identical_to_the_pre_contract_goldens`, which is the
existing three-world golden and passed unmoved.

## What this owes the ROADMAP

*(Written here rather than applied — the main session holds ROADMAP and was
editing it concurrently, the same hazard 0097 recorded.)*

1. **Sequenced → Shipped: "the weathering front needs a PROFILE, not a slab."**
   Cite the measured profile (19 voxels at the argmax, `[7,5,4,3,2,1,1,1]`
   eighths, +1.5 % mass) and `examples/weathering_profile_probe`.
2. **APPEARANCE WALKS OWED — a flag-ON walk of the *profile*.** The numbers are
   shipped; the picture is the user's. Station A is deep cell (439,276), world
   (84185 m, 9212 m), surface voxel y = 300, front y = 299…281.
3. **Observed — two paths answer "which member?" differently.** `Single` voxels
   use the per-voxel-column dither, `Mixed` voxels the canonical member; a front
   puts them adjacent and the parent rock alternates between two members of the
   same class down a column.
4. **stubs.md #19** — the profile's *shape* (decay length, 7/8 cap, 2.67 ×
   thickness ratio) is a constant measured from nothing; heir is the deep tier
   carrying the front as a depth-resolved term instead of a scalar. Stub #16 is
   **untouched and not retired**: the rind's material *identity* still awaits the
   genesis/emplacement pass.

## Status

Gates green (see the RETURN). **A flag-ON appearance walk is owed** — this is an
appearance change and the walk is the user's to make.
