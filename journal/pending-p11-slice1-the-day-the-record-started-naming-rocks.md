# P11 slice 1 — the day the deep record started naming rocks

> blogworthy: **procgen against the backdrop of priors** — the difference between a
> simulation that *records* what happened and one that *reconstructs* it afterwards, and
> why the second one cannot be fixed by making the reconstruction cleverer. Also
> **reflexions in a deepsim codebase**: the whole slice is one question — *where does a
> decision belong?* — and the answer moved a byte, not a system.

## The shape of the defect

Deep time deposited a bed. The bed was a *class*: one of seven `Litho` variants — fine
clastic, coarse clastic, soil, peat, coal, charcoal, basement. That is what
`DepUnit::species` held, one byte, and that is all the record ever knew.

Hundreds of millions of simulated years later, when a player walked up and a chunk had to
be generated, the collapse tier asked the question the record could not answer: *which rock
is this?* It answered it by running `GeologySet::select` — fitness × normalised abundance ×
an addressed draw — against a `FormationContext` built from the **present-day** climate,
sampled once at the **chunk centre**, 28.8 m wide.

So a marine mud laid under a warm wet Cambrian sea and a marine mud laid under a cold dry
Permian one were the same byte in the archive, and both were adjudicated by whatever the
weather is doing at that chunk's middle *today*. The member — mudstone or siltstone,
sandstone or conglomerate — was **invented at expression**.

Three separate threads had already walked into this wall in one week, which is what got the
arc ratified:

1. transition edges live on **material definitions** (DECIDED 2026-07-22), and a class-grade
   sim structurally cannot consult them — every class-keyed constant becomes an S-3 parallel
   rule the day edges land;
2. the member census (journal/0129 § 5) found **4 of 10 classes cap the member dither at a
   coin flip** — expression cannot diversify what history never distinguished;
3. the fluvial record-terms slice had to scope its composition term *provenance-not-size*
   partly because the roster is class-grade.

## The one arithmetic that decided the design

`MaterialId` is a `#[repr(u8)]` fieldless enum with 26 contiguous variants. `Litho` is also
one byte. **The swap is free.**

That is not a detail; it is the whole reason the ruling could be *"the record names the
rock"* rather than *"the record names the member"*. A `GeoMemberIdx` is a `u16`, and
`DepUnit` is 16 bytes with **zero padding left** — so a `u16` identity costs a whole 8-byte
alignment slot, ×5.5 M live units ≈ **42 MiB**. The design audit computed that by hand from
the field list and flagged that no `offset_of!` had ever been run. It has now:

```rust
const _: () = {
    assert!(size_of::<DepUnit>() == 16, "DepUnit must stay 16 bytes");
    assert!(size_of::<MaterialId>() == 1, "the identity byte stays a byte");
    assert!(size_of::<DepTag>() + size_of::<f64>() + size_of::<bool>()
            + size_of::<u8>() + size_of::<MaterialId>() == 16,
        "DepUnit has no padding left — a wider field costs a whole 8-byte slot");
    assert!(std::mem::offset_of!(DepUnit, species) + size_of::<MaterialId>() <= 16);
};
```

The claim is now a compiler fact, and the next person who wants to append a field to
`DepUnit` will find out at build time rather than in a residency probe.

## What actually had to move

The naive reading of the ruling is *"change the field's type"*. The real work was that the
deep-time simulator **had never seen the content registry**. `Litho` appears 308 times in
`crates/`, all of them inside `dc-worldgen`; `GeologySet` appeared nowhere in
`deeptime/`. The deep tier was content-blind by construction, and expression-time fitness
was the *consequence* of that, not the cause.

So the slice's spine is: thread the `GeologySet` into the deep-time runner, and give every
site that writes an identity the three things fitness needs — the content, the formation
conditions of that geological day, and an addressed draw.

```rust
pub struct MemberCtx<'a> { geology: &'a GeologySet, draws: Draws, epoch: u64 }
pub struct DepositCtx<'a> { geology: …, form: FormationContext, draws: Draws, cell: u64, epoch: u64 }
```

