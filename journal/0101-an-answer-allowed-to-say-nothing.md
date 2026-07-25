# 0101 — An answer allowed to say nothing

*2026-07-25 — `identify(pos)`, the honest identity surface: first slice*

## The bug you cannot see from inside the answer

A walk stood at (84 185 m, 9 212 m), asked `world_get_contents` about the rock
under its feet, and was told `dc:air` — for twelve voxels, while the very same
pose response said `eye_in_solid: true`. It filed the observation honestly as
*reported, not diagnosed*: "the record reads empty over solid ground."

The diagnosis (`docs/audits/2026-07-25-contents-empty-over-solid-diagnosis.md`)
falsified the premise and found something worse underneath. The world was solid
and correct. `eye_in_solid` and the query's own `block` field read the *same*
`block_at` and both said `dc:stone`; they never disagreed. What said `dc:air`
was the derived `classified` field — and beside it, `has_contents: true`, a flag
whose entire job is to say *"no record here"*, saying the opposite.

The mechanism is one line, and it is the kind of line nobody reads twice:

```rust
pub fn contents_at(&self, p: Vec3i) -> Option<VoxelContents> {
    let source = self.contents_source.as_ref()?;
    let grid = source(cpos)?;          // <-- None only if the whole CHUNK is empty
    Some(grid.get(lx, ly, lz))         // <-- else ALWAYS Some, even of EMPTY
}
```

`chunk_contents` returns a grid if **any** voxel in the 32³ chunk carries a
record, and inside that grid an unrecorded voxel resolves to a perfectly
ordinary `VoxelContents::EMPTY`. So the only channel that could have carried
*"this voxel has no record"* — `Option::None` — had already been spent on a
**whole-chunk** condition, inherited from the mesher, which wants a grid or
nothing and does not care which voxels inside it are blank.

Everything downstream then treated `Some(EMPTY)` as data. `has_contents` became
`contents.is_some()`: a chunk-resolution fact published under a voxel-resolution
name. `classified` became `classify(EMPTY) == Block::Air` — precisely the
operation `dc-core/src/classify.rs`'s own module doc forbids, in a paragraph
that names *"the unrecorded basement below the deep-time record"* by name. The
core knew the rule. The API had already destroyed the distinction the rule needs
before it could be applied.

This is *"a summary is not an authority"* **inverted**. journal/0097's other
finding was a voxel-level summary (`Block`) worn as an authority about
materials. This one is a **chunk-level** summary (`has_contents`) worn as an
authority about **one voxel** — the same doctrine, one tier up, and the mirror
image. A `bool` named after the thing it is not measuring.

The band's extent — 288 to 299 — was set by the chunk floor at `9 × 32 = 288`.
Not by anything in the world.

## Why it survived: the only consumer that reads contents at scale is immune

The mesher gates on the block before it ever consults contents, and the far
field explicitly guards on `!c.is_empty()`. Neither can be fooled. The two
surfaces that *were* fooled are the **F3 look-at HUD** and
`world_get_contents` — the diagnostic instruments, the ones a human and an agent
use to ask the world what it is. The defect hid for months because it only lied
to the people asking questions.

The F3 HUD's punchline is exact: it already **has** the correct branch —

```rust
let Some(c) = contents else {
    return format!("block: {block}\n(no contents record here)");
};
```

— and that branch was **unreachable** for exactly the voxels it was written for.
The right message existed, in the right place, and could never fire.

## The decision: untiered

This slice was sequenced as `identify(pos) -> { tier, payload }`, with Near /
Mid / Far tiers derived from the LOD ladder. The user killed the tiers the
morning of dispatch, with one question: *"if this is a world query then why
tiered at all when we can inspect chunk and read the voxel?"*

The provenance turned out to matter. The tiering was **assistant-originated** —
an artifact of one request ("a way to get voxel composition by looking at it")
being split into several instruments, which then hardened into architecture
without ever being re-challenged. And it encoded a conflation: *"what is at
world position X"* is a **world** question with one true answer, to which viewer
distance is irrelevant; *"what is the renderer showing at X"* is a **render**
question, legitimately per-viewer, legitimately vaguer, and the only one with
any business reading a LOD ladder. The ROADMAP entry's own first paragraph said
so — *"a world question, not a camera question"* — and then built a camera
concept into the world query anyway.

Cost was the strongest counter-argument and it fails cleanly: generation is
pure-of-position, so the honest answer is *always* derivable. There is no
fundamental barrier, only a latency one. **Cost may change the policy or the
latency; it must never change the answer.** Returning a vaguer truth because the
honest one was expensive is exactly the summary-wearing-an-authority's-clothes
this whole arc exists to retire.

Dropping tiers also dissolved the open question the brief flagged as
*must-be-settled-before-dispatch* — "whose ladder?", given that `LodLadder` is
per-viewer dc-client state and `identify` must be reachable headlessly. There
was no good answer because the question was malformed.

What survived: the payload is **uniformly a mixture**. A single dominant
material is just a one-component mixture, so the day the far field goes speckled
there is no special case to migrate — and no far tier left to special-case
anyway.

## The enabler: the block already disambiguates

The fix needed a way to tell *"nothing is here"* from *"we have no record"*
when the composition grid hands back `EMPTY` for both. The audit had already
proven the discriminator, and it costs nothing:

- unrecorded basement is `Block::Stone` **by construction** (`collapse.rs`);
- **no voxel below a column's height can ever be `Block::Air`** — the only `Air`
  branch is `vy > h`.

So, when the record is empty or absent: `block == Air` ⇒ genuinely empty;
anything else ⇒ **unrecorded**. `block_at` was already being read on the same
line of the same function. **Zero dc-worldgen changes** — which mattered, since
three sibling agents owned that crate for the day.

