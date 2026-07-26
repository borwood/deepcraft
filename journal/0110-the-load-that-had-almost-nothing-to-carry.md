# 0110 — The load that had almost nothing to carry

*2026-07-26 · Movement 2b, first slice — material-aware transport*

> blogworthy: **lens 2 (procgen against the backdrop of priors)** and **lens 1
> (AI-native development)**. The interesting thing here is not the sorting rule —
> Hjulström and Stokes are two hundred years old between them. It is that the
> slice was briefed as "the big appearance-changer", built exactly as ratified,
> passed every invariant it was supposed to pass, and then the probe it was
> required to ship measured the effect at **0.000006 % of the archive**. The value
> of the slice turned out to be the *number that explains why*, and that number
> was not available to anyone before the pass existed to produce it.

## What was asked for, and what it means

`material-behavior.md` § 13 has been sitting ratified since 2026-07-24 with a
one-line summary of the whole erosion cycle:

> **weather** (structure→loose) → **entrain** (lift loose into the load) →
> **carry** (down the flow field) → **sort** → **deposit** → the fresh bedrock
> exposed underneath weathers faster.

The engine had all of that except the middle. `erosion.rs` moved a **scalar**: a
cell entrained `min(H, room)` metres of *something*, handed it to its receivers,
and whatever exceeded the next cell's capacity was set down as *something else*.
Mass was conserved perfectly and identity did not exist.

So the rock a recorded unit is made of had to be **inferred from the
environment** at the receiver — `litho_of_tag`, which reads the depositional tag
(subaerial? arid? what energy band?) and returns a lithology. That inference is
the `DepTag → reference_material` shortcut § 13.7 predicted would "half-dissolve",
and the important thing about it is that it is **not wrong**. High energy really
does deposit coarse material. What it cannot do is know about **provenance**: it
structurally cannot say *"this cell records mud not because it is a quiet place
but because the gravel rained out twenty cells upstream and there is none left in
the water."* That sentence is the entire content of a facies gradient, and no
function of the local environment can produce it.

The ratified first slice was therefore: **identity travels, and deposition sorts.**

## The three identities, and the one that nearly became a constant

The brief carried a hard requirement borrowed from `north-star.md`'s content-layer
pass shape — *select materials matching predicate P, apply transform T at rate R* —
and stated as a rule: **a pure pass reads properties and never names a material.**
The engine already violates this in two places (`COAL_ONSET_C`,
`BEDROCK_SEAM_MATERIAL`) and the instruction was not to add a third.

That turned out to be a genuinely useful constraint rather than a hygiene chore,
because the pass needs three identities and two of them fell out of machinery that
already existed:

- **what a flow picks up** is the composition of the cell's own near-surface
  window — and erosion *already* asks that question every epoch, through the
  `outcrop_shares` provider seam, to blend its erodibility table. One walk, two
  consumers, and the entrainment composition can never disagree with the rate
  the same window sets.
- **how the load sorts** is `settle_energy(props)` = `sqrt(grain_mm × SG)`, which
  `dc_core` has carried since S8 and which the *shipped placer pass* already
  thresholds on. Writing a second settling model beside it would have been this
  project's characteristic failure in its purest form.
- **what incision detaches** is where the first draft wrote `Litho::Basement`.

The third is worth recording because A-7's diagnostic is stated as *"when you feel
the need to write a content name into a process, ask what property of that content
you are reaching for — that property is the feature the process lacks."* Applied
here the answer is **"whatever lies below the record"** — and the composition seam
answers exactly that when handed an *empty section*, because a window containing no
recorded units is, by its own deficit rule, entirely the material beneath the pile.
So the pass calls `outcrop_shares(&[])` and the constant disappears. It is also
strictly better than the constant: the day basement stops being uniform, the seam's
heir supplies the answer and the pass is not edited.

## Sorting is the falling ceiling

§ 13.5's instruction is unusually precise and it is the design:

> **Sorting is the falling ceiling; we write the ceiling, not the sort.**

There is no sorted structure anywhere in the pass. The load is seven `f64`s per
cell, one per lithology. Two limits act on it, and § 13.5 insists both are needed:

- **capacity** — how much mass the flow can hold, which the engine already had;
- **competence** — the largest **settling velocity** it can hold, which it did not.

Without competence a stream with spare capacity carries boulders to the sea and
nothing ever fines. With it, a species whose `w_s` exceeds `COMPETENCE_SCALE × cap`
rains out *wherever it is*, and what survives to the next cell is finer by
construction. The capacity limit then draws the excess **coarsest-first**, out of
the heaviest fraction still suspended. Neither step sorts a list; the ceiling falls
and the load is what is left under it.

**Where the one new constant comes from, and why it is not a knob.** The ceiling
needs a scale — settling velocity per unit of transport capacity — and inventing
one would have been a free parameter aimed at the outcome the probe was supposed to
judge. So it is anchored on something the world already ships: `energy_band` calls
a capacity of `0.002` the Low/Medium boundary, and `litho_of_tag` turns exactly
that boundary into the coarse/fine clastic split. `0.002` is therefore *already*
this world's stated "energy at which sand stops moving". Coarse clastic's
`settle_energy` is `0.84`, and `0.84 / 0.002 = 420`. The ceiling crosses the sand
threshold precisely where the shipped facies rule crosses it, and the rest of the
roster arranges itself around that anchor rather than around a fit. It was written
down before the probe was run and not revisited afterwards.

**One ordering decision that had to be made twice.** The competence sweep was first
placed at the *start* of the exchange, on the arriving load. That leaves a hole:
material the cell entrains or incises *after* the sweep leaves the cell even though
the cell cannot carry it, and the clean invariant — *nothing leaves a cell that the
cell could not carry* — is false. Moving the sweep to the **end** closes it, at the
cost of a cell sometimes prising loose a grain size it cannot lift and setting it
straight back down. That is the honest outcome, and it is deliberately *not*
armouring: the grain stays in ordinary loose cover and is tried again next epoch, so
no lag or pavement forms. Selective **entrainment** — the fine-side, cohesion-driven
half of Hjulström's curve, which is what actually armours a bed — is § 13.4 and is
deferred.

## Mass, per species — and why the obvious generalisation leaks

journal/0109 had just solved this problem for a scalar and the brief handed the
solution over with an instruction attached: *verify the handoff rather than trusting
it.* The scalar rule is that a multi-receiver split conserves mass only if the
shares sum to the whole **with no residue**, so the last weighted direction takes
`q − Σ(earlier shares)` instead of `w·q`.

The obvious generalisation to a multiset is: split the **total** exactly by that
rule, then hand each species its fraction of the total. It is wrong, and it is wrong
in the silent direction. Each species picks up its own rounding against a shared
denominator, so the **total** stays exact while every individual species drifts —
a leak that is invisible in the quantity everybody checks, per-species, compounding
down a thousand-hop chain, and unattributable afterwards because `qs_sp[j][s]` is a
sum over contributors with no unique factorisation.

So each species runs **its own budget**: for species `s`, the last weighted
direction takes `q_s − Σ(earlier shares of s)`. And the scalar the flux record
stores for a face is then the **sum of the species shares on that face** — not a
second split of the total, because two derivations of one quantity is exactly the
drift `flow.md` § 3 exists to prevent. On the sorted path the species vector *is*
the authority and every scalar the pass reads is re-summed from it in fixed index
order rather than decremented, so the two cannot drift apart at all.

That is asserted three ways rather than argued:

- `a_shared_total_leaks_per_species_and_an_own_budget_does_not` — the hazard,
  constructed. It runs both rules on the same weights and species quantities and
  asserts the shared-total rule loses mass on at least one species while the
  per-species rule closes **exactly**. It is a statement about arithmetic, so it
  needs no simulation.
- `no_species_leaks_at_its_own_junction` — the *local* half, on a whole world. The
  pass carries a running maximum of `|Σ_faces share(s) − q_s| / q_s` over every
  `(cell, species, epoch)` of the run; it must stay under `1e-12`. This is
  journal/0109's `the_partition_leaves_no_residue`, per species. (A running max
  rather than a stored plane because a leak anywhere is a leak, and the per-species
  face plane would be `n × 8 × 7` — 133 MB to assert a scalar.)
