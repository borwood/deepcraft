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

## The bug that hid inside a golden diff

The first full gate came back with **twelve failing suites** — the surface planes, the
strata record, the far field, the geotherm, the head plane, the flux record, the voxel
contents, and every *old-solve-still-reachable* golden beside them. That is a lot of world
to move with a slice whose whole claim is *"identity, not mass"*, and the honest reading of
the doctrine (goldens are tripwires; a ratified-semantics move re-captures with the why)
would have let it through. The why was even ready to write: the record is finer, so the
world is different.

**It was wrong, and reading the diff as authorized would have shipped it.**

Identity joined `deposit_as`'s merge key — deliberately, so a sand sheet and the mud that
followed it are two units and not one. So a bed that used to be a single `ClasticFine` unit
is now two when its members differ. `window_walk` — the near-surface window whose per-class
shares set the erosion rate — buckets both back into `ClasticFine` and gets the **same
quantity**. But it gets there by adding `4.1 + 3.2` instead of `7.3`, and IEEE addition is
not associative. One ulp in the shares, into the susceptibility blend, into the incision
rate, compounded over two hundred epochs: a different continent.

**The erosion rule had not changed at all.** Nothing about how a rock resists anything was
touched — `susceptibility_table` is still keyed by class and still built from the class's
reference material, so a siltstone bed still erodes at mudstone's rate (that is slice 2's
job). The world moved because the *bookkeeping* got finer, and a class-grade rate has no
business being able to tell.

So the walk was rewritten to sum **runs of the pre-P11 merge key** `(class, tag, chapter)`
rather than units — a class-grade quantity should be a function of the class-grade record,
invariant to how finely identity subdivides a bed. Eight lines, dying in slice 2, pinned by
a test asserting bit-equality.

**And it did not work, which is the more useful half of the story.**

The next full run came back with the geotherm moved *again* — to a third value, neither the
old one nor the previous new one. The premise was wrong in a way that was easy to miss:
grouping was never the only difference. The recorder builds a unit's thickness by
`top.thickness_m += d` epoch after epoch, and `erode` subtracts off the top unit by unit.
Split the bed and you have not merely regrouped the addends — **you have different
addends**, each carrying its own rounding history. Coalescing them afterwards sums numbers
that were never the same numbers.

So bit-restoration is not available at all while identity sits in the merge key, and the
coalescing was **removed**: keeping a mechanism after the measurement has falsified its
premise is the A-2 shape, and writing it into the same slice that celebrates fixing an A-2
would have been a poor joke. What survives is the bound — the shares agree to well inside
1e-12, so the erosion *rule* is unchanged and only its rounding is — and the honest
statement in its place.

Two lessons, and the second is the one worth keeping. First: **a moved golden is a
hypothesis, and the first plausible mechanism is not evidence.** The plausible story (finer
record, finer world) was true in every clause and wrong in its conclusion. Second: **the fix
for a hypothesis you cannot confirm is not a smaller fix — it is a measurement.** The eight
lines looked cheap enough to keep "just in case"; the run that would have proved them is the
same run that deleted them.

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

### The numbers (seed 1337, `Extent::Medium`, `examples/member_diversity_probe.rs`)

Deep run 38.75 s · **10,951,030 recorded units** · 404,898.8 m recorded · `DeepField`
256.91 MiB.

| class | registered members | distinct materials recorded | units | share by recorded metres |
|---|---:|---:|---:|---|
| fine | 2 | **2** | 1,194,668 | `dc:mudstone` 55.3 % · `dc:siltstone` 44.7 % |
| coarse | 2 | **2** | 8,506,445 | `dc:sandstone` 62.5 % · `dc:conglomerate` 37.5 % |
| soil | 1 | 1 | 1,224,612 | `dc:carbonaceous-mudstone` 100 % |
| peat | 1 | 1 | 22,164 | `dc:peat` 100 % |
| coal | 1 | — | **0** | nothing recorded |
| charcoal | 1 | 1 | 3,141 | `dc:charcoal` 100 % |

Before this slice every one of those rows read **one** material — the class's reference — and
the other member existed only as something expression could roll for, per chunk, from the
wrong century's weather. The two rows that *can* carry the claim both do, and they do not do
it by a hair: siltstone is 44.7 % of the fine record and conglomerate 37.5 % of the coarse.
Those are close to the abundance ratios the content set declares (1.0 : 0.7 and 1.0 : 0.6),
which is the right shape — at `depth_m = 0` every clastic window is satisfied, so abundance
carries most of the weight and climate tilts it.

