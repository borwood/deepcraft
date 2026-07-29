# 0126 — A hashed length is not a hashed artifact

*2026-07-29. The tripwire sweep journal/0124 filed and did not run — sixteen `DeepField`
members plus the far field, four new goldens, and one candidate list that was wrong in both
directions.*

> blogworthy: **lens 1 (AI-native development)** and **lens 3 (reflexions in a deepsim
> codebase)**. The interesting part is not that four artifacts were unguarded. It is *how
> the list of suspects was built* — from an index of what nothing **reads** — and that the
> index was a bad proxy in both directions at once. Then the twin arrives: a second agent
> finds the same hole from the far side of the collapse tier, by way of an acceptance
> criterion that could not fire, and the completeness check I had just shipped could not
> have caught it.

## The thing that was owed

journal/0124 ended on a sentence it could not act on:

> An artifact the ritual ships with no tripwire on it cannot have an *authorized* move,
> because nobody can see it move.

It had found this the hard way. The `Schedule` slice moved the flow record, and a full
834-test gate watched it happen and said nothing — the record is a pure sidecar to both
existing goldens, so `GOLDEN_SURFACE` (terrain) and `GOLDEN_RECORD` (strata) are both
*structurally* incapable of seeing it. Measuring the move meant writing a throwaway harness,
running it, stashing the slice, running it again on `main`, and diffing by hand.
`GOLDEN_FLUX` closed that hole. The ROADMAP entry it filed asked the general question:
**which others?**

The answer had to come from the struct, not from memory. `DeepField` has sixteen members.

## Twelve were already covered, and two of them were on the suspect list

The classification is dull where it should be dull. `surf` and `regolith` are
`GOLDEN_SURFACE`; `strata` is `GOLDEN_RECORD`; `w`/`wp`/`cell_m` are hashed as the shape
header; `recv`/`area`/`lake` are the drainage export, inside `surface_fingerprint` too.
`flux` gained its golden yesterday.

**And `exhum`/`t_crust` — named in the brief as obvious candidates — turned out to have been
hashed the whole time**, by the same `surface_fingerprint`, since before the flow record
existed.

That is worth more than the two lines it costs to fix, because of *why* they were suspected.
The candidate list was assembled from `spines.md` § 3, *"Built, and nothing calls it"* — the
index of exported machinery with no production consumer. `exhum`/`t_crust`, `geotherm`,
`chapters`, `head` and `flux` all have rows there. The inference was: **exported and unread,
therefore probably also unhashed.**

It is a *reasonable* inference and it is wrong, because **idle and unguarded are independent
axes**. What decides whether a golden can see a plane move is whether some fingerprint
function reads it. What decides whether it is in § 3 is whether production reads it. Those
are different readers. `exhum` and `t_crust` have no production consumer and a perfectly good
tripwire; the two facts have nothing to do with each other. The § 3 header now says so, in
the same paragraph that adds the tripwire annotations, so the next author briefed from that
index is not briefed into the same error.

## Three were genuinely unguarded, and one of them is embarrassing

`geotherm` and `head` are the two condition-fields, and both carry the same sentence in their
own doc comments:

> **Not part of the surface fingerprint** — a new field, covered by its own tests, so the
> pre-existing planes' goldens are unaffected.

That sentence is true, was honest when written, and is exactly the shape of a hole. "Covered
by its own tests" means covered by *invariant* tests — the geotherm is populated and varies;
an unconfined water table never stands above its own ground — plus two **flag-independence**
guards (`flux_record.rs` and `head_field.rs` assert `geotherm` is unmoved by *their* flag).
Not one of those is a fixed point across commits. A pass could have changed the geothermal
gradient of the entire world, and every one of them would still have passed.

`head` is the sharper case, because journal/0124 *moved its epoch-0 solve* — deleted the
bare-surface pre-loop seed, gave it the real drainage-anchored solve — and reported the
exported plane bit-identical. That claim was true. It was also obtained by hand, from a
harness written for the occasion, because nothing in the tree could answer it.

## And `chapters`, which is the one worth the entry's title

`DeepField::chapters` is the plate table: one `Vec<Plate>` per tectonic chapter, each plate a
position, a velocity and a continental flag. `spines.md` § 3 already calls its row *"the
sweep's own indictment"* — the field's doc comment has said *"today the table is exported and
read by nothing"* for longer than the index has existed, and three audits added rows for its
two immediate neighbours in the same struct without noticing it.

It nearly escaped a second time, for a different reason. `surface_fingerprint` **does** touch
it:

```rust
h.usize(f.chapters.len());
```

which reads, in a table of sixteen members, as coverage. It is not. The chapter count is a
config constant (`DeepConfig::chapters`). That byte pins the *config*, and says nothing
whatever about where the plates are, how fast they move, or which of them are continental.
The table could be rebuilt from a different seed salt, advected at a different rate, or have
every plate flipped oceanic, and the length would not twitch.

> **A hashed length is not a hashed artifact.**