- `mass_is_conserved_with_material_transport_on` — the *global* half, the
  whole-world ledger. A share written into the record but never added to a
  neighbour passes the residue test and fails this one.

`exchange_cell` is still **one** function serving both the D8 and the MFD chain,
which was the other half of the handoff and is not negotiable: two copies of a mass
budget is the same drift wearing a different hat.

## The record grew an axis and did not grow a byte

`DepUnit` gained `species: Litho` — the material that arrived. Every other axis of
a unit is a measurement of the *environment*; this one is a measurement of the
*load*.

Two things kept the blast radius sane. First, `deposit()` keeps its signature and
fills the species with `litho_of_tag(tag)`; only the fluvial transport pass calls the
new `deposit_as`. Every other depositor — the wind agent, the wave agent, the biotic
layer, pedogenic overprint, coalification — honestly carries no load and honestly
gets the tag-derived answer. (The **eolian** family genuinely does have a load and
would travel its own identity; it is deferred with the rest of § 13.2's wind/ice/
gravity family.) Second, the species is a **pure function of the tag** when the flag
is off, so it adds nothing to the merge key and the record is byte-identical.

It fits `DepUnit`'s existing padding, so `size_of` is unchanged — asserted against a
mirror of the pre-slice layout rather than against the number `24`, so the test says
*"the axis is free"* rather than pinning a snapshot the next field would have to be
talked out of.

Downstream, every consumer that used to ask `litho_of_tag(u.tag)` now asks
`u.species`: the outcrop window walk, the head field's permeability stack, the deep
cell inventory's base composition, and the collapse tier's class routing. With the
flag off each of those reads the identical value.

**And the golden fingerprint learned to see it.** `record_fingerprint` hashes the
species byte now. That means every record golden in the tree was re-derived in this
commit *even where the record did not move* — which is why `GOLDEN_RECORD_SCALAR_LOAD`
exists: it is the pre-2b record under the post-2b hash, so the difference between it
and the old constant is purely the new axis, and the difference between it and the
shipped constant is purely the physics. A fingerprint blind to the axis would have
let the slice move every rock in the world without moving a hash.

## The number the slice is accepted on, and it is a NULL

The brief was explicit that this was a live risk and that measuring it was an
obligation rather than a courtesy: MFD shipped with a uniform convergence exponent
and journal/0109 measured peak catchment falling **1,245 → 84 cells**. So
`examples/facies_probe.rs` runs the production world (seed 1337, `Extent::Medium`)
both ways and reports the fining gradient as a distribution.

**It does not express.** On the shipped world:

| | scalar load | material-aware |
|---|---|---|
| recorded mass whose material disagrees with its environment | 0.0000 m | **0.0250 m of 440,595 m (0.000006 %)** |
| mean grain size, Low band | 0.0040 mm | 0.0040 mm |
| mean grain size, High band | 0.3000 mm | 0.3000 mm |
| headwater/trunk grain ratio | 0.921 | 0.921 |

The gradient is unchanged to four decimal places on every axis. That is a null, and
the useful part of this entry is that the probe was built to say **why**, in three
numbers that were not available to anybody before the pass existed:

### 1. Fluvial transport is 0.109 % of this world's sediment routing

Over the full 200-epoch run, on 297,025 cells:

| where the sediment goes | metres |
|---|---|
| picked up by the flow (entrained + incised) | **659.5** |
| weathered to regolith **in place**, never travels | **256,886** |
| moved by **hillslope creep** | **605,117** |

The rivers touch **one nine-hundred-and-eighteenth** of what creep moves. The
archive is 440,595 m thick and the fluvial pass contributed 0.15 % of it. Sorting a
load that carries a thousandth of the sediment cannot change what a cliff face looks
like *however correct the sorting is* — and this is true of the scalar solve too, so
it is a fact about the landscape and not about the slice.