The four one-member rows read `1`, which is **correct and not a null**: a class with one
member cannot diversify. The coal row reads **zero units recorded** — also not a defect, and
already known: the 2026-07-25 tour map found **no coal on the shipped world** at all
(corrections #51). Peat is recorded but never buried deep enough to rank up here.

### And the number nobody had priced

**The merge-key split factor is 2.4053×** — 10,951,030 units where the pre-P11 class-only
key would have produced 4,552,847. At `DepUnit`'s 16 bytes that is **167.10 MiB against
69.47 MiB: +97.6 MiB of resident record.**

This is the design audit's I4, the one it filed as *"unpriced here"*, and it is the sharpest
correction to the audit's own headline. Its § 3.1 was right that the identity **field** is
free — `MaterialId` and `Litho` are both one byte, `size_of::<DepUnit>()` is still 16. But
free per unit is not free per world when identity joins the merge key and the unit count
2.4×s. **The zero-byte swap costs 97.6 MiB**, and `DeepField` is resident at runtime, where
residency is first-class doctrine.

The cause is worth stating precisely, because it is a **design fork and not an overhead**.
The member draw is addressed `[depositor, cell, epoch, k]` — a fresh roll every epoch. So a
cell sitting in a stable environment for two hundred epochs does not record one thick bed of
whichever member fitness favours; it records an *alternating* stack, mudstone/siltstone,
flipping on a coin the sim tosses again every epoch. That is where the 2.4× comes from.

Is that right? It is the literal reading of *"fitness at deposition, under the context of its
own geological day"*, and it is defensible: each epoch is its own depositional event. But the
alternative is at least as defensible and much cheaper — address the draw by `(cell,
chapter)` instead, so the member is **persistent while conditions are**, and changes at a
real time surface rather than on a per-epoch coin. Fitness would still vary continuously
with climate; only the tie-break within the distribution would stop being white noise in
time.

The corpus has already learned this lesson one axis over: journal/0073 moved the class dither
from white noise to a *coherent* field because white noise aliased in **space**. This is the
same shape in **time**, and it manufactures laminae no process made. It is left as it was
built, flagged rather than decided — a per-epoch roll is what the slice was specified to do,
and the honest contribution here is the measured price of that choice.

## The cost the doctrine says is free, and the gate that says otherwise

*"Gen time is not a constraint"* is standing doctrine, and it is not the same statement as
*"gen time has no gate"*. `s7_measurements::pregen_time_vs_extent` asserts a Medium pregen
stays under **60 s** — a ratified number (S13's own cost note calls it that when it argued a
finer deep cell would break it). The first full run after the conversion came back at
**81.8 s**.

Two causes, both mine, both the same mistake in different clothes — **a per-world decision
left inside a per-event loop**:

1. `GeologySet::select` found its class in a `BTreeMap<String, _>` and collected its member
   weights into a `Vec`. Free at one call per (chunk, event). Not free at one call per
   *deposition event per cell per epoch*.
2. The identity was computed as a **call argument** to `record_cell`, so every cell paid for
   it every epoch — including the ones that eroded, and the ones that did nothing.

The fixes are unglamorous and total: the identity is evaluated lazily inside the
`dh > 0` branch, `select_in` takes an already-resolved `&GeoClass` so a run resolves its six
deep classes once, and the inverse-CDF recomputes its weights on the second walk instead of
storing them (`fitness` is pure, so the answer is bit-identical and the malloc is gone —
which the goldens then confirmed, since they did not move across that change). The gate went
green.

Worth naming because the trap is structural rather than clever: **moving a decision earlier
in time moves it into a hotter loop**, and every "run it at deposition instead of at
expression" slice will meet the same wall. `select` was written for a caller that ran it
thousands of times; P11 hands it tens of millions.

### What it actually costs, measured against `main` rather than guessed

Three uncontended samples each side, same machine, same session, `main` at `957edfc`:

| | `main` | P11 slice 1 | Δ |
|---|---:|---:|---:|
| Medium pregen | 42.82 / 42.63 / 42.93 s | 48.31 / 47.05 / 47.09 s | **+4.69 s (+11.0 %)** |
| Medium `approx_resident_bytes` | 357,169,261 | 472,970,797 | **+110.5 MiB (+32.4 %)** |
| Small pregen | 2.36 s | 2.72 s | +0.35 s (+14.8 %) |

The residency number is the independent corroboration of the split factor: the probe's
per-unit arithmetic said +97.6 MiB of record, and the whole `Pregen` grew +110.5 MiB. Same
story, measured two ways.

**And the gate is RED on this, honestly.** `pregen_time_vs_extent` asserts Medium under
60 s. Uncontended we are at 47.5 s and pass; run as part of the full workspace gate — where
this binary's *other* test builds its own Medium world concurrently — it came in at
**62.7 s** and failed. `main` uncontended is 42.8 s, so under that same contention `main` sits
near 56 s: **the budget was already ~95 % spent, and this slice is what tips it over.** That
is not a reason to move the line. The 60 s is a ratified number, and moving a gate to admit
one's own work is the shape the corpus has the most receipts for. Reported red, with the
attribution, for the ruling it needs — and note the cheapest fix on the table is the *same*
`(cell, chapter)` draw fork that fixes the residency, because both costs have one cause.

## Goldens

Identity is part of the deposited record and part of `deposit_as`'s merge key, so the
world's bytes moved. Under the scratch-pad doctrine that owes no ratification loop and no
byte-identicality — it owes the **why**, per golden. The full mechanism write-up lives at
`tests/providers_common/mod.rs` § P11, beside the constants, where the next person to read a
moved hash will actually be standing. Three families, and they are not the same event:

1. **The RECORD halves** — the record names the rock, and identity joins the merge key so
   beds split where their members differ. This is the slice working. It is also the
   second-order residency cost the design audit left unpriced (its I4): more units, at 16 B
   each.
2. **The CONTENTS goldens** — the most legible move in the tree: voxels that could only ever
   hold `dc:mudstone` and `dc:sandstone` now hold `dc:siltstone` and `dc:conglomerate`.
   *This is the change a walk will see.*
3. **The SURFACE halves and everything downstream of them** (geotherm, flux, far field) —
   **rounding, not rule**, per the section above. Worth stating plainly in the diff, because
   the surface hashes are the ones that look like "the world was re-tuned" and were not.

One move in the third family is not rounding and deserves its own line: **`GOLDEN_HEAD`**.
`head::permeability_of` took a `Litho` and looked up its class's reference material; it now
takes the recorded `MaterialId`. So a siltstone aquitard (k = 0.08) stops being a mudstone
one (k = 0.02), and the head field becomes the **first consumer in the tree to read member
grade as physics rather than as albedo**. It cost nothing — that function was already
property-sheet derived rather than class-keyed, which the design audit had spotted and filed
under *"sites that are already registry-shaped and survive any option"*. The audit was right,
and the payoff arrived a slice earlier than expected.