I verified the converse before building on it: can a *recorded* voxel have empty
contents? No. The surface veneer's fill is `.clamp(1.0, 8.0)` eighths, so a
recorded surface voxel is never zero-filled, and every buried write either
interns a real mixture or `continue`s past the voxel entirely. Empty-and-solid
means unrecorded, always.

## The shape

```rust
pub enum Identity {
    /// No composition record backs this voxel. The absence of a statement,
    /// not a statement of absence.
    Unrecorded,
    /// The honest full mixture. `EMPTY` here is a *positive* answer —
    /// "nothing is in this voxel" — and is a different VALUE from Unrecorded.
    Mixture(VoxelContents),
}
```

Two variants carrying three answers, and the distinction the defect destroyed is
the one between the first two. It is in the **type**, not in a convention about
how to read a `bool`.

The symmetry is deliberate, and it is the part that took the longest to settle.
The anti-shape being guarded is A-1 — *"empty" standing in for "unrecorded"* —
and the tempting overcorrection is its exact mirror: making air `Unrecorded`
too, on the argument that the generator never writes a record for air either.
That reading is defensible from `classify.rs`'s own wording, and it is wrong
here, because it lets *"unrecorded"* stand in for *"empty"* — the same sin
pointed the other way, and it would strip the query of the one thing it can
always state with certainty. Air is a **positive, complete** composition
statement, derivable from the block alone, with no source installed, anywhere in
the world. So sky answers `Mixture(EMPTY)`, and solid-with-no-record answers
`Unrecorded`, and they are never the same value.

One detail preserves an existing property: a **non-empty** record wins outright,
whatever the block says. That is what keeps `block: dc:air` over recorded
granite legible as *"this voxel was edited"* — the divergence signal the
`get_contents` payload documents. The air rule only fires when the record is
empty or absent, so nothing that was legible before became silent.

## What changed, concretely

- `HostWorld::identify(pos) -> Identity` — the honest surface. Untiered,
  position-addressed, resident-first (the block half reads the resident chunk
  and falls back to the generator), derived-if-absent.
- `contents_at` is now documented as **the source, not the answer**: its `None`
  is a chunk-level fact, and callers who want the truth ask `identify`.
- `has_contents` is now a **per-voxel** fact; `classified` echoes the stored
  block for an unrecorded voxel instead of naming a mixture that does not exist.
  The docs in `payload.rs`, `schema.rs` (the tool doc an agent actually reads)
  and CLAUDE.md were brought back into agreement with behaviour — the schema doc
  now says, in the plainest words available: *a solid block with
  `has_contents: false` is NOT air; it is rock whose composition the world
  cannot state.*
- `character_sense_raycast` returns `None` for unrecorded rock, which is what
  its doc always promised, instead of `Some(<empty view>)`.
- The F3 HUD's `(no contents record here)` branch is **reachable**.

## The census

Measured through the real query path (`crates/dc-client/examples/identify_census.rs`),
production world, seed 1337 / Medium / flag-OFF — the same lattice the diagnosis
probe used, so the numbers are directly comparable. The example computes both
answers at every voxel: **BEFORE** from the raw `contents_at` (the pre-`identify`
semantics, reconstructed) and **AFTER** from `identify`.

At journal/0097's own station (world 84 185 m / 9 212 m → voxel 93 539 / 10 236,
surface `h = 300`):

```
CENSUS_STATION
```

Across 169 columns on a 4 096-voxel lattice, top 65 voxels each:

```
CENSUS_TOTALS
```

The load-bearing assertion is not the zero — it is the accounting identity the
example checks: **every** BEFORE-phantom voxel and **every** BEFORE-honest
no-record voxel lands in `UNRECORDED`, and the recorded-mixture count does not
move. The lie was converted, not deleted; nothing that had an answer lost one.

## Seams left open, named

**The path is edit-blind for composition, and this slice does not change that.**
The contents source is a pure function of position — worldgen re-derivation — so
an edit writes a `Block` and the mixture underneath it does not move. `identify`
reports the honest *stored* block beside the honest *derived* mixture and lets
the divergence show; it cannot report the mixture an edit produced, because no
such mixture is stored anywhere. Place a block in mid-air and `identify` says
`Unrecorded` (honest, and better than the old `dc:air` over your own stone);
break recorded granite and it still reports granite beside `block: dc:air`
(the edit signal). Closing this is the arc's **runtime edit-fact overlay** —
contents as *derivable base + edit facts*, the north star's first runtime-process
milestone — and it is what makes *"break gives you the real mixture"* true. It
is in the arc's continuation slot; the arc **stays open**.

Not done here, deliberately: `Block` is not deleted, `classify` is not retired,
no storage or wire format moved, and no consumer was drained beyond the three
named. `identify` is a new honest surface *beside* them.

> blogworthy: **lens 3 (reflexions in a deepsim codebase)** — an `Option::None`
> is a communication channel, and this one had been spent. The mesher wanted "a
> whole grid or nothing"; the query inherited its shape and, silently, its
> *meaning*. The bug is not in either consumer — it is in the assumption that
> one `Option` could serve both a chunk-shaped question and a voxel-shaped one.
> Pairs with the symmetry argument above: guarding against "empty stands in for
> unrecorded" without walking straight into "unrecorded stands in for empty".
>
> blogworthy: **lens 1 (AI-native development)** — the tiering in this arc's
> spec was assistant-originated, and survived several passes because it read
> like architecture. It died to one user question that took ten seconds to ask.
> The provenance of a design idea is worth recording *at the time*, because
> "who proposed this and in answer to what" is the cheapest available test of
> whether it was ever load-bearing.