That is worth stating plainly: **this world's sediment is routed by hillslope
diffusion, which is § 13.2's gravity/mass-wasting family — the branch of the
transport family this slice deliberately did not make material-aware.** We made the
smaller agent honest.

### 2. No cell on this world can carry sand

The largest competence ceiling anywhere on the final epoch is **0.2832**; coarse
clastic's settling threshold is **0.840**. Median ceiling is 0.107, just above mud's
0.098 — 55.5 % of land cells can hold mud, 0.0 % can hold sand, gravel or anything
above it. 92.25 % of everything the flow picks up is put straight back down by the
ceiling in the cell it came from.

The reason this is a finding about the world rather than about my constant is that
**two independent instruments agree**. The ceiling is anchored on the shipped
`energy_band` Low/Medium boundary, a capacity of `0.002`. The maximum capacity this
world reaches anywhere is **6.74 × 10⁻⁴** — *three times below it*. The shipped
facies rule's own calibration is out of range for the world it ships with; the
competence ceiling merely inherited that and made it visible.

### 3. And downstream of both, hybrid-`p`

journal/0109's uniform convergence exponent disperses flow at every cell, so
catchment never concentrates and no reach ever accumulates the discharge a channel
needs. That is the named blocker, it is the FLOW arc's owed item, and it is **third
on this list rather than first** — because even with the channels back, a transport
term that moves 0.1 % of the sediment has no purchase on the archive.

**Nothing was tuned.** The obvious move on seeing a flat gradient is to lower
`COMPETENCE_SCALE` until sand starts travelling, and it would have worked, and the
resulting number would have been a constant chosen to manufacture an outcome —
a number pretending to be a mechanism. The probe prints the anchor, prints the
world's actual maximum, and prints the refusal.

## What a player sees

**Nothing.** Mean, minimum and maximum surface elevation are identical to two
decimal places between the two solves; the whole-grid mean absolute change is below
the reporting precision. The goldens moved — the surface fingerprint, the record
fingerprint, and both medium-world content goldens — because a 2.5 cm redistribution
is enough to flip a surface voxel here and there, and the Small world moved on its
**block** hash only with `materials` and `table` byte-identical, which is the same
terrain-not-archive signature MFD produced.

This is an **appearance change on paper and a null in the viewport**, for the second
slice running. It should be announced and it should **not** consume a walk: a
tour-map would find nothing to stand in front of, and journal/0109 already paid for
that lesson once.

## What it cost

Deep run **24.1 s → 28.8 s (+4.7 s, +19 %)** on the production world; gen time is not
a constraint and nothing here is on a runtime path. Residency **+0.04 MiB
absolute** (169.00 → 169.05 MiB) — the record grew 2,895 units out of 5.54 M because
the species joined the merge key, and `DepUnit` itself did not grow. The load,
composition and deposition planes are `n × 7` f64 **gen-time scratch** (~50 MB at
production width) and are not resident in the world.

## The thing this entry is really about

The slice was briefed as *"the big appearance-changer"* and it changed nothing
visible. Both halves of that are correct, and the gap between them is the finding.

The mechanism is right: identity travels, the ceiling falls, mass closes per species,
the pass names no material, and the record now carries what arrived instead of what
its surroundings implied. Every one of those is a permanent improvement to the shape
and all of them will still be true the day the channels exist.

What was wrong was an unexamined assumption — held by the design docs, by the brief,
and by me until the probe returned — that **the fluvial transport pass is what moves
this world's sediment.** It moves 0.109 % of it. Nobody had measured that, because
until this slice there was no instrument that could: a scalar load has no ledger to
itemise, and "how much did the rivers move compared to creep" was not a question the
engine could be asked.

That is the same shape as journal/0109's two errors, one entry apart: a claim about
**order** (routing is upstream of erosion, so the world will move) smuggled in as a
claim about **substance**. Here it was a claim about **naming** — this is *the
transport pass*, so it must be what does the transporting — smuggled in as a claim
about **magnitude**. Three times in two days, in three different disguises. The
defence is the same each time and it is cheap: before believing a mechanism matters,
measure how much authority it has over the thing you are claiming it changes.