`DeepStepCtx` gained one field, `geology`, and a pass reaches it through the ctx it is
handed — never a global, like every other capability on that runner.

### The four writers, and why they are not the same event

`DepUnit::species` has exactly four writers, and converting them was where the design got
interesting, because **they do not all happen at the surface**.

| writer | what sets the identity | formation context |
|---|---|---|
| the erosion recorder (`deposit_as`) | the argmax of everything a mover brought | surface: this cell, this epoch, `depth_m = 0` |
| wind / wave / the biotic layer | the tag's own class (they carry no load) | surface, same |
| pedogenic overprint | the soil the community built out of what was lying here | surface, same |
| **burial diagenesis** (`promote_coal`) | peat becoming coal | **the slab's real burial depth and the geotherm temperature at it** |

The last row is the one worth stopping on. `CLASS_ORGANIC_COAL`'s contract has said since
journal/0026 that *"the class's depth axis is the rank axis — a pack that wants
lignite/bituminous/anthracite members discriminates them on burial depth"*. That was a
promise no code could keep: the depth in the fitness context was the *expression tier's*
notion of burial, reconstructed at chunk resolution. `promote_coal` already computes the
real thing — it walks the column top-down accumulating overburden and integrates the
geotherm across it — and it was throwing that number away to write
`species = litho_of_tag(tag)`. It now re-runs fitness under exactly that P/T. Vanilla ships
one coal member so nothing varies today; the day a pack registers lignite and anthracite
they separate with no further work, on the physically right axis.

That is the shape of the whole slice in miniature: **the sim already knew; it just was not
writing it down.**

### The draw address, and a domain that had to be its own

Fitness is an inverse-CDF over member weights and it needs a uniform. The deep tier had no
stream for this. The obvious shortcut was to reuse `GeoDeep` — the salt the collapse tier
already uses to select a member for a recorded unit — and it is exactly wrong: those are two
decisions at two tiers on two different grids, and sharing a stream would make a bed's
*recorded* identity a deterministic function of the draw the *expression* tier later makes
over it. That is the `GeoPore` defect (journal/0105) with the tiers swapped. So:

```
DeepMember = 0x5900_0002;   // address: [depositor, cell, epoch, k]
```

The `depositor` tag exists for the same reason one level down: if the wave agent and the
wind agent read one stream at one cell in one epoch, their member picks are the same number
and two beds correlate for no physical reason.

The address is **the cell index**, never an iteration counter — which is what keeps the
parallel record phase byte-identical to the scalar one. A test asserts the two agree cell by
cell.

## The thing that stopped happening

```rust
if !event.dither {
    return event.member;
}
```

`dithered_member` used to re-run `select` per voxel column for *every* strata event. For
deep-time depositional history it now hands back the recorded member and does nothing else.
`StrataEvent` gained a `dither` flag rather than a heuristic, because the two kinds of event
in that vector are genuinely different: the **year-zero veneer** (clastic veneer, igneous
bodies, placer, weathering front) selects at expression *because that is where its formation
context lives* — the veneer really is forming now, under this climate, and geology.md's
formation-context ruling calls that correct. Deep history does not.

Worth being precise about what this does **not** fix, because the two are easy to conflate.
The dither had two jobs braided together: *re-adjudicating which member fills this class*
(invention — now dead for the record) and *varying identity across a deep cell's footprint*
(a spatial membership dither — which never existed; the re-selection was standing in for
it). So a deep cell's bed is now **one material across its whole ~460 m footprint**. That
tile is real, it is visible, and it is slice 3's — the near-path restructure, whose
cell-membership dither interpolates between *recorded* neighbours instead of re-rolling.
The slice trades a wrong answer that varied for a right answer that does not vary yet.

## The scaffolding, named out loud

Ruling 2 is **A-CLEAN: no class view survives in storage or physics**. This slice ships one
anyway, and it is worth saying why rather than hoping nobody greps for it.