Which is the same defect as `has_contents` answering per-CHUNK (corrections #49), and the
same defect as a caption a probe prints beside a number it no longer describes: **a real
measurement, about the wrong thing.** Those are much harder to see than an absent one,
because absence looks like absence and a wrong-target number looks like diligence.

So the falsifier is a test rather than a paragraph.
`the_chapter_length_byte_is_blind_to_what_the_chapter_table_says` clones the table, nudges
one plate one kilometre east, asserts the lengths are unchanged at both levels — and asserts
the fingerprint moved. Without it, *"the new constant adds coverage"* is a claim; with it, it
is a demonstration.

## One constant per artifact, and why not one blob

The obvious cheap shape is a single hash over all three planes. It was rejected for the
reason journal/0124 lived through: a blob hash says **"something moved"**, and the author of
the move then has to bisect the artifact by hand — which is the throwaway-harness work the
whole sweep exists to abolish. **A move must name its artifact.** Three artifacts, three
constants, three test names, each failing with the mechanism it wants stated.

Each constant carries what an authorized re-capture owes, in the 0124 house style, and the
useful half of that is not the ceremony — it is naming *the interesting failure*. For
`geotherm`: a move here with `GOLDEN_SURFACE` **held** means the geotherm rule changed rather
than the crust under it, and the thing to report is what it did to coal rank, the plane's
only in-run consumer. For `head`: a move with `GOLDEN_SURFACE` held means the potential's own
solve changed, and `GOLDEN_FLUX` is the corroborating hash, because the vertical face family
is fed from `head` and from nothing else. For `chapters`: the table is a pure function of the
kinematic inputs, so a move names one of them.

## The part that will outlive the three constants

A prose table of classifications is a document, and documents go stale in silence. This one
would go stale the first time somebody adds a member to `DeepField`, which is a thing that
has happened four times in a month.

So the enumeration is a **pattern**:

```rust
let DeepField {
    w, wp, cell_m, surf, regolith, strata, ledgers, recv, area,
    lake, flux, exhum, t_crust, geotherm, head, chapters,
} = field();
```

An exhaustive destructure. Adding a member **stops the test compiling**, and the author has
to classify it before the gate is green again. Each binding is then used in the assertion
that states its classification, so it cannot be satisfied by a row of underscores.

CLAUDE.md carries a note, written after a count in a read-first justification went stale
inside the argument it supported, that this is *"exactly what an enumeration-completeness
check would catch, and we still have none."* This is one. It is narrow — it covers the
artifact inventory and nothing else — but it is the mechanism, not a rule asking somebody to
remember, and the difference between those two is the whole of the `flow_cost_probe` lesson.

## The one member that is deliberately not goldened

`ledgers` is empty in every world a player gets: `weather_inventory` is off in
`production_config`, and off means the S-5 identity default, an empty record. There is
nothing to hash. Goldening the *flag-on* ledger would have meant a second full world build
for an artifact nobody ships — a fifth of the gate's cost, for a configuration reachable only
behind a dev flag.

The honest disposal is that **its emptiness is the tripwire**. The enumeration asserts it,
and the assertion message carries the debt where it will actually be read:

> the shipped world grew a fact ledger — `weather_inventory` is on, and this artifact now
> needs a golden of its own

That is read-first item 5's rule applied sideways. *A one-directional pointer is not a
pointer*: a note in the ROADMAP saying "when the flag flips, add a golden" is a note to
whoever already knows. A failing assertion at the moment the flag flips is a pointer from the
stale end — and the stale end is where the next author enters.

## The other half of the finding walked in from the far side

Halfway through this slice the member-#0 far slice merged, and it brought a twin.

Its brief had said *"goldens move — re-capture with the why"*, which was exactly right for
what it did: it changed the far surface class draw twice over — nearest deep cell to
`CoarseField::sample_dithered` membership dither, plus a canonical class-order change. **Not
one hash in the workspace moved.** `contents_contract`'s
`generated_world_is_byte_identical_to_the_pre_contract_goldens` passed untouched, and
correctly so: it hashes `generate_chunk_with_materials`, and `generate_chunk` has not
consulted `surface_class` since journal/0074. The deep-time goldens sit upstream of the
collapse tier entirely. **`coarse_surface` — every metre of ground beyond the loaded radius —
was fingerprinted by nothing**, and that slice's gate would have been just as green had it
broken the far field outright.

Two agents, two sides of the collapse tier, one evening, and the same sentence. The far
field's case is if anything the sharper of the two, because of *how* it was found: not by a
sweep, but by **an acceptance criterion that could not fire.** The brief predicted a hash
move; the absence of one was read, at first, as evidence about the change. It was evidence
about the instrument.

And it names a limit of the mechanism I had just been pleased with. The exhaustive
destructure is complete over **one struct**. `coarse_surface` is a *query*, not a member, so
nothing in that test could ever have caught it. A completeness check is only complete over
the thing it enumerates, and shipping one without saying so is how the next reader
over-trusts it.

`GOLDEN_FAR_SURFACE` is the fourth constant. The two ROADMAP entries closed together.

## The far sample, and a number I would rather not have found

A far-field golden needs a *sample* — `coarse_surface` is a query over an unbounded pyramid,
so unlike the three planes there is no container whose order is the canonical one. The sample
**is** the normalization: 192² columns on a 509-voxel stride, ±48,864 voxels, which covers the
`Extent::Small` fixture's civilized ±40,960 and reaches ~7 km into the border wilds — where
the pyramid runs forever and the record does not, and where the far field is the *only*
answer.

The stride is prime on purpose. The coarse cell is 16,384 voxels and the chunk is 32, both
powers of two, so any power-of-two stride puts every sample on the same phase of both lattices
— and a far-field golden blind to lattice phase is blind to the seam artifacts this half of
the world is most prone to.

Then the sample was measured, and the measurement is the uncomfortable part. **This fixture is
an almost entirely submarine world.** Over 36,864 columns the surface height runs −2,642 … −3,
and **160 columns — 0.43 % — front with anything but the ocean block.** The highest ground
anywhere in a coarse ±56,000-voxel scan is **+7 voxels**.

So the honest accounting, which is written on the constant rather than left to be discovered:

- the **height** field is exercised completely — every column carries real, varying
  elevation, and height is the far field's dominant output;
- the **class** draw, which is what journal/0125 actually changed, is exercised by those 160
  columns. It would still trip — a class-order change over 160 columns is not going to hash
  identically — but this world cannot exercise the membership dither the way a land-bearing
  one would;
- so a far-field golden on a fixture with real continent is **filed as residue**, and its
  natural home is `contents_contract.rs`, whose Medium seeds are the worlds with land — which
  is exactly where the original heir spec put it before this suite took it.

The first draft of the sample was stride 2039 × 48, and it caught **six** non-ocean columns
out of 2,304. That is the version I would have shipped if I had not printed the composition.
A tripwire whose coverage of its own motivating change is decided by phase luck is the A-3
shape — green for a reason unrelated to what it asserts — and the difference between the two
drafts is one `println!` of a histogram.

## The numbers

```text
GOLDEN_GEOTHERM    0xDBDE_C405_EBE0_239E   25,600 cells
GOLDEN_HEAD        0x1013_984C_2B7B_BCF9   25,600 cells
GOLDEN_CHAPTERS    0xF25B_E0C3_CF39_AC55    9 chapters × 3 plates = 27 plate records
GOLDEN_FAR_SURFACE 0x1424_7B7C_AB51_EFA5   36,864 columns, stride 509
```

All on the `providers_common` golden fixture (seed `0x0B0A_57EE_0059`, `Extent::Small`) — the
same world the incumbent goldens are captured on, built **once** for the suite. The far
tripwire shares it: `golden_field()` is literally `Pregen::run` followed by `build_field`, so
holding the intermediate `Pregen` gives the `WorldGenerator` its world for free. Seven tests,
**6.75 s** of gate wall-clock, which is the whole cost of the slice.

Worth noticing how small the chapter table is: **27 plate records**, against 25,600 cells in
each plane and 36,864 far columns. It is the artifact that could have moved furthest with the
least noticed — three orders of magnitude below the noise floor of anything else the ritual
keeps, exported, read by nothing in production, and pinned until today only by its own length.

## A numbering repair, recorded because it is the same failure in miniature

Two agents branched from bases that each thought `0124` was free, and an integrator commit
(`2d6ab26`) split the references: the Schedule slice kept `0124`, the far slice became `0125`,
and twenty-eight references were classified by author and renumbered accordingly.

It missed two, in opposite directions, and both are the kind only a reader who *follows* the
pointer notices:

- `journal/0125-the-type-that-was-its-own-first-customer.md` still opened with **`# 0124`** —
  the one place in the corpus that names the file's own number disagreed with the filename.
- `spines.md`'s note that *"the `Schedule::Seed` row was added later the same day by
  journal/0124"* was renumbered to `0125`. `Schedule::Seed` was added by the **Schedule**
  slice, which kept its number; that reference should not have moved.

Both repaired here. The reason to write them down rather than fix them silently is that they
are the sweep's own thesis at another scale: a renumbering pass is a mechanical edit whose
correctness nothing checks, and the two it got wrong are exactly the two that require knowing
*which agent wrote the claim* — which is not visible in the text being edited.

## What this leaves

- **A far-field golden on a world with land** — the residue above; `contents_contract.rs` is
  its home.
- **`Pregen::grid` and `Pregen::pipeline` were not walked.** The sweep was scoped to the
  deep-time ritual's outputs. The collapse-tier world is goldened separately
  (`contents_contract.rs`, three seeds × blocks/materials/table), but nobody has said, per
  member, which golden catches a change in the *coarse* grid. Same enumeration, one tier up.
- **`recv`/`area`/`lake` are double-pinned** — hashed by `surface_fingerprint`, and their
  authority `flux` hashed by `GOLDEN_FLUX`. That is the S-3 shape (a summary goldened beside
  its authority), it is pre-existing rather than introduced here, and it expires when FLOW
  continuation (e) deletes the receiver export.
- **`GOLDEN_FLUX` lives on `flux_record.rs`'s own fixture**, not the `providers_common`
  golden world. Deliberately not re-pinned here: a second hash of one artifact is a second
  thing to re-capture, not a second thing detected.