The transport arithmetic is still `Litho::COUNT`-wide: four `n × SPECIES` budget planes, the
susceptibility and settling tables, `WindowShares = ShareVec<7>`. A `MaterialId`-grade record
has to be bucketed back down before those tables can index it. So `Litho::of_material` exists,
`window_walk` calls it, and the far field's `deep_class_slot` calls it.

It is a **summary beside an authority**, and the doctrine's price is paid rather than dodged:
it is the exact inverse of the member→class edge the `GeologySet` already declares, and a
test asserts the two agree for every registered member. It has two named heirs, both already
sequenced — slice 2 (the budgets go CSR-sparse over `MaterialId`, so the bucket has no
consumer on the transport side) and slice 4 (the roster dissolves). Nothing new may be built
on it.

## The comment whose reason expired

`lithology.rs` said, for months and in the module docs where everyone reads it:

> The reference member is *fixed per class* rather than sampled from the live registry,
> which is deliberate: it means **adding an organism or material pack can never move
> terrain**.

That was true and this slice makes it false. Deposition-time fitness reads the live
registry; the deposited identity feeds `exposed_shares` → `susceptibility_table` → the
erosion rates; a pack that registers a new clastic member therefore *can* change how fast a
hillside wears down.

The user caught this before it was ruled on, and the disposal was not "overrule it" but
**dissolve it**: a different pack set **is a different world** (the 2026-07-19 world-identity
rule), so the hazard the guard was protecting against — a pack retroactively changing an
existing world — is answered by *storing* the identity rather than re-deriving it, which is
what this slice does. The question was vacuous, not wrong (corrections #84). What remained
was owed prose, and rewriting it was part of the slice rather than a follow-up, because a
comment asserting an expired guarantee in a module every erosion reader loads is an A-2 that
costs an architecture the day someone believes it.

## The gap this slice opened

Making the deep sim content-aware means the content set is now an **input to pregen**, and
`Pregen::run(WorldParams)` does not carry one. The door exists
(`run_cells_with_geology` / `build_field_cfg_cadence_geology`) and defaults to `vanilla()`
exactly as `WorldGenerator::new` does — but nothing upstream passes through it, so a world
built with a custom set has a **vanilla-laid record read by a custom expression set**.

Zero blast radius today (production is vanilla end to end) and real the day a pack exists.
Filed as a stubs.md entry (ordinal assigned at merge) with its heir named: E7's per-world manifest — which is the same field
corrections #84 already noted is *"written as a rule and not as a field"*. The slice did not
close it; it made it visible from a second direction, which is how a seam usually gets built.

## Reading the acceptance honestly

The measurement is a distribution, not a threshold: **for each class the deep tier can
deposit, how many distinct materials does the shipped record hold, and in what proportion?**

The nulls have to be stated first or the table lies. Vanilla registers **14 members across
10 classes**, and only **4 classes have more than one member** — of which **two** are
classes the deep record can deposit (clastic-fine: mudstone + siltstone; clastic-coarse:
sandstone + conglomerate). The other two multi-member classes are igneous, which deep
history never carries. The four organic classes ship one member each.

**A one-member class cannot diversify, and a `1` in its row is the correct answer, not a
failure.** Reading it as a null would be reading the content set's shape as a defect in the
mechanism. Two rows carry the claim; the probe says so in those words, and a second gate
test pins that the one-member rows hold *that* member rather than the reference table's
answer by coincidence.

<!-- MEASUREMENTS: filled from examples/member_diversity_probe.rs on seed 1337, Extent::Medium -->

## Goldens

Identity is part of the deposited record and part of `deposit_as`'s merge key, so the
world's bytes moved. Under the scratch-pad doctrine that owes no ratification loop and no
byte-identicality — it owes the **why**, per golden. Two mechanisms, and they are different:

1. **the identity itself** — a unit that recorded `ClasticFine` now records `dc:mudstone` or
   `dc:siltstone`, chosen by fitness at deposition;
2. **the merge-key split** — two adjacent beds that coalesced as one `ClasticFine` unit are
   two units when one is mudstone and the other siltstone. The unit count rises, which is
   the record getting more honest and the second-order residency cost the design audit left
   unpriced (its I4).
